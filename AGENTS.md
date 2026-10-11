# AGENTS.md

Instructions for any coding agent working in this repository (Codex, Claude Code, or another). They are the same rules
as `CLAUDE.md`; where a file below is written "for Claude", read it as written for you.

1. Read `HANDOFF.md` first. Before you stop, add your own dated section at the end: Done / Open / Next.
2. Follow `CLAUDE.md` (project rules, engineering standard) and `.claude/skills/iterate/SKILL.md` (the improvement loop:
   research open sources on any problem with a note in `docs/research/`, change one thing, re-measure, keep only if
   measured better, log in `docs/bench/`). The skill's guardrails apply to you: never lower a bar in
   `crates/ej-bench/src/standard.rs`, delete or weaken a test, edit frozen data, or fit on test data.
3. Engineering standard, enforced by CI (`.github/workflows/ci.yml`): `cargo fmt --all -- --check`,
   `cargo clippy --workspace --all-targets -- -D warnings` (workspace lints in `Cargo.toml`), `cargo test --workspace`,
   and zero missed mutants (`cargo mutants -p ej-core -p ej-evidence -p ej-judge -p ej-delegate -p ej-habits`).
   New logic ships with tests that kill its mutants. Toolchain is pinned in `rust-toolchain.toml`.
4. Architecture: bring-your-own-model (`docs/ARCHITECTURE.md`). Users plug in their own models, subscriptions and keys.
   The owner's default roster is Claude only (Haiku 5.5, Sonnet 5.5, Opus 5.5), shipped in `config/roster.json` and today
   compiled into `ej_delegate::ALLOWED_MODELS`; making that list configurable is the next planned change. The start
   prompt for a new agent is `docs/handoffs/codex-start.md`.
5. Rules: private repo; fresh code only (nothing copied from gssc-system/, client data, memory or pricing); no invented
   figures; one small tested change per commit; claims about models need measured numbers from `ej-bench`.
