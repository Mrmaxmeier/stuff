#!/bin/env python3

from pprint import pprint
from collections import Counter, defaultdict
import sys

import pytoml

def printdepgraph(pkg, graph, ident=1):
    if ident == 1:
        print("-" + " " * (ident - 1), *pkg)
    else:
        if ident > 5:
            print(" " * ident, *pkg, "...")
            return
        print(" " * ident, *pkg)
    for _pkg in graph[pkg]:
        printdepgraph(_pkg, graph=graph, ident=ident+1)
crate_roots = sys.argv[1:]
if not crate_roots:
    crate_roots.append(".")
for crate_root in crate_roots:
    reversedepgraph = defaultdict(list)
    with open(crate_root + "/Cargo.lock", 'r') as f:
        data = pytoml.load(f)
    if 'package' not in data.keys():
        print("expected key 'package'")
        continue
    for p in data['package']:
        if 'dependencies' not in p:
            deps = []
        else:
            deps = p['dependencies']
        pkg = (p['name'], p['version'])
        for d in deps:
            d_ = tuple(d.split(" ")[:2])
            reversedepgraph[d_].append(pkg)
    # pprint(reversedepgraph)
    c = Counter([p['name'] for p in data['package']])
    for n, c in c.most_common():
        if c <= 1:
            break
        print(f"found {c} duplicates of `{n}`:")
        for pkg in data['package']:
            if pkg['name'] == n:
                printdepgraph((pkg['name'], pkg['version']), graph=reversedepgraph)
        # pprint([(p['name'], p['version']) for p in data['package'] if p['name'] == n])
