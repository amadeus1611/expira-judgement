# BART-large-MNLI as a third judge family (2026-10-16)

## Setup
- Model: `Xenova/bart-large-mnli@71a9d2a7c389b1a812afc3cc1668f8c877681b00` fp32 ONNX (MIT), research note
  docs/research/2026-10-16-second-family-judge.md.
- Same frozen cal/test and 11,393-row confirmation set as docs/bench/2026-10-16-reproduction-and-agreement.md.
- Scores: `tools/rebuild_inputs.sh /home/user/data bart`; the run hit the session time limit after 11,018
  confirmation rows, so the last 375 rows were scored separately with `ej-bench score` and appended with the row
  index offset (11,393 rows, labels checked).
- Machine: 4-vCPU cloud container, Intel Xeon @ 2.10GHz, about 0.35 s per row. No model calls; no subscription usage.
- Command: `python3 -I tools/local_cascade.py /home/user/data JUDGE --gate --confirm`.

## Results (thresholds fitted on calibration only)
| Judge | Threshold | Calibration | Test | Test lower bound | Confirmation | Confirmation lower bound |
|---|---|---:|---:|---:|---:|---:|
| bart | 0.9906 | 128/128 | 183/184 | 0.9699 | 653/670 | 0.9597 |
| large+bart | 0.8714, 0.9890 | 132/132 | 189/190 | 0.9708 | 696/712 | 0.9638 |
| roberta+bart | 0.9733, 0.00001 | 156/156 | 198/199 | 0.9721 | 786/799 | 0.9724 |
| large+roberta (current best) | | | | | | **0.9724** |

## Decision
Rejected. No setup beats 0.9724; roberta+bart only ties, and its fitted BART threshold is near zero, so BART
adds almost nothing there. A third family does not move the binding gate. Best stays large+roberta.

## Standard v1 verdict
No pass. Binding gate: `pass_lower_bound` (0.99). The free local options are spent; the next step is cleaner
held-out data or paid escalation of the unsure middle (owner approval needed).
