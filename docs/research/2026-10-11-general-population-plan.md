# Research: making the general web-text claim testable

## Problem
The owner asks the agent to own the validation design and architecture decisions. Hard benchmarks find failures,
but their challenge distribution cannot establish deployment accuracy. Existing test inspections, correlated rows
and label audits cannot provide a fresh confirmation. We need a declared population, independently sampled units,
independent reference labels and a fixed confirmation design as well as better decision logic.

This is a plan, not a successful experiment or adoption of a new judge. Standard v1 is unchanged. Its 99% gate
concerns correctness among stated decisions; it does not promise 99% accuracy across every input or all future web text.

## Sources
- The repository's measured baseline: `docs/bench/2026-10-11-rigorous-99-baseline.md`. Deterministic false blocks
  prevent the block gate even with perfect later decisions; a disclosed conditional challenge produces a false pass.
  These findings prioritise independent development experiments, not fitting on the inspected test examples.
- Angelopoulos et al., [Learn then Test](https://arxiv.org/pdf/2110.01052). The full PDF became accessible after the
  earlier research note. Read selected sections 1.1 (iid assumptions), 2.1/Theorem 1 (valid p-values and family-wise
  control), 3.2 (error conditional on accepting), Appendix C (fixed-sequence proof) and Appendix D (separate split
  for learning a testing order). This is not a claim to have read every page. Selecting within a simultaneously
  certified set is allowed; learning the family/order on the testing labels is not justified by that theorem.
  [Authors' implementation](https://github.com/aangelopoulos/ltt/blob/main/core/pfdr.py) was read in the prior note.
- [SciPy binomial interval source](https://github.com/scipy/scipy/blob/main/scipy/stats/_binomtest.py), read in the
  prior note, supplies an independent reference for Wilson and exact confidence intervals. Sample-size planning
  must also account for the probability of passing when the candidate is genuinely better than the boundary.
- Northcutt et al., [label-error investigation](https://github.com/cleanlab/label-errors/blob/master/README.md), and
  Min et al., [FActScore](https://github.com/shmsw25/FActScore/blob/main/README.md), read in the prior note: blind
  label adjudication and atomic completeness are experiment candidates. Model agreement is not independent truth.
- Existing procedure and source/licence record: `docs/validation/general-99.md` and
  `docs/research/2026-10-11-rigorous-99-validation.md`.

## Licence
Re-read the saved source LICENSE files: Learn then Test and FActScore code are MIT. The previously read label-errors
code is Apache-2.0 and SciPy BSD-3-Clause. Paper ideas are cited; no paper text or implementation is copied here.
Code, weights, training data and evaluation text need separate licence records before use. Public availability or a
dataset wrapper's licence does not establish rights to underlying web documents. Existing VitaminC code is MIT,
while its dataset card declares CC BY-SA 3.0; attribution/share-alike obligations cannot be ignored for derivatives.
No new corpus or model is introduced by this change. Missing or incompatible permissions exclude that material from
reuse; they also constrain the population we can honestly claim to have sampled.

## Candidates
1. Certify from the hardest existing benchmark. Rejected for a general claim: challenge prevalence differs from
   ordinary inputs, existing outcomes influenced research and contrastive rows are dependent.
2. Draw a broad licensed reference mixture, specify its document/claim sampling distribution, and validate a frozen
   candidate on a sealed sample. Feasible preparation now; the resulting claim must name that reference distribution.
3. Add prospective independent sampling from the actual deployed claim/evidence stream, with a fixed time window,
   inclusion protocol and labels. Best basis for a deployment claim, but no such stream has been measured here.
   Do not invent traffic frequencies or call an equal-source mixture representative of unknown traffic.
4. Repair deterministic logical errors, then compare complete-support checks and specialised local verifiers on
   independent development data. Fit acceptance policies with selection-aware risk calibration. These are unproven
   architecture candidates; keep only measured improvements under the existing bars and routing rules.

## Decision
Use candidates 2 and 3 for different, explicitly named claims, alongside independent hard challenges for robustness.
Execute the staged plan in `docs/validation/general-99-roadmap.md`. Start with production runner integrity, then
independent development repairs and experiments. Keep the existing ordered label audit separate and never fit on it.

Prefer one frozen candidate for the final run. Plan sample size for power, not merely the error-free minimum:
the independently checked calculation in `docs/bench/2026-10-11-general-population-plan.md` gives a 68.30% chance of
passing with 381 independent stated decisions at a hypothetical true accuracy of 99.9%. This is a design calculation,
not measured EXPIRA accuracy. Actual total input count and call budget require a separate pilot acceptance estimate.
