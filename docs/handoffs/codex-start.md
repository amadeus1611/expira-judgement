# Start prompt for Codex (or any coding agent)

Paste everything below the line into a new Codex session opened on `amadeus1611/expira-judgement`.

---

You are continuing EXPIRA Judgement, a private Rust repository I own. It was built with Claude Code; from now on you
are its agent. Treat every file written "for Claude" as written for you: `CLAUDE.md`, `AGENTS.md`, `HANDOFF.md`,
`.claude/skills/iterate/SKILL.md`, the research notes and bench logs are your own memory and your own operating rules.
Use them the way Claude did. Nothing in them is about which assistant you are; they describe how this repo is run.

**Read first, in this order:** `AGENTS.md`, `CLAUDE.md`, `HANDOFF.md` (every section; the newest is last),
`docs/ARCHITECTURE.md`, `.claude/skills/iterate/SKILL.md`, `crates/ej-bench/src/standard.rs`, then the newest files in
`docs/bench/` and `docs/research/`.

**What this is.** A bring-your-own-model verification architecture: a claim is checked against its evidence by a
cascade. Layer 1 is deterministic code; layer 2 is local open models that act only past thresholds proven on held-out
data; layer 3 is a strong model the user supplies (an API, or an agent subscription run as a command), which must cite
text from the evidence, and whose rulings become labelled data that recalibrates layer 2. Users plug in their own
models, subscriptions and keys; the repo ships the logic, the standard, the harness and the improvement loop. No API
key is needed to develop or test it.

**My intent, which you continue:**
- Nothing passes as fact unless proven or backed by a judge whose error rate is bounded on held-out data.
- Standards only go up. Every standard that can be checked is checked by code or CI, never by prose alone.
- Improve continuously through the iterate loop: when anything fails or could be better, research open academic and
  open-source work first (note in `docs/research/`, `cargo test` checks its sections), change one thing, re-measure on
  the same frozen data, keep it only if measured better, log it in `docs/bench/` and `HANDOFF.md`.
- Optimise cost and efficiency as hard as accuracy: cost per settled claim and per completed task, never per request.
- The loop runs inside my sessions, around my requests, filling gaps; there is no schedule. I direct; you choose the next
  gap yourself when I have not.

**Hard rules (also in AGENTS.md):** never lower a bar in `standard.rs`, delete or weaken a test, edit frozen data, or fit
on test data; work on a branch and open a PR with measured before/after; before every push run `cargo fmt --all --
--check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and for the decision crates
`cargo mutants -p ej-core -p ej-evidence -p ej-judge -p ej-delegate -p ej-habits -p ej-bench --exclude crates/ej-bench/src/main.rs` (zero missed); keep the toolchain pin
unless you are deliberately upgrading it and fixing every new lint in the same change; fresh code only (nothing copied
from gssc-system/, client data, memory or pricing); no invented figures; claims about models need measured numbers.

**Where things stand:** standard v1 is not passed by any configuration. The binding gate is the 0.99 lower bound on
"stated" calls. Best measured so far (public FEVER + MNLI, local only): DeBERTa-v3-large fp32 AUROC 0.973, mean of large
+ xsmall 0.975; the large model is likely too slow for the 250 ms gate on a 4-vCPU CPU. LAYA (from my other engine)
ranked worst and passes many negated claims; that finding is reported, not acted on, in the other repo.

**Next, in this order, unless I say otherwise:**
1. Make the architecture fully pluggable (`docs/ARCHITECTURE.md`): add a `CommandTransport` so escalation can run through
   a user's subscription CLI (e.g. `claude -p --output-format json`, `codex exec`), and move the allowed-model list from
   `ej_delegate::ALLOWED_MODELS` into `config/roster.json`, keeping my Claude-only list (Haiku 5.5, Sonnet 5.5,
   Opus 5.5) as the shipped default with a test that pins it. Tests that kill every mutant.
2. Done 2026-10-11: `ej-bench` (lib) is at zero missed mutants and in the CI mutants job; keep it there.
3. Bench: re-measure large fp32 latency on a quiet machine; try MiniCheck (export to ONNX), a calibrated int8 large
   model, and layer-1 negation handling in front of the judges (all stress wrong passes so far are negations).
4. Collect real labelled ledger cases (teacher pairs from escalation) to make the 0.99 bound reachable.

Ask me before anything that is hard to reverse or leaves the repo. At the end of every session, add your section to
`HANDOFF.md`.
