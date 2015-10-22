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
	def filtered(cond):
		counter = Counter()
		filtered_history = [v for v in history if cond(v)]
		for t, c in filtered_history:
			c = c.decode("utf-8", "ignore").split(" ")[0]
			c = c.rstrip("\n").rstrip("&")
			counter[c] += 1
		return len(filtered_history), {k: v / len(filtered_history) for k, v in counter.items()}

	now = time.time()
	past_len, past = filtered(lambda v: (now - v[0]) >= seconds)
	present_len, present = filtered(lambda v: (now - v[0]) < seconds)
	print("\ntop {} - trends - {} - total: {} ({:.0%})".format(toplen, pertext, present_len, present_len / len(history)))
	top = []
	for key in set(list(present.keys()) + list(past.keys())):
		past_abs = past.get(key, 0) * past_len
		present_abs = present.get(key, 0) * present_len
		s = "{} ({} -> {})".format(key, int(past_abs), int(present_abs))
		if key in past and key in present:
			diff = present[key] / past[key]
			if diff > 1:
				top.append((diff, "[+{:.0%}] {}".format(min(diff - 1, 9.99), s)))
			else:
				top.append((1 / diff, "[ -{:.0%}] {}".format(1 - diff, s)))
		elif key in present:
			top.append((present[key] * 100, "[ new ] {}".format(s)))
			#print(top[-1])
		else:
			top.append((past[key] * 100, "[ drp ] {}".format(s)))
			#print(top[-1])
	top = [s for d, s in sorted(top, key=lambda d: abs(d[0]))]
	print("\n".join(top[-toplen:]))

trends(toplen=10, seconds=60*60*24*365, pertext="past year")
trends(toplen=10, seconds=60*60*24*30, pertext="past month")
trends(toplen=10, seconds=60*60*24*7, pertext="past week")

from fuzzywuzzy import fuzz
import sh

print("\ntop 10 - common typos")
all_commands = set()
for line in sh.zsh("-i", c="alias"):
	all_commands.add(line.split("=", 1)[0])
for path in os.getenv("PATH").split(":"):
	if os.path.isdir(path):
		all_commands.update(os.listdir(path))
	else:
		print("[WARNING] in path but not folder:", path)

top = []
for key, value in by_base.items():
	if len(value) < 2 or any([key.startswith(v) for v in ["/", "./", "~/"]]):
		continue
	if key not in all_commands:
		for k2, v2 in by_base.items():
			if key == k2:
				continue
			if fuzz.ratio(key, k2) < 85:
				continue
			if k2 in all_commands:
				top.append((len(value), "{}: {} (-> {})".format(len(value), key, k2)))
				break

top = [s for d, s in sorted(top, key=lambda d: d[0])]
print("\n".join(top[-10:]))
