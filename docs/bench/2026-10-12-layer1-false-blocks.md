# Layer 1 false blocks and the whole local cascade (2026-10-12)

## Setup
- Data: VitaminC frozen splits from `tools/fetch_general.py` (SHA-256 `bc62be66…` cal, `cb223f92…` test, same as
  docs/bench/2026-10-11-rigorous-99-baseline.md): 2,905 calibration, 3,095 test rows. Labels as given.
- `ej check` (release build) on every row, source "VitaminC supplied quotation", date and `--today` 2026-10-11.
- Machine: 4-vCPU cloud container. No model calls; no subscription usage.
- Codex's Platt-fit rewrite (PR #3) re-measured first: the xsmall fit on these splits differs from the previous code
  only below 1e-10 (a 0.276655504620 vs 0.276655504566); test ECE, AUROC and thresholds unchanged. Kept.

## Layer 1 on the full test split
"Blocked true" = a mismatch on a row labelled supports. Block lower bound = 95% Wilson bound on mismatches that are
not labelled supports.

| Commit | Change | Mismatches | Blocked true | Block lower bound | Backed right |
|---|---|---:|---:|---:|---:|
| main (6d97fe5) | before | 411 | 119 | 0.6648 | 25/25 |
| 0ac6eb8 | absent figure escalates; only a rival figure of the same kind mismatches | 137 | 28 | 0.7205 | 25/25 |
| 5e66263 | decades and centuries are spans, not years | 95 | 10 | 0.8170 | 25/25 |
| current | approximate wording is a bound; "US $" read as currency | 84 | 4 | 0.8839 | 25/25 |

On the historical 1,000-row seed-7 sample: blocked true 33 -> 1. Remaining four: two quotes with a garbled date
("245March"), "16 March" without a year against "March 16, 2020", and "mid-April 2020" against a quote without the year.

## Whole local cascade (layer 1, then DeBERTa-v3-xsmall on what layer 1 leaves unsure)
`tools/local_cascade.py`: threshold fitted on calibration only, applied unchanged to test.

| Name rule | Threshold (raw p) | Calibration | Test "stated" right | Test lower bound |
|---|---:|---:|---:|---:|
| off | 0.9898 | 195/198 | 214/220 (97.3%) | 0.9418 |
| on | 0.9869 | 235/237 | 253/261 (96.9%) | 0.9407 |

Before (xsmall alone, round 3): 298/310, lower bound 0.9336; with the name rule 290/299, 0.9438.

## Verdict
No pass of standard v1 (pass bound 0.99). The local cascade alone tops out near 0.94 on raw labels; the gap needs
escalation of the uncertain middle and the label audit (round 3 found about six of ten "errors" were label errors).
