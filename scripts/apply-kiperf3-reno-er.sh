#!/bin/bash

cat iperf3-reno-er.yaml | sed -s "s/{CROFFSET_CLIENT}/$2/g; s/{CROFFSET_SERVER}/$3/g" | kubectl --kubeconfig /home/$1/.kube/config apply -f -