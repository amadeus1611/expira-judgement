# General web-text: execution and acceptance plan

Status: planned work, not implemented or certified. This makes the design decisions concrete; the existing protocol
in `general-99.md` and all standard v1 bars still apply. The agent owns source selection, experiments and execution.
Research: `docs/research/2026-10-11-general-population-plan.md`.

## Claim and population
The initial scope is English claim/evidence pairs: whether the complete claim is supported by the supplied evidence.
This states the current architecture's scope; it does not change its bars. Truth beyond the supplied evidence,
retrieval completeness and other languages require additional validation. Report performance among final stated
decisions, block correctness, abstention, stated coverage and local coverage separately. Do not describe selective
99% as 99% accuracy on every input. In-scope difficult inputs stay in the denominator.

Prepare a licensed reference corpus spanning encyclopedic prose, reporting, technical documentation, scientific or
public reports, community discussion, and commercial/policy text. These are planned source categories, not approved
datasets. Verify source permissions first. Register the source frame, snapshots, date range, inclusion rules,
source/category weights and claim-construction method before examining outcomes. An equal-category mixture, if
chosen, measures that named mixture; it is not an estimate of unknown deployment traffic. Synthetic claim variants
belong to a separately declared constructed distribution, not an invented natural request stream.
For an iid reference mixture, draw the category for each unit randomly with the registered probabilities. A fixed
stratified quota, unequal-probability sample or finite frame needs an appropriate design-based supplementary bound;
do not pass weighted or dependent observations into an ordinary binomial calculation as if they were iid. The v1
compatibility report remains visible, but cannot replace a valid bound for the actual sampling design.

The stronger deployment study will prospectively sample the actual input stream during a fixed window, using
predeclared inclusion probabilities unrelated to predictions or labels. Include malformed requests and failures in
operational accounting; distinguish them from valid semantic inputs without deleting difficult semantic cases.
Sampling and retention must respect rights to the original text. No production traffic is presently available here;
building and evaluating the open reference corpus remains useful independent work.
Existing restrictions on GSSC, client data and other repositories still hold. Reusable research data must have
compatible open permissions; coverage gaps imposed by those permissions limit the resulting population claim.

Group source revisions, repeated/near-duplicate text, contrastive variants and the same underlying claim/event before
partitioning. Select one example per sampled independent unit before consulting labels or model outputs. Record
how the unit is drawn: uniform groups gives a group-balanced claim, not a row-weighted traffic claim. Deduplication
is necessary but does not prove independence; inspect common events, templates and sources and document residual
dependence. If independence remains unsupported, report diagnostics without a rare-error certification.

Hold out source families and later time windows for transfer checks. Report their counts and uncertainty separately;
one pooled pass does not prove 99% in each category or under arbitrary distribution shift. Any simultaneous category
claims need their own adequate independent samples and a prespecified error allocation.

## Work stages and completion gates

| Stage | Work owned by the agent | Completion evidence |
|---|---|---|
| 1. Trustworthy measurement | Implement a runner around actual `ej check`, routing only through eligible policies. Pin data, identities, code, model, policy, prompt/schema and sampling plan; retain failed calls and unknown costs. | Offline route/identity/resume/failure tests, independent statistical references, zero missed required faults; each row accounted for. No live confirmation through an unauthenticated or unverified-isolation adapter. |
| 2. Data and labels | Build the licensed source registry, independent development data, score calibration, separate risk calibration, local qualification, sealed system confirmation and fresh sealed stress partitions. Check overlap before outcomes. | Licence/provenance ledger, frozen hashes, sampling probabilities and grouping report. Independent label process specified and available. Old test/stress/audit never enter fitting. |
| 3. Baseline and audit | Measure the actual cascade on diagnostic data; freeze predictions before the separate label audit. | Raw, audited and unresolved-case results share the same identities; the audit never chooses or fits a candidate. Subscription development starts with at most 10 calibration cases serially after authentication/isolation verification. |
| 4. Deterministic repair | On newly authored development cases, repair unsupported conditional acceptance and overconfident mismatch rules. Ambiguous evidence escalates. Check fit/production parity for exact-name guards. | Each small change has research, independent positive/negative/boundary examples, full checks and mutation coverage, plus before/after permitted development measurements. No test-error examples used for tuning. |
| 5. Architecture experiments | Compare full-claim verification with atomic obligations, decisive evidence and counterexample checks; compare licence-verified local builds and separately versioned tuning candidates. | Predeclared, bounded comparisons on development/score calibration. Keep measured coverage at the same risk, or lower cost/latency without degraded gates. Agreement/citations/self-confidence alone cannot approve a claim. |
| 6. Risk calibration | After score calibration, test a finite, predeclared acceptance-policy family on fresh risk calibration using a validated selective-risk procedure with multiplicity control. | Independent exact-tail/reference tests and whole-selection null experiments; frozen candidate, including all guards, prompts and thresholds, before final evaluation. No candidate is adopted solely because calibration looks good. |
| 7. Confirmation | Run one frozen candidate on a fixed, untouched representative sample and a separate fresh hard challenge set. | Unchanged v1 gates all pass; additional exact binomial check; complete costs/failures and sampling limitations. Insufficient counts or any failed gate means no certification. |
| 8. Deployment reinforcement | Version the validated policy, monitor coverage/routing, audit a fresh random stream sample and separately track new counterexamples. | Material model/prompt/data changes trigger renewed validation; drift or loss of validated conditions triggers abstention/escalation and investigation. No guarantee that a drift detector finds every shift. |

## Architecture direction, pending measurements
Keep the cascade, but require support for the **entire original claim**. An atomic checklist must preserve entity,
quantity, time, polarity, attribution, modality and conditions, plus their relationships. All extracted obligations
passing is insufficient if extraction drops a clause; check the original claim against the full evidence as well.
A model critic can propose counterexamples, but another model's agreement is not independent proof. Calibrate and
evaluate the combined acceptance rule as one candidate.

Use deterministic checks where they establish a narrow implication or contradiction. Missing a lexical number/name,
matching a substring, or finding a conditional conclusion does not by itself settle the complete semantic relation.
When a rule cannot establish it, escalate rather than issuing an unjustified certain verdict. Repairs stay before
the local layer; do not bypass layer 1 to hide its errors.

Research and compare specialised local verifiers before adding an expensive model committee. Tune only licence-cleared
weights in separately versioned artifacts using permitted training/development data; never change installed weights
in place or train on final/stress/audit examples. Training data generated by a teacher are candidate labels, not an
independent final reference. Select the architecture on measured risk, useful coverage, cost and quiet latency;
there is no measured optimal model yet. Codex remains the owner's engineering/subscription route, not an open-weight
core adopted by this plan.
Score unproven local candidates in offline shadow evaluation. Before enabling one in actual production routing,
qualify its frozen policy on a separately registered untouched local-qualification holdout under all v1 gates;
never hand-set `passed_v1`. A failed qualification is not tuning data. Candidate changes require fresh qualification
data or a predeclared simultaneous procedure. Reserve the final whole-cascade sample independently: qualifying a
local component does not establish the accuracy of the routing system that uses it.

## Labels, challenges and sample size
Independent reference labelling uses a written entailment rubric: supported, contradicted, or insufficient evidence.
Initial annotators see the complete original claim and evidence, but neither the system answer nor the original
benchmark label. Separate annotators label, then adjudicate disagreements with evidence. They must be independent
of the system whose accuracy is being assessed; two prompts to the same system do not supply that independence.
Where qualified independent adjudication is unavailable, labels and resulting certification remain provisional.
Ambiguous cases remain recorded, with pessimistic sensitivity results. Preserve raw labels and audited results.
Annotator agreement does not make labels infallible: record adjudication evidence and residual label uncertainty,
and qualify the resulting accuracy claim as agreement with that independently established reference.

Use established contrastive/grounding benchmarks only after checking their licences, partition provenance and model
training contamination; public leaderboard scores never substitute for our measurements. New hard challenges cover
negation and quantifier scope, entity/number/date swaps, attribution, conditional reasoning, partial/multiple facts,
conflicting sources, long evidence, missing decisive context and instructions embedded in evidence. Include valid
support controls so a reject-everything rule is exposed. Development challenges and sealed final challenges are
separate. A challenge opened to guide a repair is thereafter diagnostic, not unseen confirmation.

Plan for approximately 2,000 independent stated decisions as a **power-planning scenario**, not a stopping rule or
an authorised large subscription batch. A separate pilot supplies acceptance-rate and error-rate estimates. Use
those to choose a fixed total number of input units, including allowance for the random accepted count, and a fixed
call/time budget before opening confirmation. Publish the power calculation and resource plan before execution.
Keep subscription development small; no large paid batch is started by this document.

The existing 381-perfect-call minimum remains correct. In the fixed-count planning example, 2,000 stated decisions
allow at most 11 false passes under both v1 Wilson and the exact supplement. This does not permit collecting until
2,000 good decisions, excluding failures or extending a failed confirmation. At the registered end, insufficient
accepted counts means insufficient evidence. Changing the candidate after seeing final outcomes requires fresh data.

All six v1 gates retain their meanings: stated bound >= 0.99, block bound >= 0.95 where blocks are made, zero wrong
stress passes, ECE <= 0.05, pre-escalation local coverage >= 0.50 and quiet local p95 <= 250 ms. Report end-to-end
latency and escalation coverage separately. The 95% confidence statement concerns stated correctness under the
registered sampling procedure; six passing gates do not automatically give a joint 95% guarantee for every metric.
Broader or simultaneous claims need a separately justified statistical design.

## Immediate implementation order
1. Actual-cascade runner and independent statistics/identity/failure tests, entirely offline first.
2. Source/licence registry, independent development fixtures and sealed-data/label workflow.
3. Small authenticated diagnostic baseline and separate audit; independent deterministic repairs and production/fit parity.
4. Bounded open-verifier/complete-support experiments, separate risk calibration and one frozen candidate.
5. A preregistered resource-sized confirmation, then versioned deployment monitoring.

Every change has one small tested commit and PR; merge only after green CI. Record measured outcomes in `docs/bench/`
and Done / Open / Next in `HANDOFF.md`. No frozen data or standard bar is weakened to meet a deadline.
