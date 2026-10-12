#!/usr/bin/env python3
"""Rebuild the frozen general (web-text) calibration and test splits used by ej-bench, without storing the data in the repo.

    python3 -I tools/fetch_general.py OUT_DIR [--pages 60]
    python3 -I tools/fetch_general.py OUT_DIR --confirm [--pages 120]

--confirm builds an untouched confirmation set instead: pages at the midpoints between the 60 frozen offsets (and,
with more pages, further evenly spaced offsets that avoid every frozen page by at least 100 rows), all rows kept, written
to OUT_DIR/vitc_confirm.jsonl. It is never used to fit anything; it only confirms a configuration fitted on calibration.

Source: VitaminC test split (tals/vitaminc on Hugging Face, CC BY-SA 3.0), fetched through the public datasets-server API
in fixed pages at evenly spaced offsets, so every run gets the same rows. Labels: SUPPORTS -> supports, REFUTES ->
contradicts, NOT ENOUGH INFO -> unsure. Rows are split into calibration and test by a hash of `case_id`, never by row:
VitaminC's contrastive pairs share a case, and a pair split across both halves would leak the answer into the test.

Writes OUT_DIR/vitc_cal.jsonl and OUT_DIR/vitc_test.jsonl ({"claim","quote","label"} per line) and prints the counts.
Standard library only."""
import hashlib, json, os, sys, time, urllib.request

API = "https://datasets-server.huggingface.co/rows?dataset=tals/vitaminc&config=default&split=test&offset={}&length=100"
TOTAL = 55197  # rows in the VitaminC test split (datasets-server num_rows_total, 2026-10-11)
LABELS = {"SUPPORTS": "supports", "REFUTES": "contradicts", "NOT ENOUGH INFO": "unsure"}


def fetch(offset):
    for attempt in range(5):
        try:
            with urllib.request.urlopen(API.format(offset), timeout=60) as r:
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
    with open(os.path.join(out, "vitc_confirm.jsonl"), "w") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")
    print("confirm", len(offsets), "pages", len(rows), {l: sum(1 for x in rows if x["label"] == l) for l in LABELS.values()})


if __name__ == "__main__":
    main(sys.argv)
