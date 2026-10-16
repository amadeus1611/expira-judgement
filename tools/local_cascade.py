#!/usr/bin/env python3
"""Whole local cascade on VitaminC, exactly as `ej check` routes it: layer 1 (`ej check` output l1_{cal,test}.json)
decides first; on what it leaves unsure, a local judge's score (vitc_{split}.{judge}.scores) may call the claim stated
only past a threshold and, with --gate, only when the production wording gate passes (`ej-bench words` output
vitc_{split}.words: zero unstated claim words, every name in the quote, every figure backed). The threshold is fitted on calibration
(best 95% Wilson lower bound, at least 20 calls) and applied unchanged to test.

    python3 -I tools/local_cascade.py DATA_DIR JUDGE[+JUDGE2] [--gate] [--confirm]

JUDGE+JUDGE2 (e.g. large+xsmall): both judges must pass their own threshold; the pair of thresholds is fitted jointly
on calibration (best Wilson lower bound, at least 20 calls), over each judge's calibration score quantiles.
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


def read_scores(path):
    out = {}
    for line in open(path):
        s = json.loads(line)
        out[s["i"]] = s["p"]
    return out


def load(data, split, judge, gate):
    rows = [json.loads(l) for l in open(f"{data}/vitc_{split}.jsonl")]
    l1 = {int(x["claim_id"]): x["verdict"] for x in json.load(open(f"{data}/l1_{split}.json"))}
    # One score per row; with JUDGE+JUDGE2 a tuple, compared element-wise against a tuple of thresholds.
    parts = [read_scores(f"{data}/vitc_{split}.{j}.scores") for j in judge.split("+")]
    scores = {i: tuple(p.get(i, 0.0) for p in parts) for i in range(len(rows))}
    ok = [True] * len(rows)
    if gate:
        for line in open(f"{data}/vitc_{split}.words"):
            w = json.loads(line)
            ok[w["i"]] = w["unstated"] == 0 and w["names_ok"] and w.get("figures_ok", True)
    return rows, l1, scores, ok


def stated(split_data, t):
    """(right, called) for "stated" calls: layer-1 backed, or the judge past t where layer 1 is unsure and the gate passes."""
    rows, l1, scores, ok = split_data
    right = called = 0
    for i, r in enumerate(rows):
        v = l1[i]
        if v == "backed" or (v == "unsure" and ok[i] and all(p >= x for p, x in zip(scores[i], t))):
            called += 1
            right += r["label"] == "supports"
    return right, called


def main(a):
    data, judge, gate = a[1], a[2], "--gate" in a
    cal, test = load(data, "cal", judge, gate), load(data, "test", judge, gate)
    best = None
    k_judges = len(judge.split("+"))
    if k_judges == 1:
        grid = [(x,) for x in sorted({v[0] for v in cal[2].values()})]
    else:  # joint fit over 200 quantiles of each judge's calibration scores
        axes = []
        for j in range(k_judges):
            vs = sorted(v[j] for v in cal[2].values())
            axes.append(sorted({vs[min(len(vs) - 1, q * len(vs) // 200)] for q in range(200)}))
        grid = [(a, b) for a in axes[0] for b in axes[1]]
    for t in grid:
        k, n = stated(cal, t)
        if n >= 20 and (best is None or wilson(k, n) > best[0]):
            best = (wilson(k, n), t, k, n)
    lb, t, k, n = best
    tk, tn = stated(test, t)
    out = {"judge": judge, "gate": gate, "threshold": t[0] if len(t) == 1 else list(t), "cal": [k, n], "cal_lb": round(lb, 4),
           "test": [tk, tn], "test_accuracy": round(tk / tn, 4), "test_lb": round(wilson(tk, tn), 4)}
    if "--confirm" in a:  # the untouched set from fetch_general.py --confirm, scored once with the frozen threshold
        ck, cn = stated(load(data, "conf", judge, gate), t)
        out.update(confirm=[ck, cn], confirm_accuracy=round(ck / cn, 4), confirm_lb=round(wilson(ck, cn), 4))
    print(json.dumps(out))


if __name__ == "__main__":
    if len(sys.argv) < 3:
        raise SystemExit(__doc__)
    main(sys.argv)
