# Audited lower bound across all three untouched sets (2026-10-17)

Data: VitaminC confirmation (801 approvals), val (1697), val2 (1692); frozen large+roberta thresholds, nearest-figure
gate (docs/bench/2026-10-17-nearest-bound.md). Machine: the cloud container, local CPU only, no model calls.

## Method
No new reading was needed for confirmation and val. Their approvals under the old and new gate were diffed
(`old/vitc_{conf,val}.words` against `vitc_{conf,val}.words`):
- confirmation: +2 approvals (4622, 11348), both labelled supports; no approval removed.
- val: +35 approvals, all labelled supports; removed 1965 (supports) and the real errors 7363 and 19216.
So every non-supports approval under the new gate was already read in an earlier audit
(docs/audit/2026-10-17-validation-stated-errors.md, docs/audit/2026-10-17-val2-stated-errors.md, and the earlier
confirmation audit). Arguable rows count as errors.

## Results
| Set | Audited right/called | Wilson 95% LB |
|---|---:|---:|
| Confirmation | 796/801 | 0.9855 |
| Val | 1688/1697 | 0.98995 |
| Val2 | 1685/1692 | 0.9915 |
| Pooled | 4169/4190 | 0.9923 |

## Verdict
Pooled over 4,190 held-out approvals, the audited lower bound passes `pass_lower_bound` (0.99). Each set alone is at or
near the bar: val misses by 0.00005, and confirmation is too small to clear 0.99 with 5 errors. Caveats: approvals
labelled supports were not read (label noise there would only lower the figure), and the audit relabels by hand, so
this is evidence for the standard, not the `ej-bench fit` verdict itself.
