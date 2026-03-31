# dumpster, a cycle-tracking garbage collector for Rust.
# Copyright (C) 2023 Clayton Ramsey.

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import matplotlib.pyplot as plt
import numpy as np
import sys
import os

csv_file = open(sys.argv[1])

multi_times = {}
single_times = {}

for line in csv_file.read().split('\n'):
    if len(line) == 0:
        continue
    name, test_type, n_threads, n_ops, time = line.split(',')
    times = single_times if test_type == 'single_threaded' else multi_times
    if name not in times.keys():
        times[name] = ([], [])
    times[name][0].append(int(n_threads))
    times[name][1].append(float(time) / 1000.0)

def violin(times: dict, name: str):
    data = []
    labels = []

    for (label, (_, ys)) in times.items():
        data.append(ys)
        labels.append(label)

    def remove_outliers(data):
        percentile = 1.0
        low, high = np.percentile(data, [percentile, 100 - percentile])
        data = np.array(data)
        return data[(data >= low) & (data <= high)]

    # data = list(map(remove_outliers, data))

    plt.figure(figsize=[6, 3])
    plt.violinplot(data, range(len(data)), vert=False)
    plt.yticks(range(len(data)), labels=labels)
    plt.ylabel('Hasher')
    plt.xlabel('Runtime for 1M ops (ms)')
    plt.tight_layout(rect=(0, 0, 1, 0.95))
    plt.title(name)

    filename = name.replace("/", "_")
    path = f"target/plot/{filename}.png"
    os.makedirs(os.path.dirname(path), exist_ok=True)
    plt.savefig(path, dpi=300)

topics = {}

for name, xy in single_times.items():
    topic, name = name.split(': ')

    if topic not in topics.keys():
        topics[topic] = {}

    topics[topic][name] = xy

for topic_name, topic in topics.items():
    topic = dict(reversed(topic.items()))
    violin(topic, topic_name)