#!/usr/bin/env python3
"""Whole local cascade on VitaminC, exactly as `ej check` routes it: layer 1 (`ej check` output l1_{cal,test}.json)
decides first; on what it leaves unsure, a local judge's score (vitc_{split}.{judge}.scores) may call the claim stated
only past a threshold and, with --gate, only when the production wording gate passes (`ej-bench words` output
vitc_{split}.words: zero unstated claim words, every name in the quote, every figure backed). The threshold is fitted on calibration
(best 95% Wilson lower bound, at least 20 calls) and applied unchanged to test.

    python3 -I tools/local_cascade.py DATA_DIR JUDGE[+JUDGE2] [--gate] [--confirm] [--val] [--val2] [--fresh] [--block]

JUDGE+JUDGE2 (e.g. large+xsmall): both judges must pass their own threshold; the pair of thresholds is fitted jointly
on calibration (best Wilson lower bound, at least 20 calls), over each judge's calibration score quantiles.

--val also scores the second untouched set (fetch_general.py --validation, split "val") with the same frozen threshold.
--val2 does the same for the third set (fetch_general.py --validation2, split "val2"), and --fresh for the fourth
(fetch_general.py --fresh, split "fresh").
Only rows layer 1 leaves unsure and the gate passes need judge scores; a missing score counts as 0.

--block also lets the judges block: a row layer 1 leaves unsure, not called stated, whose every judge score is at or
below a block threshold is called not stated. The threshold is fitted on calibration (most blocks whose 95% Wilson
lower bound is at least 0.95, the standard's block bar, at least 20 blocks) and applied unchanged. A claim whose
figures layer 1 finds backed is never judge-blocked. Reports blocks
(layer 1 and judge) and coverage per set; only rows with every judge's score can be judge-blocked, so a set scored
only on its gate-passing rows reports judge blocks over those rows alone.
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
    scored = {i for i in range(len(rows)) if all(i in p for p in parts)}
    ok = [True] * len(rows)
    for line in open(f"{data}/vitc_{split}.words"):
        w = json.loads(line)
        if gate:
            ok[w["i"]] = w["unstated"] == 0 and w["names_ok"] and w.get("figures_ok", True)
        # A claim whose figures layer 1 finds in or met by the quote ("more than 95,000" against 98,491) is never
        # judge-blocked: the judges are weakest exactly at comparing figures.
        if w.get("figures_ok") and any(c.isdigit() for c in rows[w["i"]]["claim"]):
            scored.discard(w["i"])
    return rows, l1, scores, ok, scored


def stated(split_data, t):
    """(right, called) for "stated" calls: layer-1 backed, or the judge past t where layer 1 is unsure and the gate passes."""
    rows, l1, scores, ok, _ = split_data
    right = called = 0
    for i, r in enumerate(rows):
        v = l1[i]
        if v == "backed" or (v == "unsure" and ok[i] and all(p >= x for p, x in zip(scores[i], t))):
            called += 1
            right += r["label"] == "supports"
    return right, called


def blocks(split_data, t, b):
    """(right, called) for "not stated" calls: layer-1 mismatch, or with b, an unsure row not called stated whose
    every judge score is at most b."""
    rows, l1, scores, ok, scored = split_data
    right = called = 0
    for i, r in enumerate(rows):
        v = l1[i]
        is_stated = v == "unsure" and ok[i] and all(p >= x for p, x in zip(scores[i], t))
        judged = b is not None and v == "unsure" and not is_stated and i in scored and max(scores[i]) <= b
        if v == "mismatch" or judged:
            called += 1
            right += r["label"] != "supports"
    return right, called


def fit_block(cal, t):
    """Block threshold on calibration: the most judge blocks with Wilson lower bound >= 0.95 (at least 20)."""
    rows, l1, scores, ok, scored = cal
    cand = sorted(
        (max(scores[i]), rows[i]["label"] != "supports")
        for i in scored
        if l1[i] == "unsure" and not (ok[i] and all(p >= x for p, x in zip(scores[i], t)))
    )
    best, right = None, 0
    for k, (b, r) in enumerate(cand, 1):
        right += r
        if k >= 20 and wilson(right, k) >= 0.95 and (k == len(cand) or cand[k][0] > b):
            best = (b, right, k)
    return best


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
    if "--block" in a:
        fb = fit_block(cal, t)
        b = fb[0] if fb else None
        out.update(block_threshold=b, cal_judge_blocks=list(fb[1:]) if fb else None)
    if "--confirm" in a:  # the untouched set from fetch_general.py --confirm, scored once with the frozen threshold
        ck, cn = stated(load(data, "conf", judge, gate), t)
        out.update(confirm=[ck, cn], confirm_accuracy=round(ck / cn, 4), confirm_lb=round(wilson(ck, cn), 4))
    if "--val" in a:  # the second untouched set from fetch_general.py --validation, same frozen threshold
        vk, vn = stated(load(data, "val", judge, gate), t)
        out.update(val=[vk, vn], val_accuracy=round(vk / vn, 4), val_lb=round(wilson(vk, vn), 4))
    if "--val2" in a:  # the third untouched set from fetch_general.py --validation2, same frozen threshold
        wk, wn = stated(load(data, "val2", judge, gate), t)
        out.update(val2=[wk, wn], val2_accuracy=round(wk / wn, 4), val2_lb=round(wilson(wk, wn), 4))
    if "--fresh" in a:  # the fourth untouched set from fetch_general.py --fresh, same frozen threshold
        fk, fn = stated(load(data, "fresh", judge, gate), t)
        out.update(fresh=[fk, fn], fresh_accuracy=round(fk / fn, 4), fresh_lb=round(wilson(fk, fn), 4))
    if "--block" in a:
        for name in ["cal", "test", "conf", "val", "val2", "fresh"]:
            if name in ("cal", "test") or {"conf": "--confirm"}.get(name, "--" + name) in a:
                d = {"cal": cal, "test": test}.get(name) or load(data, name, judge, gate)
                br, bn = blocks(d, t, b)
                sk, sn = stated(d, t)
                out[f"{name}_blocks"] = [br, bn]
                out[f"{name}_block_lb"] = round(wilson(br, bn), 4)
                out[f"{name}_coverage"] = round((sn + bn) / len(d[0]), 4)
    print(json.dumps(out))


if __name__ == "__main__":
    if len(sys.argv) < 3:
        raise SystemExit(__doc__)
    main(sys.argv)
