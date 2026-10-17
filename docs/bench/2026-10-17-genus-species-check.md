# Genus vs species layer-1 rule (2026-10-17)

- Data: VitaminC frozen cal / test splits and the confirmation set (`/home/user/data`).
- Cascade: large+roberta behind the wording gate, thresholds (0.000207, 0.9733), unchanged, nothing refitted.
- Rule: candidate 1 of docs/research/2026-10-17-genus-species-check.md, prototyped in Python over the bench rows.
- Machine: cloud container, CPU only. No model calls. Lower bound = Wilson 95%.

| Set | Approved before | Rule removes | Approved after | LB before | LB after |
|---|---:|---|---:|---:|---:|
| Calibration | 156/156 | none | 156/156 | 0.9760 | 0.9760 |
| Test | 198/199 | none | 198/199 | 0.9721 | 0.9721 |
| Confirmation (raw) | 786/799 | 2914 (error) | 786/798 | 0.9724 | 0.9739 |
| Confirmation (audited) | 794/799 | 2914 (error) | 794/798 | 0.9854 | 0.9872 |

Across all 17,393 rows (approved or not) it fires once, on row 2914.

## Verdict
Not adopted. The gain comes only from the row the rule was written from, so it is not evidence about unseen
claims, and no gate changes verdict (all stay under 0.99). Adding code for one case would make the confirmation
number less honest. The binding gate stays `pass_lower_bound`.
