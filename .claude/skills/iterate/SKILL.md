---
name: iterate
description: Improve the EXPIRA Judgement decision block (local judges, calibration, thresholds, layer-1 checks, Claude escalation) against the standard in crates/ej-bench/src/standard.rs. Use whenever a judge, model, threshold, check or escalation setting is added or changed, when new labelled data or teacher pairs arrive, or when the standard is not yet passed.
---

# Iterate on the decision block

The loop improves the repo's harness (checks, judges, calibration, data, docs), never a model's weights in place.

1. **Baseline.** Score every candidate judge on the frozen splits and fit:
   `ej-bench score <split>.jsonl --out <split>.<judge>.scores ...` then
   `ej-bench fit --cal <cal scores> --test <test scores> --stress <stress scores> --write-policy policy.json`.
   Exit 0 means at least one configuration passes the standard. Record the verdicts.
2. **Find the binding gate.** Read which gate fails (`pass_lower_bound`, `block_lower_bound`, `stress_wrong_passes`, `max_ece`, `min_coverage`, `max_p95_ms`) and why: too few held-out cases, a failure class (figures, negation, dates), a slow model, poor ranking (AUROC).
3. **Change one thing** aimed at that gate, in its own commit with a test:
   - data: more labelled cases (Claude teacher pairs load directly as cases), a new stress class;
   - layer 1: a deterministic check that removes a failure class before any model sees it;
   - a judge: a new model (ONNX in Rust where possible), a different question wording, an ensemble;
   - fitting: target, minimum sample, calibration method;
   - escalation: Claude model, effort, prompt, schema.
   Research current options first (recent models and papers); prefer what can be measured here.
4. **Re-measure on the same splits.** Never fit on the test split. Never tune against the stress set and then report it as unseen; add a fresh stress class instead.
5. **Keep or revert.** Keep only if the standard verdict improves, or with the same verdict the coverage at the same risk rises, or latency falls with nothing else worse. Otherwise revert and log why.
6. **Log.** Append a dated run to `docs/bench/` (data, machine, commands, every gate value) and a section to `HANDOFF.md`.

If the standard cannot be met with what is available, say exactly which gate binds and what would move it (usually more held-out labelled cases); do not lower a bar to pass.
