# FOUND-001 v1 — Review record

Reviewed: 2026-10-14 (Asia/Manila). Sources were inspected directly using the browsing
tool; evidence files contain brief unchanged excerpts. The report's evidence quote
is independent source text, not this lesson or another AI explanation.

## Existing-engine findings

`ej-study --lines` called the existing `ej check` without a local model/policy.
Exact commands, exit codes and stderr are retained alongside original JSON reports.

| Claim | Original verdict | Original reason | Tutor review |
|---|---|---|---|
| A bit can take the value 0 or 1. | `unsure` | no judge available | Supported by CS50's inspected bit definition; manually reviewed, engine unresolved |
| Binary is also called base-2. | `unsure` | no judge available | Supported by CS50's inspected binary terminology; manually reviewed, engine unresolved |
| A byte contains eight bits. | `unsure` | no judge available | Supported by CS50's inspected byte terminology; manually reviewed, engine unresolved |
| Program definition quotation | `backed` | claim appears word for word in a quote | Exact quotation and its context reviewed in Think Python section 1.1 |

- [CS50 report](reports/cs50.json), [invocation](reports/cs50-invocation.json): exit 1.
- [Program report](reports/program.json), [invocation](reports/program-invocation.json): exit 0.

All three flags were reviewed against the original passages. Their engine verdicts
remain unchanged. No eligible local judge was installed, no thresholds were changed,
and no model accuracy or lesson-accuracy rate is claimed. A quotation match establishes
textual source support; it does not independently establish the source's correctness.

## Examples and exercise review

The switch analogy is explicitly a teaching model. The pattern table, doubling
argument and byte conversion were checked mathematically and by actual Python
enumeration: [arithmetic-demo.json](reports/arithmetic-demo.json) retains code,
runtime version, output and process result. These are tutor-side observations,
not learner lab results or an EXPIRA mathematical proof.

Exercise assumptions permit all bit patterns; the byte conversion has a single
answer; the addition example defines its own inputs and operation. All exercises
are unscored diagnostics. No exam-objective mapping or learner performance is claimed.

## Next review

Record the learner's actual responses to the three questions and any hints required.
Clarify misconceptions before the next segment on information interpretation and
communication. Preserve v1 sources/reports if the lesson is revised.
