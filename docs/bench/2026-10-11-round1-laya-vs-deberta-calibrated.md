# Round 1: LAYA vs DeBERTa-v3-xsmall, calibrated, against standard v1 (2026-10-11)

## Setup
- Question: does the quote state the claim? (supports = stated; neutral and contradiction = not stated)
- Data (public, local only): FEVER-as-NLI dev (`pietrolesci/nli_fever`, 919 rows from 10 spaced pages) and MNLI validation_matched
  (1,000 rows), shuffled (seed 7) and split 50/50 per set into calibration (959) and test (960). Nothing is fitted on test.
- Stress set: 460 cases built from test-split "supports" rows: 30 with one figure changed, 200 with "not" inserted after the
  first is/was/are/has/can/will, plus the 230 originals. Labels by construction.
- Judges: LAYA via `tools/laya_shim.py` (Router "english", the engine's own question wording); `cross-encoder/nli-deberta-v3-xsmall`
  fp32 ONNX via `ej-nli` (`ort` 2.0.0-rc.10, 1 thread). Mean of both calibrated scores as an ensemble.
- Fitting: Platt scaling on calibration, then thresholds whose 95% Wilson lower bound meets the target, min 20 acted per side.
- Machine: cloud container, 4 vCPU Intel Xeon @ 2.80GHz, release build; other jobs were sometimes running, so latency is an upper bound.

## Results on the test split (measured)
| Judge | AUROC | ECE raw → calibrated | Target 0.99 | Target 0.95 | Target 0.90 |
|---|---|---|---|---|---|
| LAYA | 0.875 | 0.086 → 0.036 | acts on nothing | "not stated" only: 136/139, coverage 14.5% | "not stated" only: 436/462, coverage 48.1% |
| DeBERTa-v3-xsmall | 0.956 | 0.071 → 0.029 | acts on nothing | "not stated" only: 492/506, coverage 52.7% | stated 198/210, not stated 585/639, coverage 88.4% |
| Mean of both | 0.952 | 0.099 | acts on nothing | "not stated" only: 424/437 | "not stated" only: 581/618 |

LAYA with the engine's current raw bars (yes ≥ 0.6, no ≤ 0.3) on the same test split: 378 "stated" calls, 257 right (68%);
452 "not stated" calls, 427 right.

Stress (raw p ≥ 0.6, i.e. a "stated" call under today's engine bar):
| Judge | Negated claims passed | Changed-figure claims passed |
|---|---|---|
| LAYA | 51 / 200 | 3 / 30 |
| DeBERTa-v3-xsmall | 12 / 200 | 2 / 30 |
Layer 1 alone (ej check, no model) marks 29 of the 30 changed-figure claims as mismatch.

Latency p95 per claim: DeBERTa-v3-xsmall 130 ms; LAYA 708 ms (includes the Python shim and pipe).

## Standard v1 verdict
No configuration passes. Binding gate: `pass_lower_bound` (0.99) — no local judge can make a "stated" call at that bound on
this data, so every claim layer 1 does not settle as backed must go to Claude. LAYA also fails `max_p95_ms`.
Usable now: DeBERTa-v3-xsmall as a "not stated" pre-filter at target 0.95 (lower bound 0.954, coverage 52.7%), which saves
Claude calls on clearly unsupported claims; and the engine's LAYA "stated" bar should not be trusted as-is.

## Next (round 2)
Stronger open verification models under the same protocol: cross-encoder DeBERTa-v3-base (Apache-2.0), MoritzLaurer
DeBERTa-v3-large mnli-fever-anli-ling-wanli quantised (MIT), then MiniCheck (MIT, needs ONNX export). More held-out labelled
cases (teacher pairs) to make a 0.99 bound reachable at all.
