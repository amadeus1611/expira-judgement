# EXPIRA Judgement

Layered anti-hallucination engine. Nothing passes as fact unless it is proven or a judge with a recorded reason backs it.

| Layer | Crate | What decides | Cost |
|---|---|---|---|
| 1 | `ej-evidence` | Deterministic: source + valid date + quote present, every figure/unit in the claim appears in a quote, staleness, sums | free, certain |
| 2 | `ej-judge` | Local judge for paraphrases (pluggable `Judge` trait; `OverlapJudge` placeholder until the LAYA vs NLI bake-off) | cheap |
| 3 | `ej-judge` | Claude, only for what layer 2 leaves `unsure` or high stakes; must give a reason | paid |

Supporting systems:
- `ej-delegate`: agent roster. Cheapest tier first (scout -> worker -> builder -> reviewer), escalate one tier only when the gate fails, high stakes start at reviewer, every attempt logged.
- `ej-habits`: in-repo self-improvement, no model training. Habits are candidates until enough observations agree, adopted, and retired on the first contradiction. Each Claude ruling is stored as a teacher pair for layer 2's eval set.
- `ej-cli`: `ej check claims.json --today 2026-10-11 --max-age 365` (exit 1 if any claim fails).

Verdicts: `backed`, `supported` (pass); `unbacked`, `mismatch`, `stale`, `unsure` (fail; escalate or leave out).

## Roadmap
1. Layer 1 hardening on real ledger cases (dates, ranges, currencies, negation).
2. Bake-off harness: LAYA vs a DeBERTa NLI model vs a small constrained model, ONNX in Rust (`ort`/`candle`).
3. Claude escalation client (Messages API), rulings written as teacher pairs.
4. Habit store on disk + reporting; tier log.
5. Hosted API.

Later, separate: EXPIRA Design (palette, type and derivative solver under rules).

Private. Not for publication without the owner's decision.
