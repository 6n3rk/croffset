use std::borrow::Cow;
use std::ffi::CString;
use std::ffi::OsString;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::mem::size_of;
use std::mem::ManuallyDrop;
use std::os::raw::c_char;
use std::os::unix::ffi::OsStringExt as _;
use std::path::PathBuf;
use std::ptr;
use std::slice;

use blazesym::normalize::Apk;
use blazesym::normalize::Elf;
use blazesym::normalize::NormalizeOpts;
use blazesym::normalize::Normalizer;
use blazesym::normalize::Reason;
use blazesym::normalize::Unknown;
use blazesym::normalize::UserMeta;
use blazesym::normalize::UserOutput;
use blazesym::Addr;

use crate::blaze_err;
#[cfg(doc)]
use crate::blaze_err_last;
use crate::set_last_err;
use crate::util::slice_from_user_array;


/// C ABI compatible version of [`blazesym::normalize::Normalizer`].
pub type blaze_normalizer = Normalizer;


/// Options for configuring [`blaze_normalizer`] objects.
#[repr(C)]
#[derive(Debug)]
pub struct blaze_normalizer_opts {
    /// The size of this object's type.
    ///
    /// Make sure to initialize it to `sizeof(<type>)`. This member is used to
    /// ensure compatibility in the presence of member additions.
    pub type_size: usize,
    /// Whether or not to cache `/proc/<pid>/maps` contents.
    ///
    /// Setting this flag to `true` is not generally recommended, because it
    /// could result in addresses corresponding to mappings added after caching
    /// may not be normalized successfully, as there is no reasonable way of
    /// detecting staleness.
    pub cache_maps: bool,
    /// Whether to read and report build IDs as part of the normalization
    /// process.
    pub build_ids: bool,
    /// Whether or not to cache build IDs. This flag only has an effect
    /// if build ID reading is enabled in the first place.
    pub cache_build_ids: bool,
    /// Unused member available for future expansion. Must be initialized
    /// to zero.
    pub reserved: [u8; 5],
}

impl Default for blaze_normalizer_opts {
    fn default() -> Self {
        Self {
            type_size: size_of::<Self>(),
            cache_maps: false,
            build_ids: false,
            cache_build_ids: false,
            reserved: [0; 5],
        }
    }
}


/// Options influencing the address normalization process.
#[repr(C)]
#[derive(Debug)]
pub struct blaze_normalize_opts {
    /// The size of this object's type.
    ///
    /// Make sure to initialize it to `sizeof(<type>)`. This member is used to
    /// ensure compatibility in the presence of member additions.
    pub type_size: usize,
    /// Whether or not addresses are sorted (in ascending order) already.
    ///
    /// Normalization always happens on sorted addresses and if the addresses
    /// are sorted already, the library does not need to sort and later restore
    /// original ordering, speeding up the normalization process.
    pub sorted_addrs: bool,
    /// Whether to report `/proc/<pid>/map_files/` entry paths or work
    /// with symbolic paths mentioned in `/proc/<pid>/maps` instead.
    ///
    /// Relying on `map_files` may make sense in cases where
    /// symbolization happens on the local system and the reported paths
    /// can be worked with directly. In most other cases where one wants
    /// to attach meaning to symbolic paths on a remote system (e.g., by
    /// using them for file look up) symbolic paths are probably the
    /// better choice.
    pub map_files: bool,
    /// Unused member available for future expansion. Must be initialized
    /// to zero.
    pub reserved: [u8; 6],
}

impl Default for blaze_normalize_opts {
    fn default() -> Self {
        Self {
            type_size: size_of::<Self>(),
            sorted_addrs: false,
            map_files: false,
            reserved: [0; 6],
        }
    }
}

impl From<blaze_normalize_opts> for NormalizeOpts {
    fn from(opts: blaze_normalize_opts) -> Self {
        let blaze_normalize_opts {
            type_size: _,
            sorted_addrs,
            map_files,
            reserved: _,
        } = opts;
        Self {
            sorted_addrs,
            map_files,
            _non_exhaustive: (),
        }
    }
}


/// Create an instance of a blazesym normalizer in the default
/// configuration.
///
/// C ABI compatible version of [`blazesym::normalize::Normalizer::new()`].
/// Please refer to its documentation for the default configuration in use.
///
/// On success, the function creates a new [`blaze_normalizer`] object and
/// returns it. The resulting object should be released using
/// [`blaze_normalizer_free`] once it is no longer needed.
///
/// On error, the function returns `NULL` and sets the thread's last error to
/// indicate the problem encountered. Use [`blaze_err_last`] to retrieve this
/// error.
#[no_mangle]
pub extern "C" fn blaze_normalizer_new() -> *mut blaze_normalizer {
    let normalizer = Normalizer::new();
    let normalizer_box = Box::new(normalizer);
    let () = set_last_err(blaze_err::BLAZE_ERR_OK);
    Box::into_raw(normalizer_box)
}


/// Create an instance of a blazesym normalizer.
///
/// On success, the function creates a new [`blaze_normalizer`] object and
/// returns it. The resulting object should be released using
/// [`blaze_normalizer_free`] once it is no longer needed.
///
/// On error, the function returns `NULL` and sets the thread's last error to
/// indicate the problem encountered. Use [`blaze_err_last`] to retrieve this
/// error.
///
/// # Safety
/// - `opts` needs to point to a valid [`blaze_normalizer_opts`] object
#[no_mangle]
pub unsafe extern "C" fn blaze_normalizer_new_opts(
    opts: *const blaze_normalizer_opts,
) -> *mut blaze_normalizer {
    if !input_zeroed!(opts, blaze_normalizer_opts) {
        let () = set_last_err(blaze_err::BLAZE_ERR_INVALID_INPUT);
        return ptr::null_mut()
    }
    let opts = input_sanitize!(opts, blaze_normalizer_opts);

    let blaze_normalizer_opts {
        type_size: _,
        cache_maps,
        build_ids,
        cache_build_ids,
        reserved: _,
    } = opts;

    let normalizer = Normalizer::builder()
        .enable_maps_caching(cache_maps)
        .enable_build_ids(build_ids)
        .enable_build_id_caching(cache_build_ids)
        .build();
    let normalizer_box = Box::new(normalizer);
    let () = set_last_err(blaze_err::BLAZE_ERR_OK);
    Box::into_raw(normalizer_box)
}


/// Free a blazesym normalizer.
///
/// Release resources associated with a normalizer as created by
/// [`blaze_normalizer_new`], for example.
///
/// # Safety
/// The provided normalizer should have been created by
/// [`blaze_normalizer_new`].
#[no_mangle]
pub unsafe extern "C" fn blaze_normalizer_free(normalizer: *mut blaze_normalizer) {
    if !normalizer.is_null() {
        // SAFETY: The caller needs to ensure that `normalizer` is a
        //         valid pointer.
        drop(unsafe { Box::from_raw(normalizer) });
    }
}


/// A file offset or non-normalized address along with an index into the
/// associated [`blaze_user_meta`] array (such as
/// [`blaze_normalized_user_output::metas`]).
#[repr(C)]
#[derive(Debug)]
pub struct blaze_normalized_output {
    /// The file offset or non-normalized address.
    pub output: u64,
    /// The index into the associated [`blaze_user_meta`] array.
    pub meta_idx: usize,
}

impl From<(u64, usize)> for blaze_normalized_output {
    fn from((output, meta_idx): (u64, usize)) -> Self {
        Self { output, meta_idx }
    }
}


/// The valid variant kind in [`blaze_user_meta`].
#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum blaze_user_meta_kind {
    /// [`blaze_user_meta_variant::unknown`] is valid.
    BLAZE_USER_META_UNKNOWN,
    /// [`blaze_user_meta_variant::apk`] is valid.
    BLAZE_USER_META_APK,
    /// [`blaze_user_meta_variant::elf`] is valid.
    BLAZE_USER_META_ELF,
}


/// C compatible version of [`Apk`].
#[repr(C)]
#[derive(Debug)]
pub struct blaze_user_meta_apk {
    /// The canonical absolute path to the APK, including its name.
    /// This member is always present.
    pub path: *mut c_char,
    /// Unused member available for future expansion.
    pub reserved: [u8; 8],
}

impl blaze_user_meta_apk {
    fn from(other: Apk) -> ManuallyDrop<Self> {
        let Apk {
            path,
            _non_exhaustive: (),
        } = other;

        let slf = Self {
            path: CString::new(path.into_os_string().into_vec())
                .expect("encountered path with NUL bytes")
                .into_raw(),
            reserved: [0u8; 8],
        };
        ManuallyDrop::new(slf)
    }

    unsafe fn free(self) {
        let Self { path, reserved: _ } = self;

        let _apk = Apk {
            path: PathBuf::from(OsString::from_vec(
                unsafe { CString::from_raw(path) }.into_bytes(),
            )),
            _non_exhaustive: (),
        };
    }
}


/// C compatible version of [`Elf`].
#[repr(C)]
#[derive(Debug)]
pub struct blaze_user_meta_elf {
    /// The path to the ELF file. This member is always present.
    pub path: *mut c_char,
    /// The length of the build ID, in bytes.
    pub build_id_len: usize,
    /// The optional build ID of the ELF file, if found.
    pub build_id: *mut u8,
    /// Unused member available for future expansion.
    pub reserved: [u8; 8],
}

impl blaze_user_meta_elf {
    fn from(other: Elf) -> ManuallyDrop<Self> {
        let Elf {
            path,
            build_id,
            _non_exhaustive: (),
        } = other;

        let slf = Self {
            path: CString::new(path.into_os_string().into_vec())
                .expect("encountered path with NUL bytes")
                .into_raw(),
            build_id_len: build_id
                .as_ref()
                .map(|build_id| build_id.len())
                .unwrap_or(0),
            build_id: build_id
                .map(|build_id| {
                    // SAFETY: We know the pointer is valid because it
                    //         came from a `Box`.
                    unsafe {
                        Box::into_raw(build_id.to_vec().into_boxed_slice())
                            .as_mut()
                            .unwrap()
                            .as_mut_ptr()
                    }
                })
                .unwrap_or_else(ptr::null_mut),
            reserved: [0u8; 8],
        };
        ManuallyDrop::new(slf)
    }

    unsafe fn free(self) {
        let blaze_user_meta_elf {
            path,
            build_id_len,
            build_id,
            reserved: _,
        } = self;

        let _elf = Elf {
            path: PathBuf::from(OsString::from_vec(
                unsafe { CString::from_raw(path) }.into_bytes(),
            )),
            build_id: (!build_id.is_null()).then(|| unsafe {
                Cow::Owned(
                    Box::<[u8]>::from_raw(slice::from_raw_parts_mut(build_id, build_id_len))
                        .into_vec(),
                )
            }),
            _non_exhaustive: (),
        };
    }
}


/// The reason why normalization failed.
///
/// The reason is generally only meant as a hint. Reasons reported may change
/// over time and, hence, should not be relied upon for the correctness of the
/// application.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum blaze_normalize_reason {
    /// The absolute address was not found in the corresponding process' virtual
    /// memory map.
    BLAZE_NORMALIZE_REASON_UNMAPPED,
    /// The `/proc/<pid>/maps` entry corresponding to the address does not have
    /// a component (file system path, object, ...) associated with it.
    BLAZE_NORMALIZE_REASON_MISSING_COMPONENT,
    /// The address belonged to an entity that is currently unsupported.
    BLAZE_NORMALIZE_REASON_UNSUPPORTED,
}

impl From<Reason> for blaze_normalize_reason {
    fn from(reason: Reason) -> Self {
        use blaze_normalize_reason::*;

        match reason {
            Reason::Unmapped => BLAZE_NORMALIZE_REASON_UNMAPPED,
            Reason::MissingComponent => BLAZE_NORMALIZE_REASON_MISSING_COMPONENT,
            Reason::Unsupported => BLAZE_NORMALIZE_REASON_UNSUPPORTED,
            _ => unreachable!(),
        }
    }
}


/// Retrieve a textual representation of the reason of a normalization failure.
#[no_mangle]
pub extern "C" fn blaze_normalize_reason_str(err: blaze_normalize_reason) -> *const c_char {
    use blaze_normalize_reason::*;

    match err as i32 {
        e if e == BLAZE_NORMALIZE_REASON_UNMAPPED as i32 => {
            Reason::Unmapped.as_bytes().as_ptr().cast()
        }
        e if e == BLAZE_NORMALIZE_REASON_MISSING_COMPONENT as i32 => {
            Reason::MissingComponent.as_bytes().as_ptr().cast()
        }
        e if e == BLAZE_NORMALIZE_REASON_UNSUPPORTED as i32 => {
            Reason::Unsupported.as_bytes().as_ptr().cast()
        }
        _ => b"unknown reason\0".as_ptr().cast(),
    }
}


/// C compatible version of [`Unknown`].
#[repr(C)]
#[derive(Debug)]
pub struct blaze_user_meta_unknown {
    /// The reason why normalization failed.
    ///
    /// The provided reason is a best guess, hinting at what ultimately
    /// prevented the normalization from being successful.
    pub reason: blaze_normalize_reason,
    /// Unused member available for future expansion.
    pub reserved: [u8; 7],
}

impl blaze_user_meta_unknown {
    fn from(other: Unknown) -> ManuallyDrop<Self> {
        let Unknown {
            reason,
            _non_exhaustive: (),
        } = other;

        let slf = Self {
            reason: reason.into(),
            reserved: [0u8; 7],
        };
        ManuallyDrop::new(slf)
    }

    fn free(self) {
        let blaze_user_meta_unknown {
            reason: _,
            reserved: _,
        } = self;
    }
}


/// The actual variant data in [`blaze_user_meta`].
#[repr(C)]
pub union blaze_user_meta_variant {
    /// Valid on [`blaze_user_meta_kind::BLAZE_USER_META_APK`].
    pub apk: ManuallyDrop<blaze_user_meta_apk>,
    /// Valid on [`blaze_user_meta_kind::BLAZE_USER_META_ELF`].
    pub elf: ManuallyDrop<blaze_user_meta_elf>,
    /// Valid on [`blaze_user_meta_kind::BLAZE_USER_META_UNKNOWN`].
    pub unknown: ManuallyDrop<blaze_user_meta_unknown>,
}

impl Debug for blaze_user_meta_variant {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct(stringify!(blaze_user_meta_variant)).finish()
    }
}


/// C ABI compatible version of [`UserMeta`].
#[repr(C)]
#[derive(Debug)]
pub struct blaze_user_meta {
    /// The variant kind that is present.
    pub kind: blaze_user_meta_kind,
    /// The actual variant with its data.
    pub variant: blaze_user_meta_variant,
}

impl blaze_user_meta {
    fn from(other: UserMeta) -> ManuallyDrop<Self> {
        let slf = match other {
            UserMeta::Apk(apk) => Self {
                kind: blaze_user_meta_kind::BLAZE_USER_META_APK,
                variant: blaze_user_meta_variant {
                    apk: blaze_user_meta_apk::from(apk),
                },
            },
            UserMeta::Elf(elf) => Self {
                kind: blaze_user_meta_kind::BLAZE_USER_META_ELF,
                variant: blaze_user_meta_variant {
                    elf: blaze_user_meta_elf::from(elf),
                },
            },
            UserMeta::Unknown(unknown) => Self {
                kind: blaze_user_meta_kind::BLAZE_USER_META_UNKNOWN,
                variant: blaze_user_meta_variant {
                    unknown: blaze_user_meta_unknown::from(unknown),
                },
            },
            _ => unreachable!(),
        };
        ManuallyDrop::new(slf)
    }

    unsafe fn free(self) {
        match self.kind {
            blaze_user_meta_kind::BLAZE_USER_META_APK => unsafe {
                ManuallyDrop::into_inner(self.variant.apk).free()
            },
            blaze_user_meta_kind::BLAZE_USER_META_ELF => unsafe {
                ManuallyDrop::into_inner(self.variant.elf).free()
            },
            blaze_user_meta_kind::BLAZE_USER_META_UNKNOWN => {
                ManuallyDrop::into_inner(unsafe { self.variant.unknown }).free()
            }
        }
    }
}


/// An object representing normalized user addresses.
///
/// C ABI compatible version of [`UserOutput`].
#[repr(C)]
#[derive(Debug)]
pub struct blaze_normalized_user_output {
    /// The number of [`blaze_user_meta`] objects present in `metas`.
    pub meta_cnt: usize,
    /// An array of `meta_cnt` objects.
    pub metas: *mut blaze_user_meta,
    /// The number of [`blaze_normalized_output`] objects present in `outputs`.
    pub output_cnt: usize,
    /// An array of `output_cnt` objects.
    pub outputs: *mut blaze_normalized_output,
    /// Unused member available for future expansion.
    pub reserved: [u8; 8],
}

impl blaze_normalized_user_output {
    fn from(other: UserOutput) -> ManuallyDrop<Self> {
        let slf = Self {
            meta_cnt: other.meta.len(),
            metas: unsafe {
                Box::into_raw(
                    other
                        .meta
                        .into_iter()
                        .map(blaze_user_meta::from)
                        .map(ManuallyDrop::into_inner)
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                )
                .as_mut()
                .unwrap()
                .as_mut_ptr()
            },
            output_cnt: other.outputs.len(),
            outputs: unsafe {
                Box::into_raw(
                    other
                        .outputs
                        .into_iter()
                        .map(blaze_normalized_output::from)
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                )
                .as_mut()
                .unwrap()
                .as_mut_ptr()
            },
            reserved: [0u8; 8],
        };
        ManuallyDrop::new(slf)
    }
}


unsafe fn blaze_normalize_user_addrs_impl(
    normalizer: *const blaze_normalizer,
    pid: u32,
    addrs: *const Addr,
    addr_cnt: usize,
    opts: &NormalizeOpts,
) -> *mut blaze_normalized_user_output {
    // SAFETY: The caller needs to ensure that `normalizer` is a valid
    //         pointer.
    let normalizer = unsafe { &*normalizer };
    // SAFETY: The caller needs to ensure that `addrs` is a valid pointer and
    //         that it points to `addr_cnt` elements.
    let addrs = unsafe { slice_from_user_array(addrs, addr_cnt) };
    let result = normalizer.normalize_user_addrs_opts(pid.into(), &addrs, opts);
    match result {
        Ok(addrs) => {
            let output_box = Box::new(ManuallyDrop::into_inner(
                blaze_normalized_user_output::from(addrs),
            ));
            let () = set_last_err(blaze_err::BLAZE_ERR_OK);
            Box::into_raw(output_box)
        }
        Err(err) => {
            let () = set_last_err(err.kind().into());
            ptr::null_mut()
        }
    }
}


/// Normalize a list of user space addresses.
///
/// C ABI compatible version of [`Normalizer::normalize_user_addrs`].
///
/// `pid` should describe the PID of the process to which the addresses
/// belongs. It may be `0` if they belong to the calling process.
///
/// On success, the function creates a new [`blaze_normalized_user_output`]
/// object and returns it. The resulting object should be released using
/// [`blaze_user_output_free`] once it is no longer needed.
///
/// On error, the function returns `NULL` and sets the thread's last error to
/// indicate the problem encountered. Use [`blaze_err_last`] to retrieve this
/// error.
///
/// # Safety
/// - `addrs` needs to be a valid pointer to `addr_cnt` addresses
#[no_mangle]
pub unsafe extern "C" fn blaze_normalize_user_addrs(
    normalizer: *const blaze_normalizer,
    pid: u32,
    addrs: *const Addr,
    addr_cnt: usize,
) -> *mut blaze_normalized_user_output {
    let opts = NormalizeOpts::default();

    unsafe { blaze_normalize_user_addrs_impl(normalizer, pid, addrs, addr_cnt, &opts) }
}


/// Normalize a list of user space addresses.
///
/// C ABI compatible version of [`Normalizer::normalize_user_addrs_opts`].
///
/// `pid` should describe the PID of the process to which the addresses
/// belongs. It may be `0` if they belong to the calling process.
///
/// `opts` should point to a valid [`blaze_normalize_opts`] object.
///
/// On success, the function creates a new [`blaze_normalized_user_output`]
/// object and returns it. The resulting object should be released using
/// [`blaze_user_output_free`] once it is no longer needed.
///
/// On error, the function returns `NULL` and sets the thread's last error to
/// indicate the problem encountered. Use [`blaze_err_last`] to retrieve this
/// error.
///
/// # Safety
/// - `addrs` needs to be a valid pointer to `addr_cnt` addresses
#[no_mangle]
pub unsafe extern "C" fn blaze_normalize_user_addrs_opts(
    normalizer: *const blaze_normalizer,
    pid: u32,
    addrs: *const Addr,
    addr_cnt: usize,
    opts: *const blaze_normalize_opts,
) -> *mut blaze_normalized_user_output {
    if !input_zeroed!(opts, blaze_normalize_opts) {
        let () = set_last_err(blaze_err::BLAZE_ERR_INVALID_INPUT);
        return ptr::null_mut()
    }
    let opts = input_sanitize!(opts, blaze_normalize_opts);
    let opts = NormalizeOpts::from(opts);

    unsafe { blaze_normalize_user_addrs_impl(normalizer, pid, addrs, addr_cnt, &opts) }
}


/// Free an object as returned by [`blaze_normalize_user_addrs`] or
/// [`blaze_normalize_user_addrs_opts`].
///
/// # Safety
/// The provided object should have been created by
/// [`blaze_normalize_user_addrs`] or
/// [`blaze_normalize_user_addrs_opts`].
#[no_mangle]
pub unsafe extern "C" fn blaze_user_output_free(output: *mut blaze_normalized_user_output) {
    if output.is_null() {
        return
    }

    // SAFETY: The caller should make sure that `output` was created by one of
    //         our blessed functions.
    let user_output = unsafe { Box::from_raw(output) };
    let addr_metas = unsafe {
        Box::<[blaze_user_meta]>::from_raw(slice::from_raw_parts_mut(
            user_output.metas,
            user_output.meta_cnt,
        ))
    }
    .into_vec();
    let _norm_addrs = unsafe {
        Box::<[blaze_normalized_output]>::from_raw(slice::from_raw_parts_mut(
            user_output.outputs,
            user_output.output_cnt,
        ))
    }
    .into_vec();

    for addr_meta in addr_metas {
        let () = unsafe { addr_meta.free() };
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::CStr;
    use std::io;
    use std::path::Path;

    use blazesym::helper::read_elf_build_id;

    use test_tag::tag;

    use crate::blaze_err_last;


    /// Check that various types have expected sizes.
    #[test]
    #[cfg(target_pointer_width = "64")]
    fn type_sizes() {
        assert_eq!(size_of::<blaze_normalizer_opts>(), 16);
        assert_eq!(size_of::<blaze_normalize_opts>(), 16);
        assert_eq!(size_of::<blaze_user_meta_apk>(), 16);
        assert_eq!(size_of::<blaze_user_meta_elf>(), 32);
        assert_eq!(size_of::<blaze_user_meta_unknown>(), 8);
    }

    /// Exercise the `Debug` representation of various types.
    #[tag(miri)]
    #[test]
    fn debug_repr() {
        let output = blaze_normalized_output {
            output: 0x1337,
            meta_idx: 1,
        };
        assert_eq!(
            format!("{output:?}"),
            "blaze_normalized_output { output: 4919, meta_idx: 1 }"
        );

        let meta_kind = blaze_user_meta_kind::BLAZE_USER_META_APK;
        assert_eq!(format!("{meta_kind:?}"), "BLAZE_USER_META_APK");

        let apk = blaze_user_meta_apk {
            path: ptr::null_mut(),
            reserved: [0u8; 8],
        };
        assert_eq!(
            format!("{apk:?}"),
            "blaze_user_meta_apk { path: 0x0, reserved: [0, 0, 0, 0, 0, 0, 0, 0] }",
        );

        let elf = blaze_user_meta_elf {
            path: ptr::null_mut(),
            build_id_len: 0,
            build_id: ptr::null_mut(),
            reserved: [0u8; 8],
        };
        assert_eq!(
            format!("{elf:?}"),
            "blaze_user_meta_elf { path: 0x0, build_id_len: 0, build_id: 0x0, reserved: [0, 0, 0, 0, 0, 0, 0, 0] }",
        );

        let unknown = blaze_user_meta_unknown {
            reason: blaze_normalize_reason::BLAZE_NORMALIZE_REASON_UNMAPPED,
            reserved: [0u8; 7],
        };
        assert_eq!(
            format!("{unknown:?}"),
            "blaze_user_meta_unknown { reason: BLAZE_NORMALIZE_REASON_UNMAPPED, reserved: [0, 0, 0, 0, 0, 0, 0] }",
        );

        let meta = blaze_user_meta {
            kind: blaze_user_meta_kind::BLAZE_USER_META_UNKNOWN,
            variant: blaze_user_meta_variant {
                unknown: ManuallyDrop::new(blaze_user_meta_unknown {
                    reason: blaze_normalize_reason::BLAZE_NORMALIZE_REASON_UNMAPPED,
                    reserved: [0u8; 7],
                }),
            },
        };
        assert_eq!(
            format!("{meta:?}"),
            "blaze_user_meta { kind: BLAZE_USER_META_UNKNOWN, variant: blaze_user_meta_variant }",
        );

        let user_addrs = blaze_normalized_user_output {
            meta_cnt: 0,
            metas: ptr::null_mut(),
            output_cnt: 0,
            outputs: ptr::null_mut(),
            reserved: [0u8; 8],
        };
        assert_eq!(
            format!("{user_addrs:?}"),
            "blaze_normalized_user_output { meta_cnt: 0, metas: 0x0, output_cnt: 0, outputs: 0x0, reserved: [0, 0, 0, 0, 0, 0, 0, 0] }",
        );
    }

    /// Make sure that we can stringify normalization reasons as expected.
    #[tag(miri)]
    #[test]
    fn reason_stringification() {
        use blaze_normalize_reason::*;

        let data = [
            (Reason::Unmapped, BLAZE_NORMALIZE_REASON_UNMAPPED),
            (
                Reason::MissingComponent,
                BLAZE_NORMALIZE_REASON_MISSING_COMPONENT,
            ),
            (Reason::Unsupported, BLAZE_NORMALIZE_REASON_UNSUPPORTED),
        ];

        for (reason, expected) in data {
            assert_eq!(blaze_normalize_reason::from(reason), expected);
            let cstr = unsafe { CStr::from_ptr(blaze_normalize_reason_str(expected)) };
            let expected = CStr::from_bytes_with_nul(reason.as_bytes()).unwrap();
            assert_eq!(cstr, expected);
        }
    }

    /// Check that we can convert an [`Unknown`] into a
    /// [`blaze_user_meta_unknown`] and back.
    #[tag(miri)]
    #[test]
    fn unknown_conversion() {
        let unknown = Unknown {
            reason: Reason::Unsupported,
            _non_exhaustive: (),
        };

        let unknown_c = blaze_user_meta_unknown::from(unknown.clone());
        let () = ManuallyDrop::into_inner(unknown_c).free();

        let meta = UserMeta::Unknown(unknown);
        let meta_c = blaze_user_meta::from(meta);
        let () = unsafe { ManuallyDrop::into_inner(meta_c).free() };
    }

    /// Check that we can convert an [`Apk`] into a [`blaze_user_meta_apk`] and
    /// back.
    #[tag(miri)]
    #[test]
    fn apk_conversion() {
        let apk = Apk {
            path: PathBuf::from("/tmp/archive.apk"),
            _non_exhaustive: (),
        };

        let apk_c = blaze_user_meta_apk::from(apk.clone());
        let () = unsafe { ManuallyDrop::into_inner(apk_c).free() };

        let meta = UserMeta::Apk(apk);
        let meta_c = blaze_user_meta::from(meta);
        let () = unsafe { ManuallyDrop::into_inner(meta_c).free() };
    }

    /// Check that we can convert an [`Elf`] into a [`blaze_user_meta_elf`]
    /// and back.
    #[tag(miri)]
    #[test]
    fn elf_conversion() {
        let elf = Elf {
            path: PathBuf::from("/tmp/file.so"),
            build_id: Some(Cow::Borrowed(&[0x01, 0x02, 0x03, 0x04])),
            _non_exhaustive: (),
        };

        let elf_c = blaze_user_meta_elf::from(elf.clone());
        let () = unsafe { ManuallyDrop::into_inner(elf_c).free() };

        let meta = UserMeta::Elf(elf);
        let meta_c = blaze_user_meta::from(meta);
        let () = unsafe { ManuallyDrop::into_inner(meta_c).free() };
    }

    /// Make sure that we can create and free a normalizer instance.
    #[tag(miri)]
    #[test]
    fn normalizer_creation() {
        let normalizer = blaze_normalizer_new();
        let () = unsafe { blaze_normalizer_free(normalizer) };
    }

    /// Check that we can normalize user space addresses.
    #[test]
    fn normalize_user_addrs() {
        fn test(normalizer: *const blaze_normalizer) {
            let addrs = [
                0x0 as Addr,
                libc::__errno_location as Addr,
                libc::dlopen as Addr,
                libc::fopen as Addr,
                elf_conversion as Addr,
                normalize_user_addrs as Addr,
            ];

            let result = unsafe {
                blaze_normalize_user_addrs(normalizer, 0, addrs.as_slice().as_ptr(), addrs.len())
            };
            assert_ne!(result, ptr::null_mut());

            let user_addrs = unsafe { &*result };
            assert_eq!(user_addrs.meta_cnt, 3);
            assert_eq!(user_addrs.output_cnt, 6);

            let meta = unsafe { user_addrs.metas.read() };
            assert_eq!(meta.kind, blaze_user_meta_kind::BLAZE_USER_META_UNKNOWN);
            assert_eq!(
                unsafe { meta.variant.unknown.reason },
                blaze_normalize_reason::BLAZE_NORMALIZE_REASON_UNMAPPED
            );

            let () = unsafe { blaze_user_output_free(result) };
        }

        let normalizer = blaze_normalizer_new();
        assert_ne!(normalizer, ptr::null_mut());
        test(normalizer);
        let () = unsafe { blaze_normalizer_free(normalizer) };

        let opts = blaze_normalizer_opts {
            cache_maps: true,
            ..Default::default()
        };
        let normalizer = unsafe { blaze_normalizer_new_opts(&opts) };
        assert_ne!(normalizer, ptr::null_mut());
        test(normalizer);
        test(normalizer);
        let () = unsafe { blaze_normalizer_free(normalizer) };
    }

    /// Check that we can normalize sorted user space addresses.
    #[test]
    fn normalize_user_addrs_sorted() {
        let mut addrs = [
            libc::__errno_location as Addr,
            libc::dlopen as Addr,
            libc::fopen as Addr,
            elf_conversion as Addr,
            normalize_user_addrs as Addr,
        ];
        let () = addrs.sort();

        let normalizer = blaze_normalizer_new();
        assert_ne!(normalizer, ptr::null_mut());

        let opts = blaze_normalize_opts {
            sorted_addrs: true,
            ..Default::default()
        };
        let result = unsafe {
            blaze_normalize_user_addrs_opts(
                normalizer,
                0,
                addrs.as_slice().as_ptr(),
                addrs.len(),
                &opts,
            )
        };
        assert_ne!(result, ptr::null_mut());

        let user_addrs = unsafe { &*result };
        assert_eq!(user_addrs.meta_cnt, 2);
        assert_eq!(user_addrs.output_cnt, 5);

        let () = unsafe { blaze_user_output_free(result) };
        let () = unsafe { blaze_normalizer_free(normalizer) };
    }

    /// Check that we fail normalizing unsorted addresses with a function that
    /// requires them to be sorted.
    #[test]
    fn normalize_user_addrs_unsorted_failure() {
        let mut addrs = [
            libc::__errno_location as Addr,
            libc::dlopen as Addr,
            libc::fopen as Addr,
            elf_conversion as Addr,
            normalize_user_addrs as Addr,
        ];
        let () = addrs.sort_by(|addr1, addr2| addr1.cmp(addr2).reverse());

        let normalizer = blaze_normalizer_new();
        assert_ne!(normalizer, ptr::null_mut());

        let opts = blaze_normalize_opts {
            sorted_addrs: true,
            ..Default::default()
        };
        let result = unsafe {
            blaze_normalize_user_addrs_opts(
                normalizer,
                0,
                addrs.as_slice().as_ptr(),
                addrs.len(),
                &opts,
            )
        };
        assert_eq!(result, ptr::null_mut());
        assert_eq!(blaze_err_last(), blaze_err::BLAZE_ERR_INVALID_INPUT);

        let () = unsafe { blaze_normalizer_free(normalizer) };
    }

    /// Check that we can enable/disable the reading of build IDs.
    #[test]
    fn normalize_build_id_reading() {
        fn test(read_build_ids: bool) {
            let test_so = Path::new(&env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("data")
                .join("libtest-so.so")
                .canonicalize()
                .unwrap();
            let so_cstr = CString::new(test_so.clone().into_os_string().into_vec()).unwrap();
            let handle = unsafe { libc::dlopen(so_cstr.as_ptr(), libc::RTLD_NOW) };
            assert!(!handle.is_null());

            let the_answer_addr = unsafe { libc::dlsym(handle, "the_answer\0".as_ptr().cast()) };
            assert!(!the_answer_addr.is_null());

            let opts = blaze_normalizer_opts {
                build_ids: read_build_ids,
                ..Default::default()
            };

            let normalizer = unsafe { blaze_normalizer_new_opts(&opts) };
            assert!(!normalizer.is_null());

            let opts = blaze_normalize_opts {
                sorted_addrs: true,
                ..Default::default()
            };
            let addrs = [the_answer_addr as Addr];
            let result = unsafe {
                blaze_normalize_user_addrs_opts(
                    normalizer,
                    0,
                    addrs.as_slice().as_ptr(),
                    addrs.len(),
                    &opts,
                )
            };
            assert!(!result.is_null());

            let normalized = unsafe { &*result };
            assert_eq!(normalized.meta_cnt, 1);
            assert_eq!(normalized.output_cnt, 1);

            let rc = unsafe { libc::dlclose(handle) };
            assert_eq!(rc, 0, "{}", io::Error::last_os_error());

            let output = unsafe { &*normalized.outputs.add(0) };
            let meta = unsafe { &*normalized.metas.add(output.meta_idx) };
            assert_eq!(meta.kind, blaze_user_meta_kind::BLAZE_USER_META_ELF);

            let elf = unsafe { &meta.variant.elf };

            assert!(!elf.path.is_null());
            let path = unsafe { CStr::from_ptr(elf.path) };
            assert_eq!(path, so_cstr.as_ref());

            if read_build_ids {
                let expected = read_elf_build_id(&test_so).unwrap().unwrap();
                let build_id = unsafe { slice_from_user_array(elf.build_id, elf.build_id_len) };
                assert_eq!(build_id, expected.as_ref());
            } else {
                assert!(elf.build_id.is_null());
            }

            let () = unsafe { blaze_user_output_free(result) };
            let () = unsafe { blaze_normalizer_free(normalizer) };
        }

        test(true);
        test(false);
    }
}
