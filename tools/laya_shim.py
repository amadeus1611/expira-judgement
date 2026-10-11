#!/usr/bin/env python3
"""LAYA behind the ej-bench line protocol (crates/ej-bench/src/command.rs).

Reads one JSON request per line, {"claim": "...", "quotes": ["..."]}, and answers one line,
{"ruling": "supports"|"contradicts"|"unsure", "confidence": c, "p": P(stated), "reason": "..."}.
LAYA answers a yes/no ("noul") question: does the text state the claim? It has no separate contradiction class, so "no"
is reported as "contradicts" (= not stated) for the act/escalate decision; `p` is the raw score for calibration.

Defaults mirror the setup measured in the owner's engine on 2026-10-11 (question wording and bars 0.6 / 0.3);
override with --question, --yes, --no. Needs `pip install laya` (Python >= 3.10, CPU torch)."""
import argparse, json, sys, warnings

warnings.filterwarnings("ignore")
QUESTION = "Does the text state this claim, with the same figures and units? Claim: {claim}"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--question", default=QUESTION)
    ap.add_argument("--yes", type=float, default=0.6)
    ap.add_argument("--no", type=float, default=0.3)
    ap.add_argument("--checkpoint", default="english", help="Router default checkpoint")
    a = ap.parse_args()
    from laya import Router
    router = Router(default=a.checkpoint)
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            req = json.loads(line)
            text = " ".join(req.get("quotes") or [])
            if not text.strip():
                out = {"ruling": "unsure", "confidence": 0.0, "p": 0.0, "reason": "no quote"}
            else:
                q = {"s": {"type": "noul", "instructions": a.question.format(claim=req["claim"])}}
                p = float(router.predict(text, q)["answers"]["s"]["noul"])
                if p >= a.yes:
                    out = {"ruling": "supports", "confidence": p, "p": p, "reason": f"laya p={p:.2f}"}
                elif p <= a.no:
                    out = {"ruling": "contradicts", "confidence": 1 - p, "p": p, "reason": f"laya p={p:.2f}"}
                else:
                    out = {"ruling": "unsure", "confidence": 0.0, "p": p, "reason": f"laya p={p:.2f}"}
        except Exception as e:  # one bad request must not end the session
            out = {"ruling": "unsure", "confidence": 0.0, "reason": f"error: {e}"[:300]}
        sys.stdout.write(json.dumps(out) + "\n")
        sys.stdout.flush()


if __name__ == "__main__":
    main()
