# RoBERTa-large-MNLI as a second-family judge (2026-10-16)

## Setup
Same data, machine (4-vCPU Intel Xeon @ 2.10GHz), gate and protocol as
docs/bench/2026-10-16-reproduction-and-agreement.md. Judge: `Xenova/roberta-large-mnli@f44e4108…` fp32 ONNX, scored
with `tools/rebuild_inputs.sh /home/user/data roberta`. Research note: docs/research/2026-10-16-second-family-judge.md.
Command: `python3 -I tools/local_cascade.py /home/user/data JUDGE --gate --confirm`; thresholds fitted on calibration only.

## Results
| Judge | Threshold(s) | Calibration | Test | Test lower bound | Confirmation | Confirmation lower bound |
|---|---|---:|---:|---:|---:|---:|
| large (previous best) | 0.9866 | 143/144 | 206/208 | 0.9656 | 748/763 | 0.9678 |
| roberta | 0.9723 | 159/159 | 203/204 | 0.9728 | 799/814 | 0.9698 |
| large+roberta | 0.0002, 0.9733 | 156/156 | 198/199 | 0.9721 | 786/799 | **0.9724** |
| xsmall+roberta (not pre-registered) | 0.9644, 0.3021 | 161/161 | 234/235 | 0.9763 | 849/874 | 0.9581 |

The fitted large threshold of 0.0002 makes large a veto: RoBERTa decides, and large blocks only rows it scores as near
certainly not entailed.

## Decision
Kept as the best local setup: large+roberta raises the confirmation lower bound 0.9678 -> 0.9724 and RoBERTa alone
0.9678 -> 0.9698 (both pre-registered in the research note). Caveats: the gain is small against the interval width,
and xsmall alone still has the highest test lower bound (0.9764) but the lowest on confirmation. Two large models per
claim roughly double latency; measure on a quiet machine before any policy uses it.

## Standard v1 verdict
No pass. `pass_lower_bound` binds: 0.9724 vs 0.99. Even with zero further errors, ~460+ clean stated calls are needed;
the remaining errors (13 of 799 on confirmation) need either a stronger checker or escalation.
