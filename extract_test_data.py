#!/usr/bin/python3
import sys
import re
import os
def element(arr, index):
    return arr[index] if index < len(arr) else None

def extract_log(file_in):
    app_bench_value = {}
    app_bench_value["Nginx"] = []
    app_bench_value["Memecached"] = []
    app_bench_value["iperf3"] = []
    app_bench_value["netperf"] = []
    app_bench_value['Hackbench'] = []
    app_bench_value['Untar'] = []
    app_bench_value["FileIO"] = []

    with open(file_in, 'r') as fin:
        for line in fin:
            arr = [x for x in re.sub('\s+', ' ', line).split(' ') if x]
            if (element(arr,0) == 'Requests'):
                app_bench_value["Nginx"].append(float(arr[3]))
            elif (element(arr,5) == 'TPS:'):
                app_bench_value["Memecached"].append(float(arr[-3]))
            elif (element(arr,8) == 'receiver'):
                app_bench_value["iperf3"].append(float(arr[-3]))
            elif (element(arr,1) == 'Trans/s'):
                app_bench_value["netperf"].append(float(arr[2]))
                app_bench_value["netperf"].append(float(arr[3]))
                app_bench_value["netperf"].append(float(arr[4]))
                app_bench_value["netperf"].append(float(arr[0]))
            elif (element(arr,0) == "Time:"):
                app_bench_value['Hackbench'].append(float(arr[1]))
            elif (element(arr,0) == 'real'):
                app_bench_value['Untar'].append(float(arr[1][2:-1]))
            elif (element(arr,5) == 'transferred'):
                app_bench_value["FileIO"].append(float(arr[-1][1:-7]))
    return app_bench_value

app_bench_value = extract_log(sys.argv[1]);

for app,vals in app_bench_value.items():
    print(app)
    for v in vals:
        print(v)
    print()
