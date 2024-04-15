#!/usr/bin/python3
import os
import json

def parse_dataset(path):
    extype_map = {}
    for experiment in os.listdir(path):
        if os.path.isdir(os.path.join(path, experiment)):
            summary_json = f"summary.{experiment}.json"
            with open(os.path.join(path, experiment, summary_json), 'r') as file:
                summary = json.load(file)

                hflows = []
                hflow_index = 0
                while (f"h{hflow_index}" in summary):
                    flow = summary.get(f"h{hflow_index}")
                    throughput = flow.get("throughput")
                    retransmissions = flow.get("retransmissions")
                    sr_count = flow.get("sr_count")
                    hflows.append((throughput, retransmissions, sr_count))
                    hflow_index += 1
                
                cflows = []
                cflow_index = 0
                while (f"c{cflow_index}" in summary):
                    flow = summary.get(f"c{cflow_index}")
                    throughput = flow.get("throughput")
                    retransmissions = flow.get("retransmissions")
                    sr_count = flow.get("sr_count")
                    cflows.append((throughput, retransmissions, sr_count))
                    cflow_index += 1
                
                extype = f"h{len(hflows)}-c{len(cflows)}"
                if not extype in extype_map:
                    extype_map[extype] = [(hflows, cflows)]
                else:
                    extype_map[extype].append((hflows, cflows))

    return extype_map