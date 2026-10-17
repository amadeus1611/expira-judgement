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

## 2026-10-11 (Codex) Cloud setup and ratchet verification

### Done
- Loaded the cold-start files in section 1 order; pinned-toolchain fmt/clippy/workspace tests passed before work.
- Installed Rust 1.97.0, cargo-mutants 27.1.0 and cargo-deny 0.20.2. The default ONNX Runtime download was proxy-blocked;
  the supported native-library override uses a SHA-256-verified ONNX Runtime 1.22.0 PyPI wheel. Setup lives outside
  the checkout; sources and licence checks are in `docs/research/2026-10-11-cloud-runtime.md`.
- Continued the three ratchet commits on `codex/ej-ratchets`, merging current main without losing either history.
  Both fmt/clippy and workspace tests passed on the reconciled code (117 reported passing tests; model smoke returned
  early without EJ_NLI_DIR). CLI smoke verified backed / mismatch / unsure and the expected failing-findings exit status.
- Full mutation run exited 0: 856 generated, 794 caught, 62 unviable, zero missed, zero timeouts. ej-bench library:
  292 generated, 275 caught, 17 unviable, zero missed. Current bounds and name-rule code were included.
- cargo-deny passed all checks with the existing duplicate-version warnings. CI includes ej-bench with main.rs excluded;
  updated the cold-start pre-push command accordingly. Results: `docs/bench/2026-10-11-codex-ratchets.md`.
- Exercised the complete installer twice; saved install_script and start_skill in the environment draft. Saved required
  GitHub API and Hugging Face domain additions while preserving package-manager presets. No publication is claimed.
- Pushed `codex/ej-ratchets` and opened https://github.com/amadeus1611/expira-judgement/pull/3 through the platform
  GitHub CLI route. That route also reads CI successfully despite native API CONNECT requests returning 403.
  The PR carries the verified ratchets; merging requires green CI. Inspect its current state through `gh pr view 3`.
- Owner clarification: use the in-house **Codex CLI/subscription for escalation in Claude's place**. This overrides
  the historical Claude-only default for this workflow. Built-in codex-cli 0.159.2 reports ChatGPT authentication;
  no new key is needed. Sources and bounded smoke results: `docs/research/2026-10-11-codex-subscription.md`.
- Diagnosed Codex exec startup with a file-only trace: its installation_id needs a write in the read-only managed
  home. Supported SQLite/log overrides plus an automatically approved invocation reached the subscription endpoint
  without copying credentials or changing CODEX_HOME. Saved chatgpt.com in the network draft as well.

### Open
- Native GitHub API requests are proxy-blocked (CONNECT 403); use the working platform `gh` route for PR operations.
- Hugging Face requests are proxy-blocked, so VitaminC rebuilding/model download and whole-cascade scoring are unrun.
- Claude CLI is absent; ANTHROPIC_API_KEY and EJ_NLI_DIR are unset. No model inference or live escalation was validated.
  Claude is not required for the owner's newly selected Codex route.
- One synthetic Codex smoke invocation encountered subscription-endpoint CONNECT 403 and timed out at 120 seconds.
  No structured reply or usage/cost report; Codex usage/cost are unreported. No held-out rows were sent. Claude calls: 0;
  Claude cost: $0. The general-layer 99% goal remains unproven; no standard bars or frozen data changed.

### Next
- Merge PR #3 only after CI is green. Review/save the environment draft in settings and publish the prepared
  environment; recheck actual Hugging Face access.
- Recheck authenticated Codex exec after chatgpt.com egress is active, using the approved runtime-write route if needed.
  Then add and test the Codex subscription adapter for tools/escalation_bench.py; retain schema/cited-span checks and
  report absent cost as unknown. Reuse existing authentication; do not request a Claude key or credentials in chat.
- Resume cold-start section 4 step 2: rebuild frozen VitaminC splits, measure the whole cascade using only proven local
  judges and small escalation runs, report raw results and observed cost, then do the label audit separately without fitting on it.

## 2026-10-11 (Codex) Rigorous research for the 99% goal

### Done
- PR #3 merged after green CI at `7ce64571c688088577d930e89976254eb3538f06`. Refreshed the cold-start instructions;
  baseline fmt/clippy/full tests passed before research. All existing bars, frozen data and labels remain unchanged.
- Read original research abstracts and authors' source/licence files on Learn-then-Test, conformal risk control,
  independent label audits, atomic checking and exact binomial intervals. Recorded hypotheses and limitations in
  `docs/research/2026-10-11-rigorous-99-validation.md`; wrote `docs/validation/general-99.md` as a specification,
  not a claim that a new runner or model has passed.
- Hugging Face access now works. Rebuilt both VitaminC files twice byte-identically outside the checkout. Test has
  3,095 rows but only 935 case IDs; no shared case IDs, yet 21 exact / 24 normalised claim–quote pairs cross cal/test.
  Preserve the frozen files; grouping by case alone does not establish independence or absence of overlap.
- Measured production layer 1 on all test rows and the historical seed-7 sample without fitting or inspecting error
  examples. Full test: 25/25 backed, nominal pass bound 0.866804; 119 true-labelled claims blocked. Even perfect later
  decisions give a maximal block bound of 0.914991 on these labels. Sample reproduces 13/13 backed and 33 false blocks.
  Results, hashes, commands and independent sample-requirement calculations: `docs/bench/2026-10-11-rigorous-99-baseline.md`.
- One bounded synthetic Codex probe now reaches the endpoint but fails HTTP 401 / refresh after logout/account change.
  No answer or usage/cost report: cost unknown. No held-out subscription calls; no Claude calls.
- A directed logical challenge found a false stated call on an unestablished conditional (`If the alarm is armed,
  the gate is locked`). Positive and explicit-refutation controls behaved correctly. This disclosed engineering
  counterexample is not a prevalence estimate or an unseen stress set; no code was fitted to it.
- A fresh full mutation run exited 0: 856 generated, 794 caught, 62 unviable, zero missed, zero timeouts. fmt/clippy,
  workspace tests and cargo-deny passed. Passing software tests do not establish semantic 99% accuracy.

### Open
- The general layer still has no v1 pass. Whole-cascade live scoring remains unrun: managed Codex authentication needs
  restoration by the platform, and the tested subscription adapter is not built. Do not ask for credentials in chat.
- Existing repeated test inspections and correlated/overlapping examples do not support a fresh 99% confirmation.
  New validation requires a frozen candidate, untouched grouped data, declared sampling population and stopping rule.
- The new protocol is a research specification; runner integrity, exact statistics/selection tests, live inference,
  fresh stress, ECE, eligible local coverage, quiet latency and independent label adjudication are still outstanding.
- Full-paper research hosts and current startup findings are saved in the environment draft, not published. The
  reproducibility habit now distinguishes grouping from measured overlap; the case-ID-only assumption is retired.

### Next
- Continue ordered step 2: restore the managed Codex route, test the benchmark adapter and whole-cascade runner
  failure/identity/routing behavior, then start with at most 10 calibration cases and report actual usage/cost.
- Separate label audit after frozen predictions; raw and audited scores together, unresolved corrections provisional.
  Do not fit on test, stress or audit. Later layer-1 changes must use independent development cases and meaningful tests.
- Before any new 99% claim, implement and verify the protocol's untouched confirmation procedure. Broader web-text
  claims require a representative independent deployment sample, beyond VitaminC.

## 2026-10-11 (Codex) Strict subscription diagnostic adapter

### Done
- PR #4 merged after all four exact-head check/mutants jobs were green, at
  `78426f97071eb0e4470b390a333a4708a6dae6d8`. Loaded the live instructions and passed baseline fmt/clippy/tests
  before this change. Researched the actual installed Codex release's protocol, configuration and tool emission.
- Added `tools/codex_adapter.py`: one bounded ephemeral Codex call through the existing subscription, strict
  process/turn/schema/final/citation checks, production binary32 confidence gate, usage retained on rejected
  completed replies, missing prices unknown, no automatic retries. It is a diagnostic adapter, not a production judge.
- Measured the actual legacy script on synthetic fake-process fixtures: it acted despite exit 1 and confidence
  0.79, and reported absent price as $0. The new adapter rejects these faults. The legacy script remains historical;
  its results cannot establish new cascade claims. Recorded the bad habit in Retired.
- Added 22 offline tests including real fake CLI processes, public CLI invocation, timeout descendant termination,
  type-sensitive final JSON identity, item lifecycle, independent confidence boundaries and failed-call accounting.
  Added CI enforcement of the tests and 15 targeted Python fault injections, all caught. The first injection run
  exposed a missing negative cache-write-token test; fixed the test gap. This is not exhaustive Python mutation coverage.
- Full Rust mutations: 856 generated, 794 caught, 62 unviable, zero missed/timeouts. Workspace tests 117 passed;
  fmt/clippy and cargo-deny passed. ONNX smoke still skips without model weights. Decision code, standard bars,
  policies and frozen data are unchanged. Research/measurements: `docs/research/2026-10-11-strict-codex-adapter.md`
  and `docs/bench/2026-10-11-strict-codex-adapter.md`. This change lands only through a PR after green CI;
  GitHub records its current merge state.
- Two bounded synthetic subscription attempts; final diagnostics identify managed authentication failure.
  Neither returned completed-turn usage or usable JSON. Tokens/cost unknown, not $0; labelled subscription calls 0.
  No further probe loop. Credentials stay in managed CODEX_HOME; do not request them in chat.

### Open
- No Codex model quality or whole-cascade v1 pass is established. The managed subscription needs platform restoration.
  A logged-in status is insufficient; only a completed endpoint request validates it.
- Exec omits some tool events. Supported tool disables, empty temporary CWD and read-only sandbox do not attest full
  isolation; adapter outputs `isolation_validated: false`. Do not use them for a 99% confirmation yet.
- Whole-cascade runner/identity manifest, production Rust subscription transport, independent label audit, local
  eligibility/parity, fresh stress, ECE, pre-escalation coverage, quiet latency and untouched confirmation remain open.
  The existing false blocks and conditional false pass are still unfixed; no held-out examples were used to tune here.

### Next
- Continue ordered step 2 with a tested whole-cascade runner around **actual** `ej check` decisions: hash dataset,
  code, policy, prompt/schema and sampling identity, reject unsafe resumes/joins, retain failures and unknown costs.
  Leave unproven local judges disabled. Build offline while platform authentication is restored.
- After endpoint and isolation verification, start at most 10 calibration cases serially and report observed
  usage/cost. Freeze predictions before a separate label audit; raw and audited scores together, no fitting on audit.
- Follow `docs/validation/general-99.md` before any 99% claim. Pick model/policy only from permitted development
  measurements, then use untouched grouped confirmation; do not declare an unmeasured model optimal.

## 2026-10-11 (Codex) Population, architecture and confirmation execution plan

### Done
- Owner asks the agent to make the design decisions and reinforce validation, rather than asking the owner to supply
  domain expertise. Loaded the cold-start files and passed baseline fmt/clippy/locked workspace tests before editing.
- Wrote `docs/validation/general-99-roadmap.md`, linked from the existing protocol: initial English supplied-evidence
  scope, licence-cleared source frame, reference-mixture versus prospective traffic claims, explicit independent
  sampling units/design, separate development/score/risk/final/stress partitions and independent blind labels.
- Assigned each stage a completion gate: actual production runner, data/labels, diagnostic baseline and separate
  audit, independent deterministic repairs, complete-original-claim/open-verifier experiments, selection-aware risk
  calibration, fixed untouched confirmation and versioned deployment monitoring. All architecture candidates remain
  unproven. The owner's tuning direction permits separately versioned, licence-cleared candidate artifacts using
  development/training data; installed weights and test/stress/audit data stay untouched.
- Read selected full Learn then Test paper passages on assumptions, selective risk and simultaneous control; re-read
  saved source licences. Research: `docs/research/2026-10-11-general-population-plan.md`. No copied implementation,
  new model or corpus is introduced. Licence rights to underlying texts must be checked separately from wrappers.
- Independently calculated fixed-count power: 381 perfect stated calls have only a 68.30% chance of passing at a
  hypothetical true 99.9% accuracy. A 2,000-independent-stated planning scenario allows 11 errors under both unchanged
  Wilson and the exact supplement. Rational score inversion and 70-digit binomial recurrence agree with the binary64
  calculations. These are design examples, not measured accuracy, stopping rules or authorised large paid batches.
- Final fmt/clippy/locked workspace tests passed (117 reported Rust passes; ONNX smoke still returns early without
  weights). Python tests: 22 passes; targeted Python faults: 15 caught, zero missed/unviable/timeouts. cargo-deny passes
  with the existing duplicate warnings. Bench/reproduction: `docs/bench/2026-10-11-general-population-plan.md`.
- Full Rust mutation run exited 0: 856 generated, 794 caught, 62 unviable, zero missed, zero timeouts. The change
  follows the required PR/green-CI merge process; GitHub records its current status.
- No new subscription/model calls. Historical authentication-failure costs remain unknown. Decision code, v1 bars,
  policies, labels and frozen data are unchanged; this plan makes no semantic improvement or certification claim.

### Open
- General-layer 99% remains unproven; neither hard benchmarks nor passing software checks establish a general
  population guarantee. A reference mixture does not establish representativeness of unknown production traffic.
- Actual-cascade runner, licence/source registry, new independent development/confirmation/stress data and qualified
  independent reference adjudication are not implemented by this documentation change. Where independent labels
  or sampling assumptions remain unsupported, certification stays provisional/unavailable.
- Managed subscription authentication and no-tool isolation remain unverified; no repeated connectivity probe was
  made. Existing deterministic false blocks/conditional false pass and fit/production name parity remain open.
- Final total-input size and call budget need a separate pilot acceptance estimate and a power calculation. Do not
  stop on the first passing bound, keep collecting accepted calls, or extend a failed confirmation to pass.

### Next
- Implement and test the actual-cascade runner offline first: hashes/identity, complete outcomes, safe resumes,
  production routing/eligibility, unknown-cost accounting and independent statistics. Follow the staged roadmap.
- Prepare licensed independent development fixtures and sealed sampling/label workflow. Run at most 10 serial
  calibration cases only after endpoint/isolation verification. Keep label audit separate; never fit on it.
- Research and test each deterministic repair independently, then bounded complete-support/open-local experiments.
  Freeze one candidate after separate risk calibration before resource-sized untouched confirmation. Every change
  remains one tested commit/PR with green CI before merge.

## 2026-10-11 (Codex) Offline actual production capture

### Done
- Loaded cold-start instructions in section 1 order; baseline fmt/clippy/locked workspace tests passed before work.
  Read actual CLI/routing and primary subprocess/statistics source/licences before implementation. Research:
  `docs/research/2026-10-11-offline-cascade-capture.md`.
- Added `tools/cascade_capture.py`, the first actual-cascade runner component: one offline deterministic
  pre-escalation batch, row-byte-bound companions, immutable executable/request snapshots, manifest before execution,
  strict whole-batch identities/schema/exit consistency and atomic final report. Existing directories cannot resume
  or overwrite. Failed batches retain every input as an operational failure; labels never enter production requests.
- Nominal counts/Wilson diagnostics retain abstentions separately from blocks; all-input ECE, fresh stress, quiet
  local p95, sampling independence, exact confirmation and full escalation stay unmeasured. Certification is always
  unavailable; tracked-source hashes do not attest binary/source correspondence or untracked bytes.
- Verified all 3,095 frozen test lines/case IDs against saved original source pages, creating a new hashed companion
  outside Git. Preserved both frozen files. Actual capture reproduces every prior verdict/layer/confidence/reason:
  25/25 stated, nominal bound 0.866804; 292/411 blocks, bound 0.664824; 119 false blocks; coverage 0.140872.
  No error examples were inspected for tuning. Bench: `docs/bench/2026-10-11-offline-cascade-capture.md`.
- All 19 new offline tests passed (41 Python tests total), including actual compiled CLI controls and proof that an unproven policy
  skips a nonexistent model directory. An independent passing baseline plus all 20 declared targeted runner faults
  caught; this is targeted injection, not exhaustive Python mutation coverage. CI now includes that check.
- Final fmt/clippy/locked workspace tests passed (117 reported Rust passes; ONNX smoke still returns early),
  cargo-deny passed with existing duplicate warnings. Full Rust mutations: 856 generated, 794 caught, 62 unviable,
  zero missed/timeouts. All 15 existing adapter faults and 20 capture faults caught, zero missed/unviable/timeouts.
- The initial combined Python run had one existing timeout fixture fail on its missing descendant-startup marker
  under concurrent mutation CPU load. The quiet 41-test rerun passed with the existing test unchanged. New capture
  timeout assertions require an actually spawned, dead descendant; interruption and repository-root checks pass.
- Prepared next-step primary research outside Git in `/workspace/setup/conditional-next-research.md`: HANS's
  lexical/subsequence failure taxonomy and formal/complete-claim controls support independently authored conditional
  development cases. No external dataset/weights adoption follows; separate data permissions remain unverified.
- No new subscription/model calls; attempts zero, component usage/cost unknown/null, historical failed-call costs
  remain unknown. Production decisions, v1 bars, policies, labels and frozen data are unchanged.

### Open
- Whole-cascade 99% remains unproven. This component has no eligible-local inference, Rust subscription transport,
  independent label audit, safe resume or sealed representative confirmation implementation.
- Managed subscription endpoint authentication and complete no-tool isolation remain unverified; no probe was
  repeated after environment startup. Independent reference labels and sampling assumptions remain prerequisites.

### Next
- This change lands only through a PR after exact-head CI is green; GitHub records its current merge status.
- Add structured production scores/timing and eligible-artifact integrity, then the Rust Codex route after endpoint
  and isolation verification. At most 10 serial calibration calls, no automatic retry, actual unknown-cost accounting.
- Keep any label audit blind/separate and raw/audited scores together; never fit on test/stress/audit. Independent
  development repairs must address conditional false passes and false blocks; untouched confirmation remains sealed.

## 2026-10-12 (Claude) Review of the Codex rounds; layer-1 false blocks 119 -> 4

### Done
- Reviewed PRs #3-#7 (Codex): no bar, test or decision rule was weakened. The ratchets (cargo-deny, missing_docs,
  zero missed mutants incl. ej-bench) and the cross-split text-overlap finding are kept. Its Platt rewrite re-measured:
  identical to 1e-10. Its rounds made no measured accuracy gain (documents and a Codex adapter only).
- Layer 1 (`ej-evidence`): an absent figure escalates and only a rival figure of the same kind is a mismatch (`rival`);
  decades are spans; approximate wording is `Cmp::About`; "US $" read as currency. VitaminC test: true claims blocked
  119 -> 4, block lower bound 0.66 -> 0.88, backed 25/25. Zero missed mutants on the changed functions.
- Whole local cascade measured (`tools/local_cascade.py`): test "stated" 214/220, lower bound 0.9418.
  Bench: docs/bench/2026-10-12-layer1-false-blocks.md.

### Next
- Label audit (blind, separate file, raw and audited scores side by side) on the cascade's disagreements.
- Small escalation run on the cascade's uncertain middle only (report cost); then the Rust subscription transport.

## 2026-10-12 (Claude, later) Wording and figures gates; untouched confirmation; label audit

### Done
- `ej_evidence::unstated_words` and `figures_backed`: a local judge's "stated" needs every claim content word and
  every figure in the quotes (enforced in `adjudicate`; exported per row by `ej-bench words`).
- Untouched confirmation set: `tools/fetch_general.py --confirm --pages 200` (11,393 rows after removing 7 shared
  with the frozen splits). `tools/local_cascade.py --gate --confirm` measures the frozen configuration on it.
- Raw-label results, threshold fitted on calibration only: confirmation 828/860 (lb 0.9479) -> 851/877 (lb 0.9569);
  test 214/220 (0.9418) -> 235/236 (0.9764). Bench: docs/bench/2026-10-12-content-word-gate.md.
- Label audit of every remaining wrong call (docs/audit/2026-10-12-vitaminc-stated-errors.md): 14 labels changed,
  0 of 40 sampled correct calls changed; audited confirmation 864/877 (98.52%), lb 0.9748.

### Next
- Remaining real errors: role swaps ("Catalonia is the capital of Barcelona"), negation-like words in garbled quotes
  ("immunity", "rejected"), near-synonyms ("nut"/"peanut"). Research a deterministic role-order check first.
- Escalate the gate-failing middle to Haiku in a small paid run; report cost and the combined bound.

## 2026-10-13 (Claude): DeBERTa-v3-large vs xsmall under the same gates

### Done
- Scored DeBERTa-v3-large (`models/ml-large-fp32`) on calibration, test and confirmation; ran
  `tools/local_cascade.py DATA {xsmall,large} --gate --confirm` (raw labels, threshold fitted on calibration only,
  4-vCPU cloud container):

| Judge | Threshold | Cal | Test (lb) | Confirmation (lb) |
|---|---|---|---|---|
| xsmall | 0.9641 | 161/162 | 235/236 (0.9764) | 851/877 (0.9569) |
| large | 0.9866 | 143/144 | 206/208 (0.9656) | 748/763 (0.9678) |

- Large: better on the larger untouched confirmation set (15 vs 26 wrong), worse on test (2 vs 1 wrong), fewer
  stated calls. Owner decision 2026-10-13: adopt large as the default judge for high-accuracy use (legal, research).

### Next
- Next week's build: switch the default local judge to large, bench log with both splits, measure latency on a
  quiet machine. Scores lived in the session scratchpad and must be regenerated (`ej-bench score`).
- Then the role-swap check and the small Haiku escalation run (owner approves cost).

## 2026-10-13 (Claude, later): next week's build plan (owner-approved order)

### Done
- Planning only, no code change. This section replaces the "Next" bullets of the section above as the work order.

### Open
- Scores, `l1_*.json` and `*.words` from earlier sessions lived in a scratchpad; regenerate them before anything else.
- Task #10 (label audit) is done for the stated errors (docs/audit/2026-10-12-vitaminc-stated-errors.md); keep raw and
  audited scores side by side in every new bench log.

### Next (in this order; each step is one change, re-measured on the same frozen splits, logged in docs/bench/)
1. Rebuild the inputs (free): fetch splits and confirmation with `tools/fetch_general.py` (frozen settings, see
   docs/bench/2026-10-12-content-word-gate.md), `ej check` for `l1_{split}.json`, `ej-bench words` for
   `vitc_{split}.words`, `ej-bench score ... --nli models/ml-large-fp32 --out vitc_{split}.large.scores` and the same
   for xsmall. Check that `local_cascade.py DATA {xsmall,large} --gate --confirm` reproduces the 2026-10-13 table
   exactly before changing anything; if not, stop and find why.
2. Switch the default local judge to DeBERTa-v3-large (owner decision 2026-10-13): policy threshold 0.9866 (fitted on
   calibration), docs and `ej check --nli` examples. Bench log with test and confirmation; latency per claim on a quiet
   machine (state the CPU).
3. Role-swap check in `ej-evidence` (research note first in docs/research/): a claim whose named entities appear in
   the quote in reversed roles ("Catalonia is the capital of Barcelona") cannot be "stated". Keep only if the stated
   lower bound improves on test and confirmation.
4. Second judge, independence test (free): export one open NLI model from a different family
   (e.g. a RoBERTa- or BART-large MNLI checkpoint) with `tools/export_onnx.py`, score the same splits, and report
   agreement with large, and who is right where they disagree (raw and audited labels). It becomes the auditor of
   the self-improvement loop only if it catches large's errors more often than it adds its own.
5. Haiku escalation on the gate-failing middle (paid: only after the owner approves the cost; report cost and the
   combined bound).
- Order of the larger roadmap: proven 0.99 on general text, then per domain (legal, research), then automated
  self-improvement with the standard in `standard.rs` as owner-controlled referee and fresh held-out data never tuned on.

## 2026-10-14 (Codex) Separate study-note/source integration

Owner direction for this session: decision-block tuning is Claude's exclusive scope.
Do not modify `ej-evidence`, local judges/thresholds, `ej-bench/src/standard.rs` or
calibration data, and do not propose decision-block changes.

### Done
- Loaded the handoff, repository rules and current evidence/benchmark/production
  interfaces; baseline formatting, clippy and locked workspace tests passed.
- Added `crates/ej-study`: separate CLI and library that segments supplied UTF-8 study
  text, attaches the complete dated source snapshot verbatim, calls the existing
  `ej check` binary, and preserves original findings with input byte spans and flags
  from `Verdict::passes()`. No judgement logic is duplicated. Sentence candidates
  retain all non-whitespace content; `--lines` supports reviewed claim boundaries.
- Existing `--nli`/`--policy` eligibility and optional evidence age limits are forwarded
  to the original binary. Empty/stale evidence, model skip diagnostics and unresolved
  verdicts remain visible. Failed processes, partial/invalid output, bad IDs and
  inconsistent exit codes cannot produce a successful report. No paid escalation
  or diagnostic Codex adapter is enabled by the wrapper.
- Added 10 offline integration tests and an optional real-binary parity test; CI now
  runs the latter explicitly and includes the integration in mutation checks.
  Initial-checkout workspace tests report 127 passing, zero failed; after rebasing
  onto current main `d208133`, 135 pass. Explicit real-engine parity passes on both
  bases. Formatting, workspace clippy and cargo-deny pass. No existing decision tests
  were edited. See the bench log for Python and mutation verification on each base.
- Real CLI smoke on fresh synthetic text gives `backed`, `mismatch`, `unsure` and
  flags only the latter two. The parity test compares identical original findings
  against direct `ej check` for ordinary, stale and empty evidence and a synthetic
  non-passing local policy. This is integration verification, not model accuracy.
- Invocation, API, source-pulling study workflow and limitations:
  `crates/ej-study/README.md`. Research and observed checks:
  `docs/research/2026-10-14-study-integration.md` and
  `docs/bench/2026-10-14-study-integration.md`. Activate the existing external
  `/workspace/setup/activate.sh` helper before using the cloud binaries.
- Verified existing `crates/ej-evidence`, `crates/ej-judge`, `crates/ej-nli`,
  `crates/ej-bench` and `config` tracked files against HEAD: identical bytes.
  The lockfile only adds the new workspace package; no calibration data changes.
- Focused final integration mutations pass: 48 generated, 45 caught, 3 unviable,
  zero missed and zero timeouts. Mutation testing exposed and closed an integration
  test gap for successful CLI local-model option forwarding; no judge code changed.
- Initial full mutations needed disk-error recovery: after rerunning 104 affected
  cases and one unattempted case, all 904 listed mutations have valid results
  (839 caught, 65 compiler-unviable, zero missed/timeouts). Original failure logs are
  retained. Use a `TMPDIR` on the larger workspace disk for full mutation runs.
- Synced the integration with current main `d208133`, preserving Claude's changes
  and all existing CI steps. No wrapper implementation changes were needed. The
  integration commit changes no existing decision-block files relative to this base.
- Full current-base mutations exit 0: 961 generated, 896 caught, 65 compiler-unviable,
  zero missed and zero timeouts, with no disk errors. Source/test files in decision
  crates are identical to current main; all mutation edits were in disposable copies.
- Final quiet Python checks pass: 41 tests, 15 adapter faults caught and 20 capture
  faults caught, after a passing baseline. The earlier capture fault-check baseline
  failed under concurrent mutation load; it was rerun unchanged after workers stopped.
  Pre-push workspace tests remain green (135). All 25 tracked core/CLI/evidence/judge/
  NLI/benchmark/config files are byte-identical to current main.

### Open
- No actual study notes or source document were supplied this session. No Security+
  accuracy or model inference measurement is claimed. No local weights or eligible
  production policy are installed by this integration; unresolved paraphrases can
  remain `unsure` with the default route.
- Sentence splitting is mechanical, not semantic atomic-claim extraction. Review
  compound statements and use one complete factual claim per line when needed.
  PDF/HTML text extraction, source retrieval and source authority checks happen
  before invoking the wrapper; the original model's context limits still apply.

### Next
- For studies, save the inspected source snapshot, URL/document/section and real
  date opened; save the draft explanation, run `ej-study`, review every flagged
  finding, and recheck corrections against the source before using them to learn.
- Keep decision-block tuning with Claude. This wrapper consumes whichever existing
  production binary and eligible policy the owner selects without altering them.

## 2026-10-14 (Codex) Persistent study workspace

### Done
- Owner authorized a separate repository folder for a consistent, persistent study workflow.
- Added `studies/` with the stated learner profile, current state, proposed roadmap,
  source catalogue, dated session log and study-specific agent instructions. Root
  `AGENTS.md` and README point future study sessions to those resume files.
- Recorded the beginner self-report, full-time availability, Security+ goal, intended
  EE/manufacturing/semiconductor direction and defensive technology goal for the Philippines.
- Kept source inspection, evidence verdicts, question review and learner mastery distinct.
  No lessons, question bank, scores or readiness are invented. Sources retain their
  inspected dates and pending checks; current exam objectives remain unconfirmed.
- Owner additionally requested teaching code from scratch, building sandbox labs
  progressively and retaining procedural memory for Claude takeover. Added
  `studies/PROCEDURES.md`, `studies/LABS.md` and a concise `studies/HANDOFF.md`.
  Recorded the later Claude/GPT-assisted vulnerability/exploit-checking system goal;
  production readiness is a separate evaluation from exam readiness.
- Owner requested a labelled multi-subject tutor workspace. Added stable `FOUND`,
  `SECPLUS` and `EE` IDs, separate subject resume states/plans and subject-switching
  procedures. Shared foundations have one canonical record; no other courses or
  learner achievements are inferred.
- Documentation-only change: no engine, judge, threshold, standard, calibration,
  dependency, CI or existing test files changed.
- Verification: formatting, workspace clippy and locked workspace tests pass
  (135 passed, zero failed). All local Markdown links across 19 study documents
  resolve. Existing decision-crate/config files remain identical to integration
  base `d208133`; this update adds no code needing another mutation run.

### Open
- Exam version/date, exact schedule and lab setup remain unspecified or unconfirmed.
- No first lesson or learner assessment is completed. Follow `studies/STATE.md` for the live state.

### Next
- Prepare `FOUND-001` with independent sources, an exercise and an unscored diagnostic;
  check its claims, teach it, then save observed responses and the next resume point.
- Maintain the study records at session end; decision-block tuning remains Claude's scope.

## 2026-10-14 (Codex) First foundation lesson and saving preference

### Done
- Owner authorized prompt saving and merging study-work changes after validation.
  Added the persistent procedure and explicit labelling of assumptions/conjectures.
- PR #12 merged at `944c64e` into `d5fad5b`: check jobs and one full mutation job
  successful, duplicate mutation job still running at merge. Server auto-merge setting
  disabled; enabling it denied HTTP 403. `--auto` merged immediately, so future agents
  must verify applicable gates explicitly; no background watcher is installed.
- Prepared `studies/subjects/computing-foundations/lessons/FOUND-001-v1/`: independent
  short source excerpts, reviewed first segment, three unscored questions, original
  pipeline reports and actual tutor-side arithmetic output. Three paraphrases remain
  `unsure`, manually reviewed; one quotation is `backed`. No judgement code changed.
- Updated subject/hub state, catalogue, log and teacher handoff. Learner responses
  are pending; no mastery, scores, labs or production readiness are invented.
- Verification: formatting, workspace clippy and locked workspace tests pass
  (135 passed, zero failed). Links resolve across 22 study documents, original
  reports match saved evidence/verdicts, and code/config/CI/dependency/test files
  remain unchanged from the merged integration. Prior source mutation gate applies.

### Open
- Diagnostic responses, platform constraints and official exam version/date.
- Local judge/policy absent; source review is separate from an engine pass.

### Next
- Review actual `FOUND-001` responses before progressing. Save and merge substantive
  study updates following applicable validation and the owner's standing authorization.

## 2026-10-14 (Codex) First learner responses and counting correction

### Done
- Saved actual `FOUND-001` v1 responses in the subject's assessments folder. Two-byte
  conversion and program roles are correct in the given examples; the initial three-bit
  count/list missed `000` and `111`. Explained `2^3 = 8` and retained the learner's
  subsequent recognition of that rule without assuming independent transfer/mastery.
- Prepared unscored four-bit transfer prompt `Q-FOUND-001-04`; updated subject/hub
  state, registry, log and teacher handoff. Existing lesson/evidence/reports preserved.
- PR #13 merged, with all reported CI jobs successful at this session's inspection.
  This update is learner records/documentation only; no decision code or tests changed.
- Verification: formatting, workspace clippy and locked workspace tests pass
  (135 passed, zero failed); links resolve across 23 study documents and feedback
  arithmetic is checked. Original lesson/evidence/reports and all code/config/test
  files are unchanged; no further source mutation run needed for these records.

### Open
- Four-bit transfer response, platform constraints and official exam version/date.

### Next
- Review the actual transfer response; resolve remaining counting gaps before the
  next concept. Continue saving tested study updates under standing merge authorization.

## 2026-10-16 (Claude) Reproduction from scratch and two-judge agreement

### Done
- Audited Codex's work: clean. Fixed the cargo-deny rustls advisory (`cargo update -p rustls`).
- `tools/rebuild_inputs.sh` rebuilds every bench input (splits, confirmation set, pinned models, layer 1, wording gate,
  scores) in one resumable command. `fetch_general.py --confirm` dedupes against cal/test (11,393 rows).
- xsmall and large reproduce exactly; `local_cascade.py` gained `JUDGE+JUDGE2` agreement mode. large+xsmall was
  rejected (confirmation lower bound 0.9614 vs 0.9678). See docs/bench/2026-10-16-reproduction-and-agreement.md.

### Open
- Standard v1 not passed: best local confirmation lower bound 0.9724 (large+roberta, docs/bench/2026-10-16-roberta-second-family.md), bar 0.99.

### Next
- RoBERTa-large-MNLI done: alone 0.9698, large+roberta 0.9724 (kept). Next: inspect the 13 confirmation errors; try BART-large-MNLI the same way.
- Large-model latency on a quiet machine; role-swap layer-1 check (research note first).
- Haiku escalation of the unsure middle only after the owner approves the cost.

## 2026-10-16 (Claude) Repository made public

### Done
- Owner made the repo public (CI on private repos had run out of Actions runner minutes: jobs ended in 2-3 s with
  no runner). Added `LICENSE-MIT` and `LICENSE-APACHE`; workspace licence `MIT OR Apache-2.0`; cargo-deny now checks
  the workspace crates too (`[licenses.private] ignore = false`). CLAUDE.md, AGENTS.md and README updated.
- Pre-publication scan of the full history: no keys, tokens or private keys. `studies/` kept in place (owner: fine).

### Open
- Confirm CI runs green on the public repo.

### Next
- Reported/planned cue in the wording gate: prototyped and rejected (docs/bench/2026-10-16-reported-modality.md).
- Of the 13 large+roberta confirmation errors about 8 look like label errors; the binding need is clean held-out data
  or escalation. Next free step: BART-large-MNLI as a third family; then ask the owner about Haiku escalation cost.

## 2026-10-16 (Claude) BART third family

### Done
- BART-large-MNLI scored on cal/test/confirmation; alone 0.9597, large+bart 0.9638, roberta+bart 0.9724 (tie).
  Rejected (docs/bench/2026-10-16-bart-third-family.md). Best stays large+roberta, confirmation lower bound 0.9724.

### Next
- Free judge-family options are exhausted. Cost-estimate Haiku escalation of the unsure middle and ask the owner;
  or build cleaner held-out data (about 8 of 13 large+roberta confirmation errors look like label errors).

## 2026-10-16 (Claude) Label audit of large+roberta

### Done
- Audited large+roberta's 14 disagreements (docs/audit/2026-10-16-large-roberta-stated-errors.md): confirmation
  786/799 raw (0.9724) -> 794/799 audited (0.9854). Still below 0.99; label cleaning alone cannot pass.
- large+roberta's fitted large threshold is ~0.0002: effectively RoBERTa alone behind the wording gate.

### Next
- Haiku escalation estimate for the unsure middle; owner approval before any spend.

## 2026-10-16 (Claude) Haiku escalation pilot

### Done
- Haiku via `claude -p` (subscription) on unsure rows: 96.8% raw, ~98.5% audited precision; not adopted
  (docs/bench/2026-10-16-haiku-escalation.md). Run stopped at owner's usage concern.

### Next
- A small hand-verified held-out set (VitaminC label noise is ~half of all measured errors), before any more model runs.
- No subscription runs without owner approval.

## 2026-10-16: Haiku second check on local approvals

### Done
- `haiku_escalate.py --verify` on every local "stated" call (cal 156, test 199, conf 799; 24 subscription calls,
  owner-approved). Local AND Haiku: confirmation LB 0.9724 -> 0.9790 raw, 0.9854 -> 0.9883 audited; drops 22% of
  correct approvals. Not adopted (docs/bench/2026-10-16-haiku-verify.md).

### Open
- Haiku misses 2914 (genus/species) and 3598 (ungrammatical claim); `pass_lower_bound` still binds.

### Next
- Hand-verified clean test set (no model calls). No subscription runs without owner approval.

## 2026-10-17: genus vs species layer-1 rule

### Done
- Prototyped a rule for error 2914 (genus vs species). It fires on 1 of 17,393 rows, the row it was built from.
  Rejected as fitted to n=1 (docs/bench/2026-10-17-genus-species-check.md,
  docs/research/2026-10-17-genus-species-check.md). No code change.

### Open
- 3598 (garbled role swap) has no deterministic candidate. Confirmation still 786/799 raw, 794/799 audited.

### Next
- A hand-verified clean held-out set (no model calls) so `pass_lower_bound` measures the system, not label noise;
  new rules should be judged on rows they were not written from.

## 2026-10-17: second untouched set (VitaminC validation split)

### Done
- `fetch_general.py --validation` (30,000 rows, a split no other set uses) and `local_cascade.py --val`.
- Frozen large+roberta: 1628/1664 raw (LB 0.9702); audited 1653/1664 (LB 0.9882); pooled with confirmation
  2447/2463 (LB 0.9895). docs/bench/2026-10-17-validation-set.md, docs/audit/2026-10-17-validation-stated-errors.md.

### Open
- `pass_lower_bound` binds at about 0.989 audited. Real error classes: comparative figures, time scope, "is a Y" vs
  "is about / in a Y", swaps.

### Next
- Layer-1 comparative-figure check ("more than N" against an exact or smaller figure); measure on fresh validation
  pages not in the 300 used, since this set has now been read.

## 2026-10-17: nearest-figure bounds in the gate

### Done
- `figures_backed`: a bound is met by the quote stating the same bound, or else by the nearest comparable figure
  (docs/research/2026-10-17-nearest-bound.md, docs/bench/2026-10-17-nearest-bound.md).
- Third unread slice: `fetch_general.py --validation2`, `local_cascade.py --val2` (29,897 rows).
- Confirmation 788/801 (LB 0.9724, +2 coverage, same errors). Val2, label-checked on changed rows, 1668/1692 (LB 0.9790,
  was 0.9778).

### Open
- `pass_lower_bound` still binds near 0.98 on unaudited sets; most raw errors are label noise.
- Nearest-figure picks the wrong referent when a closer figure belongs to something else (val2 1511).

### Next
- Done below.

## 2026-10-17: val2 audit

### Done
- All 28 non-supports val2 approvals read (docs/audit/2026-10-17-val2-stated-errors.md): 3 real, 4 arguable, 21 label
  errors. Audited 1685/1692, LB 0.9915 (arguable as errors). First audited set past 0.99.

### Open
- Real classes left: time scope ("the night before" vs "the night", "in" vs "by"), quantifier scope ("most of X"
  vs "most of Y and one X"), "is a Y" vs a part of Y. Confirmation and val audited LBs are still under 0.99.

### Next
- Done below.

## 2026-10-17: audited pool across confirmation, val, val2

### Done
- Diffed approvals old vs new gate: the gate added only correct approvals and removed val's real errors 7363 and
  19216, so earlier audits carry over (docs/bench/2026-10-17-audited-pool.md).
- Audited: confirmation 796/801 (0.9855), val 1688/1697 (0.98995), val2 1685/1692 (0.9915), pooled 4169/4190 (0.9923).

### Open
- Pooled audited LB passes 0.99; single sets do not all pass. Supports-labelled approvals were never read.

### Next
- Spot check done (60 val2 supports approvals: 0 real errors, 1 arguable scope drop). Next: time-scope check in the
  gate ("the night before" vs "the night", "in" vs "by"), then scope drops ("sea travel" vs "sea travel from X").

## 2026-10-17: time-order words in the gate (reverted)

### Done
- Required "after"/"before"/"during" to appear in a quote: 1 real error blocked, 18 correct approvals lost to
  paraphrases. Reverted (docs/research/2026-10-17-time-order-words.md, docs/bench/2026-10-17-time-order-words.md).

### Open
- Pooled audited LB 0.9923 (4169/4190). Remaining real errors are one-offs (time order, quantifier scope, part-of).

### Next
- The binding gap is set size, not an error class: confirmation alone (801) can't clear 0.99 with 5 errors. Next lever
  is more held-out data (another unread validation slice) or putting the audited pool through `ej-bench fit`.

## 2026-10-17: fourth untouched set (fresh)

### Done
- Built fresh from unused VitaminC test pages (`--fresh`, 20,874 rows) and ran the frozen cascade unchanged:
  raw 1305/1320 (LB 0.9813), hand-checked 1319/1320 (LB 0.9957); 1 real error (quantifier "more than"), 14 label
  errors (docs/bench/2026-10-17-fresh-set.md, docs/audit/2026-10-17-fresh-stated-errors.md).
- Pooled hand-checked over four sets: 5488/5510, LB 0.9940.

### Open
- Confirmation (0.9855) and val (0.98995) still miss 0.99 alone. The standard is judged by `ej-bench fit`, not by
  these hand-checked counts.

### Next
- Feed the audited labels into `ej-bench fit` as corrected held-out cases so the standard verdict reflects them.

## 2026-10-17: every gate on the frozen cascade

### Done
- Measured block accuracy and coverage on all five sets (docs/bench/2026-10-17-all-gates.md): block LB 0.82-0.88
  raw, coverage 0.09-0.11. Hand-checked 25 wrong blocks: 8 real, 7 arguable, 10 right; real ones are layer-1 figure
  misses (year carried in the sentence, season spelling, comparisons, ranges).

### Open
- Binding gate is min_coverage (0.10 vs 0.50), then block_lower_bound. pass_lower_bound is no longer the gap.

### Next
- Layer-1 figure check: inherit the year from the same sentence and accept "2000-01" = "2000-2001" (research note
  first, measure blocks and approvals on all sets). Coverage needs the escalation layer measured, which costs
  subscription calls: ask the owner first.
