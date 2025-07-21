#!/usr/bin/env pypy3

import os, re, sys

# k: fn-plus-clock, v: list of (stddev, winner?)
d = {}

# print("sys.argv[1:]: %r" % sys.argv[1:])

# for fname in os.listdir(os.path.join(".", "results")):
for fname in sys.argv[1:]:
    # print("f: %r" % fname)
    
    winning = None
    for l in open(fname, "r").readlines():
        l = l.strip()
        # print("l: %r" % l)
    
        mo = re.search("^ *([a-z0-9_:]+) +?([A-Za-z_]+).+?([0-9,]+) +(---|[0-9.]+)$", l)

        if mo:
            # print("mo.group(1): %r, mo.group(2): %r, mo.group(3): %r" % (mo.group(1), mo.group(2), mo.group(3)))

            fnname = mo.group(1)
            clockname = mo.group(2)
            stddev = int(mo.group(3).replace(r",", ""))

            # print("fn: %s, clockname: %s, stddev: %s" % (fnname, clockname, stddev,))

            fnplusclock = fnname + "-" + clockname

            lstddevs = d.get(fnplusclock, [])
            lstddevs.append((stddev, "  "))
            if winning is None or stddev < winning:
                winning = stddev

            d[fnplusclock] = lstddevs
        # else:
        #     print("no match %r" % l)

    # Okay now run back over the new set of stddev results and tag all the winners (there should be
    # at least one)
    for k, v in d.items():
        if v[-1][0] == winning:
            v[-1] = (v[-1][0], " *")

for (k, v) in d.items():
    lis = list(v)

    winners = 0
    # print("--- k: %r, v: %r" % (k, v,))
    s = ""
    for v in lis:
        s += format(f" {v[0]:6d}{v[1]}")
        if v[1] == " *":
            winners += 1

    s = format(f"{k:48s}: #winners: {winners:3d}; ") + s
    print(s)
       

