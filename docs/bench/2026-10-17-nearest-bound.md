# Gate: nearest-figure bounds (2026-10-17)

Question: does checking a claim's bound ("more than N") against the nearest comparable quote figure, instead of any
figure, stop the comparative-figure errors without losing coverage? Research: docs/research/2026-10-17-nearest-bound.md.

- Change (gate only, `figures_backed`): a bound is met when the quote states the same bound, or else when the
  comparable quote figure nearest to it satisfies it. Before, any comparable figure that satisfied it was enough.
- Data: frozen cal/test/confirmation; the read validation set (`--validation`); and a third, unread slice
  `tools/fetch_general.py DATA --validation2 --pages 300` (29,897 rows from the pages between the validation pages, 103
  dropped as duplicates of earlier sets). The cascade is frozen: large+roberta, thresholds (0.000207, 0.9733) fitted on
  calibration only. Only rows layer 1 leaves unsure and either gate passes were scored (2,454 on val2).
- Commands: `ej-bench words` per split (old gate built from the parent commit in a worktree), `ej-bench score` per
  judge, `python3 -I tools/local_cascade.py DATA large+roberta --gate --confirm --val --val2`.
- Machine: cloud container, 4 CPUs, no GPU. No model calls to Claude. Lower bound = Wilson 95%.

## Results (raw labels unless stated)
| Set | Old gate | LB | New gate | LB |
|---|---:|---:|---:|---:|
| Calibration | 156/156 | 0.9760 | 157/157 | 0.9761 |
| Test | 198/199 | 0.9721 | 198/199 | 0.9721 |
| Confirmation | 786/799 | 0.9724 | 788/801 | 0.9724 |
| Validation (read; rule found here) | 1628/1664 | 0.9702 | 1663/1697 | 0.9721 |
| Val2 (unread) | 1628/1653 | 0.9778 | 1664/1692 | 0.9762 |
| Val2, changed rows label-checked | 1628/1653 | 0.9778 | 1668/1692 | 0.9790 |

Val2 rows that changed, read by hand:
- Removed (2): 7265 "more than $ 120 million" vs a $ 118.7 million total, a real error now blocked; 1511 "more than
  $ 65 million" worldwide vs $ 67.4 million, a correct approval lost (the nearest figure was the $ 64.2 million domestic gross).
- Added (41): all are claims whose bound the quote states. The 4 raw "errors" among them (13434, 15321, 17857, 17858)
  are label errors; each claim appears word for word in its quote.
- The same-bound clause was added after reading the first val2 run (it removed 5 such correct approvals). It is a
  logical identity, not a fitted threshold, but val2 is no longer fully unread for this check.

## Verdict
Kept. On the frozen confirmation set coverage rises (+2) at the same errors. On val2, label-checked, coverage rises by
40 approvals and real errors fall by 1 (LB 0.9778 to 0.9790). `pass_lower_bound` (0.99) still binds; no bar changed.
