# Handoffs

Read this first. Each session appends its own section at the end (newest last): **Done / Open / Next**.
Rules carried over: repo stays private; fresh Rust only (nothing copied from gssc-system/, client data, memory or pricing);
no invented figures; one small tested change per commit.

## 2026-10-11 Restore scaffold

### Done
- Restored the scaffold from the transfer bundle parked in gssc-opp-engine (`memory/handoffs/expira-judgement/`).
  Commit `d0bf0cf` kept intact, merged into this repo's initial commit; `cargo test`: 16 passed.
- Added this file.

### Open
- Delete the transfer copy `memory/handoffs/expira-judgement/` from gssc-opp-engine once this lands.
- Roadmap work below.

### Next
- a. Claude escalation client (Messages API) behind `Judge`; every ruling stored as a teacher pair (ej-habits).
- b. Bake-off harness: LAYA vs DeBERTa NLI vs small constrained model, ONNX in Rust; no winner without measured numbers.
- c. Harden ej-evidence: ranges, currencies, negation, dates.

## 2026-10-11 (same session) Escalation, bake-off harness, layer-1 hardening

### Done
- a. `ej-judge::claude::ClaudeJudge` (Messages API behind a `Transport` trait; reply must be JSON with a reason, otherwise Unsure).
  Rulings collected as `TeacherPair`s; `ej-habits::{append_pairs, load_pairs}` (JSONL). `ej check --claude [--model] [--pairs]`
  uses a ureq transport (`ANTHROPIC_API_KEY`, default model `claude-sonnet-5-5`, override with `--model`/`EJ_CLAUDE_MODEL`).
  **Not run against the live API**: no key in the container. Only offline tests with canned replies.
- b. `ej-bench` (harness + CLI + `CommandJudge` line protocol for out-of-process judges such as LAYA) and `ej-nli`
  (DeBERTa cross-encoder via `ort` 2.0.0-rc.10 on CPU). First measured run in `docs/bench/2026-10-11-mnli-overlap-vs-deberta.md`:
  public MNLI, in-distribution for that model, so it is **not** the decision run. No winner claimed.
- c. ej-evidence: calendar-valid dates, future-dated evidence unusable, canonical currencies (₱/$/€/US$/glued codes/words),
  ranges (both ends, shared unit), dates and months in text, negated matches no longer backed.
- Tests: 35 pass (+1 model smoke test that runs only with `EJ_NLI_DIR` set).

### Open
- gssc-opp-engine PR removing `memory/handoffs/expira-judgement/` (https://github.com/amadeus1611/gssc-opp-engine/pull/128).
- LAYA in the bake-off needs a shim speaking the `CommandJudge` protocol (see `crates/ej-bench/src/command.rs`); LAYA lives in gssc-opp-engine.
- Small constrained generative model for the bake-off is not wired.
- No labelled ledger cases yet; the teacher-pair file from `--claude` runs is meant to become that set.
- `ort` is a release candidate pinned exactly; it downloads ONNX Runtime binaries at build time.

### Next
- Run `ej check --claude` on a few real claims once a key is available; check the pairs file.
- Write the LAYA shim and rerun the bake-off on ledger cases; add the constrained model.
- Habit store on disk + reporting; tier log (README roadmap 4).
