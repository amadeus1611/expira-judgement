# Second untouched set: VitaminC validation split (2026-10-17)

Question: does the frozen local cascade hold up on a larger set drawn from a split no earlier set touches?

- Data: `tools/fetch_general.py DATA --validation --pages 300`: 30,000 rows from 300 evenly spaced pages of the
  VitaminC validation split (14,979 supports / 10,697 contradicts / 4,324 unsure), 0 rows shared with cal, test or
  confirmation. Built and frozen before any score was read.
- Cascade: layer 1, then large+roberta behind the wording gate, thresholds (0.000207, 0.9733) fitted on calibration
  only. Nothing refitted. Only the 2,394 rows layer 1 leaves unsure and the gate passes were scored.
- Commands: `ej check` and `ej-bench words` as in `tools/rebuild_inputs.sh`; `ej-bench score` per judge;
  `python3 -I tools/local_cascade.py DATA large+roberta --gate --confirm --val`.
- Machine: cloud container, 4 CPUs, no GPU. No model calls to Claude. Lower bound = Wilson 95%.

## Results
| Set | Approved correct / approved | Accuracy | LB |
|---|---:|---:|---:|
| Calibration | 156/156 | 1.0000 | 0.9760 |
| Test | 198/199 | 0.9950 | 0.9721 |
| Confirmation (raw labels) | 786/799 | 0.9837 | 0.9724 |
| Validation (raw labels) | 1628/1664 | 0.9784 | 0.9702 |
| Validation (audited labels) | 1653/1664 | 0.9934 | 0.9882 |
| Confirmation + validation (audited) | 2447/2463 | 0.9935 | 0.9895 |

Audit of all 36 raw errors: docs/audit/2026-10-17-validation-stated-errors.md. 25 are label errors, 9 are real
errors and 2 are arguable (both counted as errors above). Correct approvals were not audited here; the earlier
140-row samples found 1 arguable in 140.

## Verdict
The cascade generalises: raw accuracy 0.978 on 30,000 fresh rows matches confirmation (0.984) within noise.
`pass_lower_bound` still binds: the audited lower bound is 0.988 here and 0.9895 pooled, just under 0.99.
The real errors fall in clear classes: 2 comparative figures ("more than 102 million" vs 101.7; "more than 65" vs
exactly 65), 2 time scope ("in 2016" vs "by 2016"; "will last two weeks" vs "at least two weeks"), 3 "X is a Y" vs
"X is in / about a Y" (song from a film, movie about a competition, album by an artist), 2 actor or verb swaps.
Next: a layer-1 comparative-figure check. These rows are now seen, so its gain must be measured on a further fresh
slice of the validation split (pages between the 300 used here), not on this one.
