#!/usr/bin/python3
import subprocess
import tempfile
import os
import numpy as np
import signal
import sys
import time
import re
import argparse

def main():
    num_flows = int(args.flow)
    time = int(args.time)
    app = args.app
    interface = "ens801f0"
    cca = "bbr"

    global experiment
    global iwd
    iwd = os.getcwd()

    initialize_nic()

    if args.vxlan:
        experiment = f"rx-e-f{num_flows}-t{time}-{cca}-{app}-0"
    elif args.native:
        experiment = f"rx-n-f{num_flows}-t{time}-{cca}-{app}-0"
    else:
        experiment = f"rx-h-f{num_flows}-t{time}-{cca}-{app}-0"

    if not os.path.exists(os.path.join(iwd, '..', 'data')):
        os.mkdir(os.path.join(iwd, '..', 'data'))
    os.chdir(os.path.join(iwd, '..', 'data'))

    while os.path.exists(os.path.join(iwd, '..', 'data', experiment)):
        (remained, last) = experiment.rsplit("-", 1)
        trial = int(last) + 1
        experiment = f"{remained}-{trial}"

    os.mkdir(os.path.join(iwd, '..', 'data', experiment))
    os.chdir(os.path.join(iwd, '..', 'data', experiment))

    if args.app == "neper":
        neper_processes = start_neper_servers(num_flows)

    (util_p, util_f) = start_cpu_utilization()
    interrupt_f = start_interrupt_count()
    (epping_p, epping_f) = start_epping(interface)

    print("Press \'Enter\' to end the recording.")
    line = sys.stdin.readline()

    end_cpu_utilization(util_p, util_f)
    post_process_interrupt_count(interrupt_f)
    epping_map = end_client_epping(epping_p, epping_f, num_flows)

    if args.app == "neper":
        end_neper_servers(neper_processes)

def initialize_nic():
    print("Initialize ice driver.", end=" ", flush=True)
    subprocess.run(["rmmod", "ice"])
    time.sleep(1)
    subprocess.run(["modprobe", "ice"])
    time.sleep(1)
    subprocess.run([os.path.join(iwd, 'flow_direction_rx_tcp.sh')], stdout=subprocess.DEVNULL)
    subprocess.run([os.path.join(iwd, 'smp_affinity.sh')], stdout=subprocess.DEVNULL)
    print("Done.")
    
def start_neper_servers(num_flows):
    neper_processes = []
    for i in range(0, num_flows):
        port = 5300 + i
        cpu = 16 + i
        neper_args = ["numactl", "-C", str(cpu), os.path.join(iwd, 'tcp_rr'), "--nolog", "-P", str(port), "-l", args.time]
        f = tempfile.TemporaryFile()
        p = subprocess.Popen(neper_args, stdout=f)
        neper_processes.append(p)

    return neper_processes

def end_neper_servers(neper_processes):
    for process in neper_processes:
        os.kill(process.pid, signal.SIGTERM)

def start_epping(interface):
    with open(f'raw.epping.{experiment}.out', 'w') as f:
        if args.vxlan:
            p = subprocess.Popen(["./pping", "-i", interface, "-x", "native", "-V"], stdout=f, cwd=os.path.join(iwd, '..'))
        else:
            p = subprocess.Popen(["./pping", "-i", interface, "-x", "native"], stdout=f, cwd=os.path.join(iwd, '..'))

    time.sleep(2)
    return (p, f)

def end_client_epping(epping_p, epping_f, num_flows):
    epping_p.kill()

    epping_map = {}

    dports = [5200, 5201, 5202, 5203, 5204, 5205, 5206, 5207]

    with open(epping_f.name, 'r') as epping_f:
        for i in range(0, num_flows):
            epping_f.seek(0)
            if args.vxlan:
                print(f"{i}: UDP 192.168.2.102:*+192.168.2.103:{str(dports[i])}")
                target = "192.168.2.103" + ":" + str(dports[i])
                expr = re.compile(r"^(\d{2}:\d{2}:\d{2}\.\d{9})\s(.+?)\sms\s(.+?)\sms\s" + r"UDP\s192.168.2.102:.*\+" + re.escape(target) + r"$")
                samples = parse(epping_f, expr)
            # elif args.native:
            #     print(f"{i}: TCP {str(flow.daddr)}:{str(flow.dport)}+{str(flow.saddr)}:{str(flow.sport)}")
            #     target = str(flow.daddr) + ":" + str(flow.dport) + "+" + str(flow.saddr) + ":" + str(flow.sport)
            #     expr = re.compile(r"^(\d{2}:\d{2}:\d{2}\.\d{9})\s(.+?)\sms\s(.+?)\sms\s" + r"TCP\s" + re.escape(target) + r"$")
            #     samples = parse(epping_f, expr)
            else:
                print(f"{i}: TCP 192.168.2.102:*+192.168.2.103:{str(dports[i])}")
                target = "192.168.2.103" + ":" + str(dports[i])
                expr = re.compile(r"^(\d{2}:\d{2}:\d{2}\.\d{9})\s(.+?)\sms\s(.+?)\sms\s" + r"TCP\s192.168.2.102:.*\+" + re.escape(target) + r"$")
                samples = parse(epping_f, expr)

            if len(samples) == 0:
                print("There is no epping samples.")
                return None

            x = list(zip(*samples))[0]
            y = list(zip(*samples))[1]

            epping_map[i] = (x, y)

            with open(f'epping.{i}.{experiment}.out', 'w') as epping_output_per_flow:
                for j, _ in enumerate(epping_map[i][0]):
                    epping_output_per_flow.write(f'{epping_map[i][0][j].astype(int)},{epping_map[i][1][j]}\n')

    return epping_map

def start_cpu_utilization():
    f = tempfile.NamedTemporaryFile()
    p = subprocess.Popen([os.path.join(iwd, 'cpuload.sh')], stdout=f)

    return (p, f)

def end_cpu_utilization(p, f):
    p.kill()
    with open(f'cpu.{experiment}.out', 'w') as cpu_output:
        subprocess.run([os.path.join(iwd, 'cpu.py'), f.name], stdout=cpu_output)

    if not args.silent:
        subprocess.run([os.path.join(iwd, 'cpu.py'), "-c", f.name])
    
def start_interrupt_count():
    f = tempfile.NamedTemporaryFile()
    subprocess.run(["cat", "/proc/interrupts"], stdout=f)

    return f

def post_process_interrupt_count(old_f):
    new_f = tempfile.NamedTemporaryFile()
    subprocess.run(["cat", "/proc/interrupts"], stdout=new_f)

    with open(f'int.{experiment}.out', 'w') as interrupt_output:
        subprocess.run([os.path.join(iwd, 'interrupts.py'), old_f.name, new_f.name], stdout=interrupt_output)

    if not args.silent:
        subprocess.run([os.path.join(iwd, 'interrupts.py'), old_f.name, new_f.name])

def str_to_ns(time_str):
    h, m, s = time_str.split(":")
    int_s, ns = s.split(".")
    ns = map(lambda t, unit: np.timedelta64(t, unit), [h,m,int_s,ns.ljust(9, '0')],['h','m','s','ns'])
    return sum(ns)

def parse(f, expr):
    samples = []
    initial_timestamp_ns = 0
    lines = f.readlines()
    for l in lines:
        match = expr.search(l)
        if match == None:
            continue
        
        timestamp_ns = str_to_ns(match.group(1))

        if initial_timestamp_ns == 0:
            initial_timestamp_ns = timestamp_ns
            
        rtt_us = float(match.group(2)) * 1000
        samples.append(((timestamp_ns - initial_timestamp_ns), rtt_us))

    return np.array(samples)

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument('--flow', '-f', default=6)
    parser.add_argument('--time', '-t', default=60)
    parser.add_argument('--app', '-a', default='iperf')
    parser.add_argument('--silent', '-s', action='store_true')
    parser.add_argument('--vxlan', '-v', action='store_true')
    parser.add_argument('--native', '-n', action='store_true')

    global args
    args = parser.parse_args()

    if args.app != "iperf" and args.app != "neper":
        print("Wrong app name.")
        exit()

    main()
    