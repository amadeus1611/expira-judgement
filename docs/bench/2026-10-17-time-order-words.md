# Time-order words required in the gate (2026-10-17): reverted

Data: VitaminC cal, test, conf, val, val2; frozen large+roberta thresholds; machine: the cloud container, local CPU.
Change: "after", "before" and "during" removed from `FUNCTION_WORDS`. `ej-bench words` re-run per split; the gate can
only block, so no new judge scores were needed. Approvals blocked by the change, against the current gate:

| Set | Approvals | Blocked correct | Blocked errors |
|---|---:|---:|---:|
| Calibration | 156 -> 155 | 1 | 0 |
| Test | 199 -> 199 | 0 | 0 |
| Confirmation | 801 -> 799 | 2 | 0 |
| Val | 1697 -> 1686 | 11 | 0 |
| Val2 | 1692 -> 1687 | 4 | 1 (4276) |

The blocked correct rows were paraphrases ("following" for "after", "prior to" for "before", "when ... later",
"succeeded by"). On val2 the audited result would go from 1685/1692 (LB 0.9915) to 1685/1687 counting the error removed,
but it removes 18 correct approvals to drop 1 error; coverage at the same risk falls on every other set. Verdict:
reverted (iterate step 6).
