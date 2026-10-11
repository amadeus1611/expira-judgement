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

## 2026-10-11 (later) General layer to 99%: VitaminC, name rule, Claude via subscription CLI

Owner direction: EXPIRA only until mature (engine work parked on gssc-opp-engine branch `claude/ej-learnings`, unmerged);
general web-text layer to a proven 99% first; owner directs, the agent analyses and executes; recommendations come only
from adopted habits (`docs/HABITS.md`).

### Done (branch `claude/general-99`)
- `tools/fetch_general.py`: reproducible VitaminC calibration/test splits, split by case (2,905 / 3,095 rows).
- Measured on VitaminC test: DeBERTa-v3-xsmall AUROC 0.794, best proven "stated" bound 0.9336 (298/310).
- Exact-name rule (`ej_evidence::names_in_quotes`, enforced on local judges in `adjudicate`): bound 0.9336 -> 0.9438.
- Negation pre-check measured and rejected (0.9336 -> 0.9327).
- `tools/escalation_bench.py`: layer 3 through `claude -p` (no API key), schema + cited-span check, cost per call.
  Haiku 5.5 partial: 66/68 "supports" right; ~$0.00044 per call.
- `ej-nli` loads binary checkers via `ej_labels.json`; `tools/export_onnx.py` (MiniCheck exported; its ONNX scores
  looked near-random on 40 rows and were being checked against the PyTorch original).
- `docs/HABITS.md` + `ej_habits::recommendable`: only adopted habits are recommended.

### Open (were running at this section; data in the session scratchpad, re-creatable with the tools above)
- Haiku and Opus 5.5 on the same 1,000 VitaminC test rows (seed 7): decides the 99% design (Haiku "supports" confirmed
  by Opus, both with cited spans). Needs >= 381 agreed "supports" calls with zero errors for a 0.99 bound.
- DeBERTa-v3-large fp32 on VitaminC; MiniCheck ONNX vs PyTorch check.
- Background agent PR: ej-bench zero mutants, cargo-deny, missing_docs (branch `claude/ej-ratchets`).
- Mutation-test the new name rule code; integrate the name rule into `ej-bench fit` scoring.

### Next
- Fit and log round 3 in `docs/bench/2026-10-11-round3-general-99.md`; adopt the escalation design that proves 0.99.
- Rust `CommandTransport` for `ej check` using the same CLI flags as `tools/escalation_bench.py`.

## 2026-10-11 (hand-over to Codex) Round 3 stopped at the owner's usage limit

### Done
- Round 3 logged: `docs/bench/2026-10-11-round3-general-99.md` (VitaminC; xsmall 0.9336 -> 0.9438 with the name rule;
  Haiku 277/289, Opus 185/194, agreement 164/171 on partial runs; cost per call; label-noise and cascade findings).
- Overlap placeholder removed from the `ej check` cascade (it passed 9 of 82 unsupported claims); retired habit recorded.
- Layer 1 reads bounds ("more than", "fewer than", "before", "at least" ...; `Cmp`, `bounded_quantities`) and bare years;
  a bound not met by any quote figure escalates instead of being a certain mismatch. False "mismatch" on true claims in
  the 1,000-row sample: 145 -> 33; "backed" unchanged 13/13. 100 tests pass.
- MiniCheck ONNX export verified against PyTorch (1e-6); scores weakly on a 40-row sample.
- All background runs stopped to save the owner's usage; agent work saved on branch `claude/ej-ratchets` (2 finished
  commits + 1 WIP commit, unverified).
- Cold-start prompt for Codex rewritten: `docs/handoffs/codex-start.md`.

### Open / Next
- See `docs/handoffs/codex-start.md` section 4 (ordered next steps). Mutation testing of the new layer-1 bound and
  name-rule code was not run yet; run it first (`cargo mutants -p ej-evidence -p ej-judge`).
