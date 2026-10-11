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

## 2026-10-11 (same session) Standard, calibration, cost, Claude-only roster, engineering ratchet

### Done
- Standard v1 as code (`crates/ej-bench/src/standard.rs`): pass decisions >= 0.99 and block decisions >= 0.95 as 95% Wilson
  lower bounds on held-out data, 0 stress wrong passes, ECE <= 0.05, local coverage >= 0.50, p95 <= 250 ms. Unmeasured = failed.
- `ej-bench score` / `fit`: raw scores once, Platt calibration and bounded thresholds fitted on a calibration split,
  verdicts on test and stress, policy file for `ej check --nli --policy` (a judge is used only if it passed).
- Round 1 measured (`docs/bench/2026-10-11-round1-laya-vs-deberta-calibrated.md`): nothing passes; binding gate is the
  0.99 bound on "stated" calls. DeBERTa-v3-xsmall AUROC 0.956 vs LAYA 0.875; LAYA passes 51/200 negated claims at the
  engine's 0.6 bar and its current bar is right on 68% of "stated" calls on that data.
- Claude escalation: schema-constrained reply with a cited span checked against the quotes, refusals Unsure, fallbacks on,
  Opus 5.5 at low effort by default; usage per call logged and priced from `config/prices.json` (sourced, dated).
- Claude-only roster (`config/roster.json`, `ALLOWED_MODELS`): Haiku 5.5, Sonnet 5.5, Opus 5.5; anything else is rejected.
- Delegation: start tier per task kind by lowest expected cost from measured pass rates and attempt costs (`ej-delegate/src/cost.rs`).
- Layer 1 fixes from the stress run (detached commas, ordinals, decades); overlap judge abstains on negation.
- Harness: `CLAUDE.md`, the `iterate` skill (research open sources on any problem, one change at a time, keep only if
  measured better, unattended guardrails), research notes checked by `cargo test`, CI with fmt + clippy (all + pedantic,
  `-D warnings`) + tests.

### Open
- Round 2 scoring (cross-encoder DeBERTa-v3-base, MoritzLaurer DeBERTa-v3-large mnli-fever-anli-ling-wanli q8) was running
  at the end of this section; data and scripts live in the session scratchpad, not the repo (public data, not redistributed).
- No configuration passes standard v1 yet. A 0.99 bound needs ~460+ held-out "stated" calls with no error: more labelled
  cases (teacher pairs from `ej check --claude`) are the main lever, plus a stronger verifier.
- `ej check --claude` has never run against the live API (no key in the container).
- LAYA in the owner's engine: its "stated" bar is not safe on this evidence; changing the engine is the owner's call
  (gssc-opp-engine charter), so it is reported, not changed.
- No scheduled runs, by the owner's choice: the iterate loop runs inside the owner's sessions, filling gaps around their requests.

### Next
- Fit round 2, log it, adopt or reject; then MiniCheck (needs ONNX export) and a Haiku-first escalation cascade measured on teacher pairs.
- Supply-chain gate (cargo-deny: licences and advisories) and `missing_docs`, as the next engineering ratchets.

## 2026-10-11 (same session) Test rigour, round 2, hand-over to Codex

### Done
- Property tests (proptest): layer 1 (7 invariants x 2,000 cases), acting statistics, routing.
- Mutation testing (cargo-mutants): zero missed in ej-core, ej-evidence, ej-judge, ej-delegate, ej-habits; CI job enforces it.
  It found and we fixed: a claimed day backed by a month-only quote; a figure tolerance loose enough to equate
  1,000,000,001 with 1,000,000,000; a magic acting bar replaced by `Judge::self_gated()`.
- Toolchain pinned (1.97.0) after CI's newer clippy diverged.
- Round 2 (public FEVER + MNLI, same splits): cross-encoder DeBERTa-v3-base rejected (AUROC 0.958 vs 0.956, ~2.6x slower).
  MoritzLaurer DeBERTa-v3-large mnli-fever-anli-ling-wanli: the 8-bit ONNX build is damaged (MNLI AUROC 0.935) while the
  fp32 build scores 0.990 on MNLI (median ~358 ms under load). Mean of all calibrated judges at target 0.90: 93/94
  stated calls right, 0 stress wrong passes, pass bound 0.942 (not yet 0.99).
- `AGENTS.md` so Codex (or any agent) follows the same rules, skill and standards.

### Open
- fp32 large model: full cal/test/stress scoring was running at hand-over (scratchpad, not in repo); fit it and its
  ensembles, measure latency on a quiet machine, log as round 2 in `docs/bench/`.
- ej-bench lib not yet mutation-tested (376 mutants incl. CLI); add to the CI mutants job once at zero.
- Standard v1 still not passed; more held-out labelled ledger cases (teacher pairs) remain the main lever.

### Architecture decision (owner, end of session)
- The repo is a bring-your-own-model architecture people plug their own subscriptions and models into
  (`docs/ARCHITECTURE.md`). "Claude only" is the owner's default configuration, not a limit. No API key needed to develop.
- Start prompt for Codex or any agent: `docs/handoffs/codex-start.md`. Round 2 is logged in `docs/bench/`.

### Next
- Pluggable escalation transport (subscription CLI) and configurable allowed-model list; then then MiniCheck (ONNX export) and a Haiku-first escalation cascade measured on teacher pairs.

## 2026-10-11 (Claude, branch claude/ej-ratchets) Engineering ratchets

### Done
- `ej-bench` lib at zero missed mutants (CLI `main.rs` excluded): 78 missed + 1 timeout before, 0 after. `Platt::fit`
  rewritten with bounded loops (fixed Newton iterations, step chosen from a fixed ladder including 0, so loss never rises
  and no mutant can loop forever); new tests check the Newton direction against finite differences, the two-point
  optimum, loss/logit/apply hand values, AUROC over all pairs, ECE bins, selective coverage, percentile ranks, latency
  units, every command-judge ruling word, child cleanup on drop, and the block gate when a judge never blocks.
  Added to the CI mutants job, `AGENTS.md` and `docs/handoffs/codex-start.md`.
- Supply-chain gate: `deny.toml` + `cargo-deny-action@v2` in the CI `check` job (note: `docs/research/2026-10-11-supply-chain.md`).
  Licences: MIT, Apache-2.0, BSD-3-Clause, ISC, Unicode-3.0, CDLA-Permissive-2.0. No vulnerabilities; RUSTSEC-2024-0436
  (paste unmaintained, via tokenizers) ignored with its reason.
- `missing_docs = "deny"`: every public item in every crate documented.

### Open
- The paste advisory ignore should go when tokenizers drops paste.
- Duplicate crate versions only warn (base64, getrandom, r-efi, rand_core, smallvec, syn, windows-sys).

### Next
- Tighten `bans.multiple-versions` to deny once the duplicates are gone; consider cargo-llvm-cov as the next ratchet.
