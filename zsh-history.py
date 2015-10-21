#!/usr/bin/python3

import os
import time
from collections import defaultdict, Counter
from pprint import pprint

history = []
by_base = defaultdict(list)

with open(os.path.expanduser("~") + "/.zsh_history", "rb") as f:
	for line in f.readlines():
		if not line.startswith(b": ") or not b";" in line:
			#print("skipping", line)
			continue
		data = line.split(b";", 1)
		if not len(data) == 2:
			#print("skipping", line)
			continue
		timestamp, command = int(data[0][2:-2]), data[1]
		#print(timestamp, command)
		history.append((timestamp, command))
		base = command.decode("utf-8", "ignore").split(" ")[0]
		base = base.rstrip("\n").rstrip("&")
		by_base[base].append(command)

print()
print("total:", len(history))
dups = len(set([c for t, c in history]))
print("duplicates: {} ({:.0%})".format(dups, dups / len(history)))
print("unique: {} ({:.0%})".format(len(history) - dups, (len(history) - dups) / len(history)))

print("\ntop 20 - base")
top = []
for key in sorted(by_base.keys(), key=lambda d: len(by_base[d])):
	top.append("{}: {}".format(len(by_base[key]), key))
print("\n".join(top[-20:]))

print("\ntop 20 - full command")
top = []
counter = Counter()
for t, c in history:
	c = c.decode("utf-8", "ignore")
	c = c.rstrip("\n")
	counter[c] += 1
for key in sorted(counter.keys(), key=lambda d: counter[d]):
	top.append("{}: {}".format(counter[key], key))
print("\n".join(top[-20:]))


def trends(toplen, seconds, pertext):
	total = {}
	for key, value in by_base.items():
		total[key] = len(value) / len(history)
	now = time.time()
	filtered = [v for v in history if (now - v[0]) < seconds]
	filteredCounter = Counter()
	for t, c in filtered:
		c = c.decode("utf-8", "ignore").split(" ")[0]
		c = c.rstrip("\n").rstrip("&")
		filteredCounter[c] += 1
	print("\ntop " + str(toplen) + " - trends - " + pertext + " - total: " + str(len(filtered)))
	top = []
	for key, value in filteredCounter.items():
		new = value / len(filtered)
		diff = new - total[key]
		top.append((diff, "[{}{:.0%}] {}".format("+"*(diff > 0), diff, key)))
	#print("\n".join([s for d, s in top]))
	top = [s for d, s in sorted(top, key=lambda d: abs(d[0]))]
	print("\n".join(top[-toplen:]))

trends(toplen=10, seconds=60*60*24*30, pertext="last month")
trends(toplen=10, seconds=60*60*24*7, pertext="last week")
trends(toplen=10, seconds=60*60*24, pertext="today")
