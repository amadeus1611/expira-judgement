# Reported/planned cue in the wording gate: rejected (2026-10-16)

## Setup
Same data, machine and protocol as docs/bench/2026-10-16-roberta-second-family.md. Prototype only: the cue rule of
docs/research/2026-10-16-reported-modality.md applied to the `ej-bench words` rows (folded into `figures_ok`), then
`tools/local_cascade.py DATA JUDGE --gate --confirm`. Rows flagged: 78 of 2,905 cal, 77 of 3,095 test, 236 of 11,393
confirmation.

## Results (lower bounds; base -> with cue)
| Judge | Calibration | Test | Confirmation (not a fair judge here) |
|---|---|---|---|
| large | 143/144 0.9617 -> 142/143 0.9615 | 206/208 0.9656 -> 205/207 0.9655 | 0.9678 -> 0.9674 |
| roberta | 159/159 0.9764 -> 158/158 0.9763 | 203/204 0.9728 -> 202/203 0.9726 | 0.9698 -> 0.9724 |
| large+roberta | 156/156 0.9760 -> 155/155 0.9758 | 198/199 0.9721 -> 197/198 0.9720 | 0.9724 -> 0.9751 |
| xsmall | 161/162 0.9659 -> 160/161 0.9657 | 235/236 0.9764 -> 234/235 0.9763 | 0.9569 -> 0.9590 |

## Decision
Rejected: on the sets that can judge it, it removes one correct call each and catches no error. The confirmation
gain (two Lazerism-type errors) comes from the very rows that suggested the rule, so it is not evidence. No Rust change.
