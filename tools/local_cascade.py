#!/usr/bin/env python3
"""Whole local cascade on VitaminC, exactly as `ej check` routes it: layer 1 (`ej check` output l1_{cal,test}.json)
decides first; on what it leaves unsure, a local judge's score (vitc_{split}.{judge}.scores) may call the claim stated
only past a threshold and, with --gate, only when the production wording gate passes (`ej-bench words` output
vitc_{split}.words: zero unstated claim words and every name in the quote). The threshold is fitted on calibration
(best 95% Wilson lower bound, at least 20 calls) and applied unchanged to test.

    python3 -I tools/local_cascade.py DATA_DIR JUDGE [--gate]
"""
import json
import math
import sys


def wilson(k, n, z=1.96):
    if n == 0:
        return 0.0
    p = k / n
    d = 1 + z * z / n
    return (p + z * z / (2 * n) - z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n))) / d


def load(data, split, judge, gate):
    rows = [json.loads(l) for l in open(f"{data}/vitc_{split}.jsonl")]
    l1 = {int(x["claim_id"]): x["verdict"] for x in json.load(open(f"{data}/l1_{split}.json"))}
    scores = {}
    for line in open(f"{data}/vitc_{split}.{judge}.scores"):
        s = json.loads(line)
        scores[s["i"]] = s["p"]
    ok = [True] * len(rows)
    if gate:
        for line in open(f"{data}/vitc_{split}.words"):
            w = json.loads(line)
            ok[w["i"]] = w["unstated"] == 0 and w["names_ok"]
    return rows, l1, scores, ok


def stated(split_data, t):
    """(right, called) for "stated" calls: layer-1 backed, or the judge past t where layer 1 is unsure and the gate passes."""
    rows, l1, scores, ok = split_data
    right = called = 0
    for i, r in enumerate(rows):
        v = l1[i]
        if v == "backed" or (v == "unsure" and ok[i] and scores.get(i, 0.0) >= t):
            called += 1
            right += r["label"] == "supports"
    return right, called


def main(a):
    data, judge, gate = a[1], a[2], "--gate" in a
    cal, test = load(data, "cal", judge, gate), load(data, "test", judge, gate)
    best = None
    for t in sorted(set(cal[2].values())):
        k, n = stated(cal, t)
        if n >= 20 and (best is None or wilson(k, n) > best[0]):
            best = (wilson(k, n), t, k, n)
    lb, t, k, n = best
    tk, tn = stated(test, t)
    print(json.dumps({"judge": judge, "gate": gate, "threshold": t, "cal": [k, n], "cal_lb": round(lb, 4),
                      "test": [tk, tn], "test_accuracy": round(tk / tn, 4), "test_lb": round(wilson(tk, tn), 4)}))


if __name__ == "__main__":
    if len(sys.argv) < 3:
        raise SystemExit(__doc__)
    main(sys.argv)
