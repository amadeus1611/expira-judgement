# Research: what would establish the general layer's 99% claim?

## Problem
The owner requests continued research with the most rigorous tests. Standard v1 remains unchanged: its stated-call
95% Wilson lower bound must reach 0.99, alongside the block, stress, calibration, coverage and latency gates.
Round 3's best local bound was 0.9438; agreement was 0.9179. Neither is an adopted 99% solution.

There are two different problems: improving the checker and establishing that an improvement generalises. More rows,
agreement, an exact citation, or a model's reported confidence do not by themselves solve either problem. This note
proposes experiments, not a new model recommendation. Measurements are in
`docs/bench/2026-10-11-rigorous-99-baseline.md`; the executable acceptance procedure to build next is specified in
`docs/validation/general-99.md`.

## Sources
Read on 2026-10-11:

- Angelopoulos et al., *Learn then Test: Calibrating Predictive Algorithms to Achieve Risk Control*:
  https://huggingface.co/papers/2110.01052 (original abstract, excluding the generated summary).
  Official authors' code and overview:
  https://github.com/aangelopoulos/ltt/blob/main/README.md,
  https://github.com/aangelopoulos/ltt/blob/main/core/pfdr.py,
  https://github.com/aangelopoulos/ltt/blob/main/core/bounds.py.
  The selective-risk example counts errors among selected predictions, computes binomial p-values, and applies
  Bonferroni correction across candidate thresholds. An in-sample Wilson scan alone is not a simultaneous guarantee.
- Angelopoulos et al., *Conformal Risk Control*: https://huggingface.co/papers/2208.02814 (original abstract);
  https://github.com/aangelopoulos/conformal-risk/blob/main/README.md.
  The overview describes controlling expected monotone loss on a new point. That result is not automatically a
  high-confidence bound on conditional error among accepted claims; the loss, assumptions and guarantee must match.
- Northcutt, Athalye and Mueller, *Pervasive Label Errors in Test Sets Destabilize Machine Learning Benchmarks*:
  https://huggingface.co/papers/2103.14749 (original abstract);
  https://github.com/cleanlab/label-errors/blob/master/README.md.
  Algorithmically flagged disagreements were human validated. The repository explicitly says corrections are not
  perfect and exposes annotator votes. This supports investigating labels, not automatically replacing them with a
  checker's answer. No result about VitaminC label-error prevalence follows from this paper.
- Schuster, Fisch and Barzilay, *Get Your Vitamin C! Robust Fact Verification with Contrastive Evidence*:
  https://github.com/TalSchuster/VitaminC/blob/main/README.md;
  https://huggingface.co/datasets/tals/vitaminc/raw/main/README.md.
  The dataset contains contrastive evidence from Wikipedia revisions and synthetic edits. Case grouping matters;
  this session also measured repeated text across different case IDs.
- Min et al., *FActScore: Fine-grained Atomic Evaluation of Factual Precision in Long Form Text Generation*:
  https://huggingface.co/papers/2305.14251 (original abstract);
  https://github.com/shmsw25/FActScore/blob/main/README.md.
  Atomic fact checking motivates an experiment that requires support for every part of a claim. Its biography results
  and automatic metric are not an EXPIRA accuracy guarantee.
- Tang et al., *MiniCheck: Efficient Fact-Checking of LLMs on Grounding Documents*:
  https://huggingface.co/papers/2404.10774 (original abstract). Checking each fact and multi-sentence synthesis motivates
  stress cases; published relative performance is not evidence that our exported model meets v1.
- SciPy's documented exact and Wilson binomial intervals, read directly in the official implementation:
  https://github.com/scipy/scipy/blob/main/scipy/stats/_binomtest.py.
  `proportion_ci(method="exact")` uses Clopper–Pearson; a two-sided 95% interval uses a 0.025 lower tail.

Direct arXiv and ACL Anthology requests returned proxy CONNECT 403. Full paper proofs were not read through those
routes. The original mirrored abstracts and official repository contents above were read; generated paper summaries
were excluded. The proposed guarantees still require implementation and assumption checks here.

## Licence
Read the actual source files, not recalled licence names:

- Learn then Test and Conformal Risk Control repositories: MIT, each repository's `LICENSE`.
- cleanlab/label-errors: Apache-2.0, repository `LICENSE`. Dataset licences remain separate.
- TalSchuster/VitaminC repository code: MIT, repository `LICENSE`; the tals/vitaminc dataset card declares CC BY-SA 3.0.
- FActScore repository: MIT, repository `LICENSE`. Its code licence does not establish rights to every dataset or model.
- SciPy: BSD-3-Clause, repository `LICENSE.txt`.
- Papers were read and cited; no paper text, third-party implementation, model or dataset is redistributed in this change.
  MiniCheck is a research reference only; no model licence is inferred from the paper.

## Candidates
1. **Freeze and validate the actual cascade.** Count final decisions after deterministic routing, production name
   checks, eligible local policy and escalation. Include failures and abstentions in coverage and cost. A layer's
   standalone score cannot substitute for the system's result. Newly measured layer-1 results expose a separate block
   gate failure that even perfect escalation cannot repair on these labels.
2. **Protect selection from validation.** Choose model, prompt, effort, decomposition, thresholds and agreement policy
   using development/calibration cases only. Preserve the old frozen test as diagnostics. Lock one candidate before
   collecting an untouched confirmation sample; publish failures too. If multiple candidates are validated, reserve
   fresh data for each or predeclare a finite family with simultaneous error control. Do not stop when a running
   fixed-sample interval first passes.
3. **Make the sampling unit explicit.** A case split prevents the same case ID crossing halves, but 3,095 test rows
   are only 935 cases; 21 exact and 24 normalised claim–quote pairs cross halves. Keep frozen files unchanged. A new
   confirmation design needs case/duplicate/source grouping and a declared population. A preselected single row per
   independent group can support a case-balanced claim, not automatically a row-weighted web-text claim. Merely
   replacing n with the number of groups in the row-level formula is invalid.
4. **Fit selective risk with explicit multiplicity handling.** Learn-then-Test is a candidate for a separate risk
   calibration partition after score calibration. Validate an independently written implementation against binomial
   enumeration, boundary cases and synthetic false-certification experiments before adopting it. Keep v1's Wilson
   check and all other gates, adding an exact binomial confirmation check rather than substituting a weaker bar.
5. **Check complete support.** Experiment on calibration data with an atomic claim checklist, decisive spans and a
   counterexample search for added details, scope, attribution, time, entities, quantities and polarity. Accept the
   original claim only if all required parts are supported. A decomposition that drops a clause must fail. This may
   reduce false passes or may add correlated model errors and cost; neither benefit nor 99% is measured yet.
6. **Audit labels independently.** After frozen predictions, audit all disagreements and a preregistered random sample
   of agreements. Hide the system answer and original label during initial annotation; preserve original text and
   labels. Record clearly justified corrections and unresolved cases separately, with independent adjudication where
   available. Publish raw, audited and pessimistic unresolved-case results side by side. An agent's self-audit is
   provisional, not independent validation. Never fit the policy or prompts on the audit.
7. **Separate software correctness from semantic performance.** Preserve property tests and zero missed decision-crate
   mutants. Add protocol integrity and failure tests before trusting a new runner. Fresh, independently generated
   semantic stress classes cover absent details, partial support, conflicting evidence, unsupported entities, numeric
   comparators, dates, negation, clause loss, and evidence that attempts to instruct the checker. Stress data are not
   used to tune the policy. A model-free smoke test cannot count as tested model inference.

## Decision
Adopt no new model, threshold, agreement rule or label correction from this research. Make the next whole-cascade
measurement reproducible under the protocol in `docs/validation/general-99.md`, preserving v1 and frozen splits.
The ordered work remains measurement, then separate label audit, then independently tested rule changes. The owner
selected Codex for escalation; its current managed authentication is rejected by the endpoint, so live scoring remains
blocked. One bounded synthetic probe produced no answer or usage/cost report; cost is unknown.

New confirmation data and stricter selection control are prerequisites for a defensible new 99% claim after repeated
experimentation. A VitaminC result would support a statement about that declared distribution, not all web text.
Broader deployment needs a separate, representative, independently labelled sample of its actual claim/evidence stream.
