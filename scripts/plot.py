#!/usr/bin/python3
import matplotlib.pyplot as pp
import numpy as np

def plot_rtts(flow, output_name, plot_loss):
    figure = pp.figure(figsize=(10, 6))
    yrange = np.array([0, 500])
    pp.ylim(yrange)
    pp.plot(flow.brtts[:, 0], flow.brtts[:, 1], linewidth=0.1, label='BRTT')
    pp.plot(flow.trtts[:, 0], flow.trtts[:, 1], linewidth=0.1, label='TRTT')
    pp.plot(flow.rrtts[:, 0], flow.rrtts[:, 1], linewidth=0.1, label='RRTT')

    if plot_loss == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns, ymin=0, ymax=0.03, color='red')

    pp.legend(loc='upper right', fontsize=18)
    pp.savefig(output_name, dpi=300, bbox_inches='tight', pad_inches=0.05)

def plot_offsets(flow, output_name, plot_loss):
    figure = pp.figure(figsize=(10, 6))
    yrange = np.array([-500, 1500])
    pp.ylim(yrange)
    pp.step(flow.offsets[:, 0], flow.offsets[:, 1], where='post', label='offset')

    if plot_loss == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns, ymin=0, ymax=0.03, color='red')

    pp.savefig(output_name, dpi=300, bbox_inches='tight', pad_inches=0.01)

def plot_offsets2(flow, output_name, plot_loss):
    figure = pp.figure(figsize=(10, 6))
    yrange = np.array([-500, 1500])
    pp.ylim(yrange)
    pp.step(flow.offsets2[:, 0], flow.offsets2[:, 1], where='post', label='offset')

    if plot_loss == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns, ymin=0, ymax=0.03, color='red')

    pp.savefig(output_name, dpi=300, bbox_inches='tight', pad_inches=0.01)

def plot_offsets3(flow, output_name, plot_loss):
    figure = pp.figure(figsize=(10, 6))
    yrange = np.array([-500, 1500])
    pp.ylim(yrange)
    pp.step(flow.offsets3[:, 0], flow.offsets3[:, 1], where='post', label='offset')

    if plot_loss == True:
        for (ts_ns, loss) in flow.losses:
            pp.axvline(x=ts_ns, ymin=0, ymax=0.03, color='red')

    pp.savefig(output_name, dpi=300, bbox_inches='tight', pad_inches=0.01)