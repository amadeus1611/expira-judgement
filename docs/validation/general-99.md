# General layer 99% validation protocol

Status: research specification, not an implemented complete validation procedure or a successful validation. The
initial [offline production capture](cascade-capture.md) records deterministic pre-escalation diagnostics only. Research sources:
`docs/research/2026-10-11-rigorous-99-validation.md`. Existing standard v1 and frozen data remain authoritative.
The staged implementation, population/source design, label workflow and architecture experiments are in
[`general-99-roadmap.md`](general-99-roadmap.md). That roadmap is also a plan, not a certification.

## Before fitting or running
1. Declare the target population, sampling unit, inclusion rules, label rubric, total validation budget, sample size,
   seeds and stopping rule. Accuracy means correctness among final stated decisions; report stated coverage, block
   correctness and abstentions separately. A broad web-text claim requires representative web-text data, beyond one
   Wikipedia benchmark. Do not assume a foundation model has never seen a public benchmark.
2. Register hashes of source snapshot, dataset manifests, calibration/test/stress files, code commit, model/build,
   prompt, schema, effort, policy and routing. Retain original case and source identifiers in a companion manifest;
   the current frozen three-field JSONL format discards them. Pin data bytes, not just a mutable URL or row count.
3. Check exact and normalised duplicates, contrastive groups and shared sources across partitions without consulting
   final labels. New partitions group these units before selection. Record near-duplicate detection rules before
   outcomes are available. Leave old frozen partitions intact and report their measured overlap as a limitation.
4. Allocate development/score calibration, risk calibration, and a sealed final confirmation sample by independent
   groups. Only the permitted development/calibration partitions may influence candidate selection. Existing test,
   stress and audit outcomes are never fit inputs. They have already been inspected and cannot become untouched data
   by renaming them. Any policy revised after a final outcome needs another untouched confirmation sample.
5. Select exactly one candidate and freeze it before final predictions. If there is a candidate family, declare all
   members and the simultaneous procedure in advance. An ordinary 95% interval for the winner of a search is
   insufficient. The family budget includes model, prompt, effort and threshold variants, not just model names.

## Required runner tests before any subscription batch
- Independent references for Wilson and exact binomial tails: zero cases, all correct, all wrong, nonzero errors,
  threshold equality and one-less-than-minimum n. Check monotonicity and enumeration of small binomial samples.
  For any proposed risk-selection procedure, simulate false certification under a null risk of at least 1%, using a
  preregistered seed/count and a confidence bound on the observed violation frequency. Test selection as a whole,
  including multiplicity; comparing an implementation with itself proves nothing.
- Identity invariants: unique row IDs, correct data/model/prompt hashes, grouping integrity, complete one-to-one
  prediction/label joins. Reject missing, repeated, reordered or foreign score rows before computing statistics.
  Resume rejects a different dataset, policy, model or schema. Do not silently merge outputs from different runs.
- Route fixtures exercise the real production path: layer 1 prevents later calls; an unproven local policy never
  decides; name checks and confidence gates match production; malformed escalation never becomes supports. Check
  runner decisions against `ej check`, including verdict-to-binary and three-way label mapping.
- Adapter failure tests: nonzero exit, timeout, invalid JSON, missing/extra schema fields, invalid confidence, absent
  or fabricated decisive span, refusal, missing usage and missing price. Fail closed, log failures, retain them in
  the denominator. Unknown cost stays unknown. A citation substring proves provenance only, not entailment.
- Property tests and zero missed mutants still gate decision logic. Mutant timeouts and unviable mutants are reported
  separately. Inference and quantised/fp32 equivalence need actual models; an early-return smoke test is not evidence.

## Prediction and measurement
1. Run the frozen layer-1 policy first. A local judge may act only if its unchanged v1 policy is eligible, with the
   same production guards used during evaluation. Run the owner's Codex escalation only on the residual rows.
   Preserve per-row final verdict, routing, scores, confidence, citation, errors, latency and reported usage. Do not
   reveal final labels to the policy/model and do not silently rerun only failed or inconvenient outcomes.
2. Begin live development work with at most 10 calibration rows, one worker, a bounded per-call timeout and no automatic
   repeated batch. Report attempts, completed calls, tokens where supplied, known dollar subtotal and number of
   unpriced calls. The final sample size and budget are decided before a confirmation run; small development batches
   cannot establish 99%. No Codex price or acceptance rate is presently measured here.
3. Compute v1's stated and block bounds, fresh stress wrong passes, ECE, local coverage and quiet local p95. Report
   escalation coverage and end-to-end latency separately, retaining the meaning of the local coverage/latency gates.
   Count pre-escalation deterministic and eligible local decisions together, with each route also reported; model
   escalation coverage cannot satisfy the required 50% local coverage. Unmeasured gates
   fail. Keep a three-way confusion table even though v1 block correctness pools contradicts and unsure as not stated.
4. Report Wilson on the original fixed rows for compatibility, marking dependence/selection limitations. Confirmation
   must use the declared independent sampling design. For a preselected single row per independent group, confidence
   statements apply to that group-balanced sampling distribution. A cluster bootstrap can be a sensitivity analysis;
   it cannot manufacture a 99% rare-error guarantee from a few correlated groups or substitute for the design.
5. Supplement the unchanged v1 stated gate with an exact 97.5% one-sided lower bound, equivalent to the lower endpoint
   of a two-sided 95% Clopper–Pearson interval. Require both checks. For e observed false passes among n stated calls,
   test the boundary error risk 0.01 using:

   `P(Binomial(n, 0.01) <= e) = sum(comb(n,j) * .01**j * .99**(n-j), j=0..e)`.

   The exact check passes at tail <= 0.025 for one frozen candidate. With K prespecified candidates, a simple
   Bonferroni supplement uses 0.025/K per candidate. This does not change v1's Wilson z or bars. Ordinary fixed-sample
   intervals do not justify repeatedly inspecting results and stopping at the first passing bound. Use a fixed total
   sample, or separately research/validate a prespecified anytime-valid procedure; do not extend a failed run until
   it passes. If accepted counts are insufficient at completion, the outcome is insufficient evidence.
6. Audit after freezing predictions, under the blind procedure in the research note. Publish raw, audited and
   unresolved-case sensitivity results on the same row IDs; no cherry-picked deletion or denominators. Raw v1 stays
   visible. Audited performance cannot silently become a passed raw-label policy. Without independent adjudication,
   label changes remain provisional. Keep audit data away from all fitting and model/prompt selection.
7. Publish unsuccessful candidates, every gate, data/sampling limitations, costs and hashes. Match the claim to its
   measured population. A confidence level applies to the repeated-sampling procedure, not a probability that every
   future claim is correct. Model/service changes require renewed validation.

## Computed sample requirements, not a stopping rule

These are counts of independent stated decisions for a single frozen candidate, not total input rows. Calculated with
v1's z=1.96 and the binomial tail above at 60-decimal precision. Each previous n was checked to fail its respective test.

| False passes | v1 Wilson minimum n | Exact minimum n | Minimum satisfying both |
|---|---:|---:|---:|
| 0 | 381 | 368 | 381 |
| 1 | 563 | 555 | 563 |
| 2 | 726 | 720 | 726 |
| 5 | 1,166 | 1,164 | 1,166 |
| 10 | 1,836 | 1,835 | 1,836 |

For illustration, prespecified families of 10 and 100 candidates need respectively 597 and 826 error-free independent
stated calls for the Bonferroni exact supplement. These hypothetical counts do not certify our previous model search.
The unchanged v1 gates are necessary; data independence, untouched confirmation and procedure integrity must also hold.
