# Study-text integration around the existing engine

## Problem
Study notes and generated explanations need a usable path from text plus a source
document to per-claim EXPIRA findings. The owner's scope excludes all decision-block
tuning. A wrapper must preserve evidence and original findings, surface uncertainty,
and distinguish a completed flagged check from a broken invocation.

## Sources
- Existing production contracts read locally: `crates/ej-cli/src/main.rs`,
  `crates/ej-core/src/lib.rs`, `crates/ej-evidence/src/lib.rs`,
  `crates/ej-bench/src/lib.rs`, `crates/ej-bench/src/command.rs`, and `HANDOFF.md`.
  `ej-bench` measures labelled cases and out-of-process judges; `ej check` runs
  production evidence checking and controls local-model policy eligibility.
- Rust 1.97.0 process source, retrieved for literal argument handling and process
  completion semantics:
  https://github.com/rust-lang/rust/blob/1.97.0/library/std/src/process.rs
- tempfile's upstream licences, retrieved and read:
  https://github.com/Stebalien/tempfile/blob/master/LICENSE-MIT and
  https://github.com/Stebalien/tempfile/blob/master/LICENSE-APACHE
  The already locked tempfile 3.27.0 provides a private temporary claims file and
  cleanup without inventing a shared filename scheme.

## Licence
Implementation is fresh private Rust code. No model, study corpus, third-party
implementation or calibration data is added. tempfile declares MIT/Apache-2.0,
already permitted by the repository's supply-chain rules; the lockfile adds only
the new workspace package. Source-document rights remain those of the caller's
selected documents.

## Candidates
1. Duplicate the cascade and judge setup in a new CLI. Rejected because policies
   and production routing could diverge.
2. Call the existing `ej check` binary in one batch. Chosen: prepare ordinary
   `Claim`/`Evidence` input and retain its `Finding` outputs and eligibility gate.
3. Add generative claim extraction or retrieval. Deferred: unnecessary for this
   thin integration and could omit or rewrite factual statements. Mechanical
   candidates retain all non-whitespace content; reviewed one-claim-per-line
   input supplies explicit boundaries when needed.

## Decision
Add `ej-study` outside the decision crates, with sentence/line candidates and
original byte spans, explicit source/checking dates, a verbatim source snapshot,
and flags computed by the existing `Verdict::passes()` method. Validate process
completion, IDs and output schema before producing a report. Check setup failures,
partial results, unchanged source forwarding and all verdicts with integration-only
tests. Compare the wrapper's real results with direct `ej check` on synthetic
examples. This measures integration behaviour, not judgement quality or Security+
accuracy. Leave all decision files, thresholds, calibration data and old tests intact.
