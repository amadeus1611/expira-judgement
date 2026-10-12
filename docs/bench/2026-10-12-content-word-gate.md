# Content-word gate: local cascade 0.9418 -> 0.9769 on VitaminC test (2026-10-12)

## Setup
Same frozen splits, layer-1 build and machine as docs/bench/2026-10-12-layer1-false-blocks.md. Local judge:
DeBERTa-v3-xsmall scores from round 3. No model calls. Production rule (`ej_evidence::unstated_words` plus
`names_in_quotes`, applied in `adjudicate`) exported per row by `ej-bench words`; cascade by
`python3 -I tools/local_cascade.py DATA xsmall [--gate]`. Threshold fitted on calibration only.

## Results
| Cascade | Threshold | Calibration | Test "stated" right | Test lower bound |
|---|---:|---:|---:|---:|
| layer 1 + xsmall | 0.9898 | 195/198 | 214/220 (97.3%) | 0.9418 |
| + wording gate (k = 0) | 0.9641 | 165/166 | 240/241 (99.6%) | 0.9769 |

Choice of k on calibration only: k=0 bound 0.9667, k<=1 0.9669 (test 284/288, 0.9648), k<=2 0.9652, none 0.9564.
k = 0 and k <= 1 tie on calibration (0.9667 vs 0.9669); k = 0 was taken as the stricter rule, which the test split
then favoured. Report both: the selection is not free of that tie.

The one remaining test error: "The Colbert Report was taped Monday." against "taped and broadcast Monday through
Friday", labelled contradicts (arguably a label error; not corrected here).

Python and Rust gate counts agree on every test row; on calibration one row ("I was a commercial success")
differs on tokenisation; Rust is the production rule and all numbers above use it.

Escalation (Haiku, round-3 partial run, seed-7 sample): 277/289 overall; on gate-passing rows 51/51; on rows the gate
fails 226/238. Claude's remaining errors are on paraphrased claims.

## Verdict
No pass yet: 0.9769 < 0.99. Coverage is the limit: at most 338 test rows pass the gate where layer 1 is unsure, and a
0.99 bound needs 381 error-free calls. Next: grow proven coverage (escalation on gate-failing rows with agreement,
larger held-out set), and the label audit.
