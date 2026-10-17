#!/usr/bin/env python3
"""Rebuild the frozen general (web-text) calibration and test splits used by ej-bench, without storing the data in the repo.

    python3 -I tools/fetch_general.py OUT_DIR [--pages 60]
    python3 -I tools/fetch_general.py OUT_DIR --confirm [--pages 120]
    python3 -I tools/fetch_general.py OUT_DIR --validation [--pages 300]
    python3 -I tools/fetch_general.py OUT_DIR --validation2 [--pages 300]

--confirm builds an untouched confirmation set instead: pages at the midpoints between the 60 frozen offsets (and,
with more pages, further evenly spaced offsets that avoid every frozen page by at least 100 rows), all rows kept, written
to OUT_DIR/vitc_confirm.jsonl, minus rows whose exact claim and quote appear in the frozen splits (which must already
be in OUT_DIR). It is never used to fit anything; it only confirms a configuration fitted on calibration.

--validation builds a second untouched set from the VitaminC validation split, which no earlier split draws from:
evenly spaced pages, all rows kept, written to OUT_DIR/vitc_val.jsonl, minus rows whose exact claim and quote appear in
the cal, test or confirmation sets.

--validation2 builds a third untouched set for checks written after reading the first: the pages halfway between the
--validation pages (offset +105, so no row overlaps), written to OUT_DIR/vitc_val2.jsonl, minus rows in any earlier set.

Source: VitaminC test split (tals/vitaminc on Hugging Face, CC BY-SA 3.0), fetched through the public datasets-server API
in fixed pages at evenly spaced offsets, so every run gets the same rows. Labels: SUPPORTS -> supports, REFUTES ->
contradicts, NOT ENOUGH INFO -> unsure. Rows are split into calibration and test by a hash of `case_id`, never by row:
VitaminC's contrastive pairs share a case, and a pair split across both halves would leak the answer into the test.

Writes OUT_DIR/vitc_cal.jsonl and OUT_DIR/vitc_test.jsonl ({"claim","quote","label"} per line) and prints the counts.
Standard library only."""
import hashlib, json, os, sys, time, urllib.request

API = "https://datasets-server.huggingface.co/rows?dataset=tals/vitaminc&config=default&split={}&offset={}&length=100"
TOTAL = 55197  # rows in the VitaminC test split (datasets-server num_rows_total, 2026-10-11)
VAL_TOTAL = 63054  # rows in the VitaminC validation split (datasets-server /size, 2026-10-17)
LABELS = {"SUPPORTS": "supports", "REFUTES": "contradicts", "NOT ENOUGH INFO": "unsure"}


def fetch(offset, split="test"):
    for attempt in range(5):
        try:
            with urllib.request.urlopen(API.format(split, offset), timeout=60) as r:
                return json.load(r)["rows"]
        except Exception:  # transient network or rate limit: back off and retry
            time.sleep(2 ** attempt)
    raise SystemExit(f"could not fetch offset {offset}")


def main(argv):
    if len(argv) < 2:
        raise SystemExit(__doc__)
    out, pages = argv[1], int(argv[argv.index("--pages") + 1]) if "--pages" in argv else 60
    os.makedirs(out, exist_ok=True)
    if "--confirm" in argv:
        return confirm(out, pages)
    if "--validation2" in argv:
        return validation(out, pages, shift=105, name="val2")
    if "--validation" in argv:
        return validation(out, pages)
    step = (TOTAL - 100) // (pages - 1)
    halves = {"cal": [], "test": []}
    for k in range(pages):
        for r in fetch(k * step):
            row = r["row"]
            label = LABELS.get(row["label"])
            if not label or not row["claim"].strip() or not row["evidence"].strip():
                continue
            half = "cal" if hashlib.sha256(row["case_id"].encode()).digest()[0] % 2 == 0 else "test"
            halves[half].append({"claim": row["claim"], "quote": row["evidence"], "label": label})
    for half, rows in halves.items():
        with open(os.path.join(out, f"vitc_{half}.jsonl"), "w") as f:
            for row in rows:
                f.write(json.dumps(row, ensure_ascii=False) + "\n")
        counts = {l: sum(1 for x in rows if x["label"] == l) for l in LABELS.values()}
        print(half, len(rows), counts)


def confirm(out, pages):
    frozen = [k * ((TOTAL - 100) // 59) for k in range(60)]
    step = (TOTAL - 100) // (pages - 1)
    offsets = [o for o in (k * step for k in range(pages)) if all(abs(o - f) >= 200 for f in frozen)]
    rows = []
    for o in offsets:
        for r in fetch(o):
            row = r["row"]
            label = LABELS.get(row["label"])
            if label and row["claim"].strip() and row["evidence"].strip():
                rows.append({"claim": row["claim"], "quote": row["evidence"], "label": label})
    # Drop rows whose exact (claim, quote) text is in a frozen split (7 rows in the 200-page set), so the set stays untouched.
    frozen_pairs = set()
    for half in ("cal", "test"):
        path = os.path.join(out, f"vitc_{half}.jsonl")
        if not os.path.exists(path):
            raise SystemExit(f"--confirm needs the frozen splits in {out}; run without --confirm first")
        for line in open(path):
            r = json.loads(line)
            frozen_pairs.add((r["claim"], r["quote"]))
    rows = [r for r in rows if (r["claim"], r["quote"]) not in frozen_pairs]
    with open(os.path.join(out, "vitc_confirm.jsonl"), "w") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")
    print("confirm", len(offsets), "pages", len(rows), {l: sum(1 for x in rows if x["label"] == l) for l in LABELS.values()})


def labelled(raw):
    rows = []
    for r in raw:
        row = r["row"]
        label = LABELS.get(row["label"])
        if label and row["claim"].strip() and row["evidence"].strip():
            rows.append({"claim": row["claim"], "quote": row["evidence"], "label": label})
    return rows


def validation(out, pages, shift=0, name="val"):
    step = (VAL_TOTAL - 100) // (pages - 1)
    rows = [x for k in range(pages) for x in labelled(fetch(min(k * step + shift, VAL_TOTAL - 100), "validation"))]
    seen = set()
    for earlier in ("cal", "test", "confirm") + (("val",) if name != "val" else ()):
        path = os.path.join(out, f"vitc_{earlier}.jsonl")
        if not os.path.exists(path):
            raise SystemExit(f"--validation needs {path}; build the other sets first")
        for line in open(path):
            r = json.loads(line)
            seen.add((r["claim"], r["quote"]))
    dropped = sum(1 for r in rows if (r["claim"], r["quote"]) in seen)
    rows = [r for r in rows if (r["claim"], r["quote"]) not in seen]
    with open(os.path.join(out, f"vitc_{name}.jsonl"), "w") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")
    counts = {l: sum(1 for x in rows if x["label"] == l) for l in LABELS.values()}
    print(name, pages, "pages", len(rows), "dropped", dropped, counts)


if __name__ == "__main__":
    main(sys.argv)
