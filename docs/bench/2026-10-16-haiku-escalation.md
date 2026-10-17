# Haiku escalation of the unsure middle via the Claude Code CLI (2026-10-16)

## Setup
- `tools/haiku_escalate.py`: claims layer 1 leaves unsure and large+roberta does not pass behind the wording gate,
  50 per `claude -p --model haiku` call (owner's subscription, no API key). Haiku must cite exact quote spans; code
  checks every span occurs in the quote and every claim number occurs in a span.
- Calibration: all 2,642 unsure rows. Test: 1,800 of 2,812, then stopped by the owner's concern about subscription
  usage. Confirmation not run.

## Results
| Split | Rows | Haiku "stated" | Right by label |
|---|---:|---:|---:|
| cal | 2,642 | 316 | 306 (96.8%) |
| test (partial) | 1,800 | 157 | 152 (96.8%) |

Reading the 15 disagreements (auditor Claude, method of docs/audit/2026-10-12-vitaminc-stated-errors.md): about 8
are label errors (e.g. cal 658, 1115, 1162, 1726, 1820; test 485) and about 7 are real Haiku errors (e.g. cal 2900
"Watertown is in France" passed against a Massachusetts quote; cal 1751 "pop rock" vs "rock"; test 397/406 date
direction; test 486 "diverse" vs "mixed to positive"). Audited precision is roughly 98.5%, similar to the local
judges, not above them.

## Decision
Not adopted. Haiku in one pass does not reach the precision the 0.99 bar needs, and adding its calls would lower the
combined lower bound. No further subscription runs without owner approval.

## Standard v1 verdict
No pass. Binding gate: `pass_lower_bound`. The measured limit is now label noise in VitaminC as much as the judges:
both the local and Haiku disagreements are about half label errors.
