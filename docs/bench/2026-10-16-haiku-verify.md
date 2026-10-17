# Haiku as a second check on local approvals (2026-10-16)

Question: does requiring Claude Haiku to agree (veto) on every claim the local cascade calls "stated" raise the
`pass_lower_bound`?

- Data: VitaminC frozen cal / test splits and the 11,393-row confirmation set (`/home/user/data`).
- Local cascade: layer 1, then large+roberta behind the wording gate, thresholds (0.000207, 0.9733) fitted on
  calibration (docs/bench/2026-10-16-roberta-second-family.md). Unchanged; nothing refitted.
- Haiku: `tools/haiku_escalate.py DATA SPLIT --verify`, `claude -p --model haiku` on the owner's subscription,
  50 claims per call (24 calls in all), cited-span check in code. Labels never sent.
- Rule: "stated" only if the local cascade says stated AND Haiku says stated with valid spans.
- Machine: cloud container, CPU only. Lower bound = Wilson 95%.

## Results
| Set | Local alone | LB | Local AND Haiku | LB | Errors caught |
|---|---:|---:|---:|---:|---|
| Calibration | 156/156 | 0.9760 | 129/129 | 0.9711 | none to catch |
| Test | 198/199 | 0.9721 | 151/151 | 0.9752 | 2000 |
| Confirmation (raw labels) | 786/799 | 0.9724 | 614/620 | 0.9790 | 146, 438, 790, 2442, 2443, 4196, 10104 |
| Confirmation (audited labels) | 794/799 | 0.9854 | 618/620 | 0.9883 | 438, 2442, 2443 |

Audited labels are those of docs/audit/2026-10-16-large-roberta-stated-errors.md. Of the 5 real or arguable errors
there, Haiku vetoes 3 (438 "many of them", 2442/2443 announcement) and misses 2914 (genus vs species) and 3598
(ungrammatical claim). It also vetoes 4 rows the audit decided are label errors (146, 790, 4196, 10104).

The cost: Haiku drops 22% of the correct approvals (172 of 786 on confirmation) back to unsure.

## Verdict
Not adopted. Raw confirmation LB rises 0.9724 -> 0.9790 and audited 0.9854 -> 0.9883, but both stay under the
0.99 bar, and coverage falls by about a fifth. With 620 approvals the bar allows 0 errors (620/620 gives 0.9938;
619/620 gives about 0.9910, 618/620 0.9883). The binding gate is still `pass_lower_bound`, now limited both by the
2 arguable errors and by label noise in the measurement itself.

Next: a hand-verified clean test set (no model calls), so the number measures the system rather than VitaminC's
label errors.
