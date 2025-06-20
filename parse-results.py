#!/usr/bin/env pypy3

import os, re, sys

d = {}

for fname in os.listdir(os.path.join(".", "results")):
    # print("%r" % fname)
    
    for l in open(os.path.join(".", "results", fname), "r").readlines():
        l = l.strip()
    
        mo = re.search("^( *[^: ]*).*?([0-9]+)$", l)

        if mo:
            # print("mo.group(1): %r, mo.group(2): %r" % (mo.group(1), mo.group(2),))

            fnname = mo.group(1)
            stddev = int(mo.group(2))

            # print("fn: %s, stddev: %s" % (fnname, stddev,))

            d.setdefault(fnname, []).append(stddev)
#         else:
#             print("no match %r" % l)

for (k, v) in d.items():
    lis = list(v)
    lis.sort()

    # print("--- k: %r, v: %r" % (k, v,))
    s = format(f"{k:21s}:")
    for v in lis:
        s += format(f" {v: 6d}")

    print(s)
       

