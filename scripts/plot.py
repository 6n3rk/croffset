#!/usr/bin/python3
import matplotlib.pyplot as pp
import numpy as np

def plot_rtts(flow, output_name, plot_loss):
    figure = pp.figure(figsize=(10, 6))
    yrange = np.array([-500, 1000])
    pp.ylim(yrange)
    # pp.plot(flow.brtts[:, 0], flow.brtts[:, 1], linewidth=0.1, label='BRTT')
    # pp.plot(flow.trtts[:, 0], flow.trtts[:, 1], linewidth=0.1, label='TRTT')
    # pp.plot(flow.rrtts[:, 0], flow.rrtts[:, 1], linewidth=0.1, label='RRTT')
    pp.scatter(flow.brtts[:, 0], flow.brtts[:, 1], s=1, label='BRTT')
    pp.scatter(flow.trtts[:, 0], flow.trtts[:, 1], s=1, label='TRTT')
    pp.scatter(flow.rrtts[:, 0], flow.rrtts[:, 1], s=1, label='RRTT')

    if plot_loss == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.02, color='red')
        for (ts_ns, sretrans) in flow.sretrans:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.01, color='blue')

    pp.legend(loc='upper right', fontsize=18)
    pp.savefig(f"{output_name}.png", dpi=300, bbox_inches='tight', pad_inches=0.05)

    figure = pp.figure(figsize=(10, 6))
    yrange = np.array([-100, 1000])
    pp.ylim(yrange)
    pp.scatter(flow.brtts[:, 0], flow.brtts[:, 1], s=1, label='BRTT')
    pp.axhline(y=0, color='r', linewidth=1, linestyle='-')

    if plot_loss == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.02, color='red')
        for (ts_ns, sretrans) in flow.sretrans:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.01, color='blue')

    pp.legend(loc='upper right', fontsize=18)
    pp.savefig(f"{output_name}_brtt.png", dpi=300, bbox_inches='tight', pad_inches=0.05)

    figure = pp.figure(figsize=(10, 6))
    yrange = np.array([-100, 1000])
    pp.ylim(yrange)
    pp.scatter(flow.trtts[:, 0], flow.trtts[:, 1], s=1, label='TRTT')
    pp.axhline(y=0, color='r', linewidth=1, linestyle='-')

    if plot_loss == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.02, color='red')
        for (ts_ns, sretrans) in flow.sretrans:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.01, color='blue')

    pp.legend(loc='upper right', fontsize=18)
    pp.savefig(f"{output_name}_trtt.png", dpi=300, bbox_inches='tight', pad_inches=0.05)

    figure = pp.figure(figsize=(10, 6))
    yrange = np.array([-100, 1000])
    pp.ylim(yrange)
    pp.scatter(flow.rrtts[:, 0], flow.rrtts[:, 1], s=1, label='RRTT')
    pp.axhline(y=0, color='r', linewidth=1, linestyle='-')

    if plot_loss == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.02, color='red')
        for (ts_ns, sretrans) in flow.sretrans:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.01, color='blue')

    pp.legend(loc='upper right', fontsize=18)
    pp.savefig(f"{output_name}_rrtts.png", dpi=300, bbox_inches='tight', pad_inches=0.05)

def plot_synced_offsets(flow, output_name, plot_retrans):
    # if plot_retrans == True:
    #     for (ts_ns, retrans) in flow.retrans:
    #         print(ts_ns + flow.init_ts, ts_ns, retrans.segsent)

    tcp_recv_ts = [t.tcp_recv - flow.init_ts for t in flow.synced_offsets]
    offsets = [t.offset for t in flow.synced_offsets]
    offsets_send = [t.offset_send for t in flow.synced_offsets]
    offsets_recv = [t.offset_recv for t in flow.synced_offsets]
    y_max = max(np.max(offsets), np.max(offsets_send), np.max(offsets_recv))
    y_min = min(np.min(offsets), np.min(offsets_send), np.min(offsets_recv))
    figure = pp.figure(figsize=(10, 6))
    yrange = np.array([y_min, y_max])
    pp.ylim(yrange)
    # pp.step(flow.offsets[:, 0], flow.offsets[:, 1], where='post', label='offset')
    pp.scatter(tcp_recv_ts, offsets, s=1, label='offset')

    if plot_retrans == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.02, color='red')
        for (ts_ns, sretrans) in flow.sretrans:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.01, color='blue')
            
    pp.axhline(y=0, color='r', linewidth=1, linestyle='-')

    pp.savefig(f"{output_name}.png", dpi=300, bbox_inches='tight', pad_inches=0.01)

    figure = pp.figure(figsize=(10, 6))
    yrange = np.array([y_min, y_max])
    pp.ylim(yrange)
    # pp.step(flow.offsets[:, 0], flow.offsets[:, 1], where='post', label='offset')
    pp.scatter(tcp_recv_ts, offsets_send, s=1, label='offset')

    if plot_retrans == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.02, color='red')
        for (ts_ns, sretrans) in flow.sretrans:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.01, color='blue')

    pp.axhline(y=0, color='r', linewidth=1, linestyle='-')

    pp.savefig(f"{output_name}_send.png", dpi=300, bbox_inches='tight', pad_inches=0.01)

    figure = pp.figure(figsize=(10, 6))
    yrange = np.array([y_min, y_max])
    pp.ylim(yrange)
    # pp.step(flow.offsets[:, 0], flow.offsets[:, 1], where='post', label='offset')
    pp.scatter(tcp_recv_ts, offsets_recv, s=1, label='offset')

    if plot_retrans == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.02, color='red')
        for (ts_ns, sretrans) in flow.sretrans:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.01, color='blue')

    pp.axhline(y=0, color='r', linewidth=1, linestyle='-')

    pp.savefig(f"{output_name}_receive.png", dpi=300, bbox_inches='tight', pad_inches=0.01)

def plot_offsets(flow, output_name, plot_retrans):
    figure = pp.figure(figsize=(10, 6))
    yrange = np.array([-500, 1500])
    pp.ylim(yrange)
    # pp.step(flow.offsets[:, 0], flow.offsets[:, 1], where='post', label='offset')
    pp.scatter(flow.offsets[:, 0], flow.offsets[:, 1], s=1, label='offset')

    if plot_retrans == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.02, color='red')
        for (ts_ns, sretrans) in flow.sretrans:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.01, color='blue')

    pp.savefig(f"{output_name}.png", dpi=300, bbox_inches='tight', pad_inches=0.01)

def plot_offsets2(flow, output_name, plot_retrans):
    figure = pp.figure(figsize=(10, 6))
    yrange = np.array([-500, 1500])
    pp.ylim(yrange)
    # pp.step(flow.offsets2[:, 0], flow.offsets2[:, 1], where='post', label='offset')
    pp.scatter(flow.offsets2[:, 0], flow.offsets2[:, 1], s=1, label='offset')

    if plot_retrans == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.02, color='red')
        for (ts_ns, sretrans) in flow.sretrans:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.01, color='blue')

    pp.savefig(f"{output_name}.png", dpi=300, bbox_inches='tight', pad_inches=0.01)

def plot_offsets3(flow, output_name, plot_retrans):
    figure = pp.figure(figsize=(10, 6))
    yrange = np.array([-500, 1500])
    pp.ylim(yrange)
    # pp.step(flow.offsets3[:, 0], flow.offsets3[:, 1], where='post', label='offset')
    pp.scatter(flow.offsets3[:, 0], flow.offsets3[:, 1], s=1, label='offset')

    if plot_retrans == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.02, color='red')
        for (ts_ns, sretrans) in flow.sretrans:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.01, color='blue')

    pp.savefig(f"{output_name}.png", dpi=300, bbox_inches='tight', pad_inches=0.01)

def plot_diff_offsets(flow, output_name, plot_retrans):
    x = []
    diff_offsets = []
    diff_send_offsets = []
    diff_recv_offsets = []
    for i in range(0, len(flow.synced_offsets) - 1):
        # if flow.synced_offsets[i][1] != 0 and  flow.synced_offsets[i][2] != 0 and flow.synced_offsets[i][3] != 0:
        #     x.append(flow.synced_offsets[i][0])
        #     diff_offsets.append((flow.synced_offsets[i + 1][1] - flow.synced_offsets[i][1]) / flow.synced_offsets[i][1]) 
        #     diff_send_offsets.append((flow.synced_offsets[i + 1][2] - flow.synced_offsets[i][2]) / flow.synced_offsets[i][2])
        #     diff_recv_offsets.append((flow.synced_offsets[i + 1][3] - flow.synced_offsets[i][3]) / flow.synced_offsets[i][3])
        
        x.append(flow.synced_offsets[i].tcp_recv - flow.init_ts)
        diff_offsets.append(flow.synced_offsets[i + 1].offset - flow.synced_offsets[i].offset) 
        diff_send_offsets.append(flow.synced_offsets[i + 1].offset_send - flow.synced_offsets[i].offset_send)
        diff_recv_offsets.append(flow.synced_offsets[i + 1].offset_recv - flow.synced_offsets[i].offset_recv)

    figure = pp.figure(figsize=(10, 6))
    # yrange = np.array([-500, 1500])
    # pp.ylim(yrange)
    # pp.step(flow.offsets3[:, 0], flow.offsets3[:, 1], where='post', label='offset')
    pp.scatter(x, diff_offsets, s=1, label='offset')

    if plot_retrans == True:
        for (ts_ns , loss) in flow.losses:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.02, color='red')
        for (ts_ns, sretrans) in flow.sretrans:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.01, color='blue')

    pp.savefig(f"{output_name}.png", dpi=300, bbox_inches='tight', pad_inches=0.01)

    figure = pp.figure(figsize=(10, 6))
    # yrange = np.array([-500, 1500])
    # pp.ylim(yrange)
    # pp.step(flow.offsets3[:, 0], flow.offsets3[:, 1], where='post', label='offset')
    pp.scatter(x, diff_send_offsets, s=1, label='offset')

    if plot_retrans == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.02, color='red')
        for (ts_ns, sretrans) in flow.sretrans:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.01, color='blue')

    pp.savefig(f"{output_name}_send.png", dpi=300, bbox_inches='tight', pad_inches=0.01)

    figure = pp.figure(figsize=(10, 6))
    # yrange = np.array([-500, 1500])
    # pp.ylim(yrange)
    # pp.step(flow.offsets3[:, 0], flow.offsets3[:, 1], where='post', label='offset')
    pp.scatter(x, diff_recv_offsets, s=1, label='offset')

    if plot_retrans == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.02, color='red')
        for (ts_ns, sretrans) in flow.sretrans:
            pp.axvline(x=ts_ns - flow.init_ts, ymin=0, ymax=0.01, color='blue')

    pp.savefig(f"{output_name}_recv.png", dpi=300, bbox_inches='tight', pad_inches=0.01)