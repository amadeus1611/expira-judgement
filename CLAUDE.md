# EXPIRA Judgement

Private Rust anti-hallucination engine. Read `HANDOFF.md` first; add your own Done / Open / Next section at the end before you stop.

- Engineering standard (enforced by CI): `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` under the workspace lints in `Cargo.toml` (no `unsafe`, clippy all + pedantic as errors, each allow with its reason), `cargo test`. It only ratchets up: add or tighten lints and checks, never relax one to get green.
- Posture (owner, 2026-10-11): AI-driven self-improvement. The owner gives help and direction; the agent does the analysis, decides within the guardrails, and executes end to end, choosing the next gap itself when not directed.
- Rules: repo stays private; fresh code only (nothing copied from gssc-system/, client data, memory or pricing); no invented figures; one small tested change per commit; `cargo test` green before every push.
- The decision block (layer 1 `ej-evidence`, local judges, Claude escalation) is adopted only when it passes the standard in `crates/ej-bench/src/standard.rs`, measured on held-out data with `ej-bench fit`. The standard is code: change a bar only in a deliberate commit that says why.
- Improving the decision block (new model, tuning, new check, new data) follows the `iterate` skill: measure, research open sources whenever a problem appears (note in `docs/research/`), change one thing, re-measure on the same frozen splits, keep it only if the standard verdict or coverage-at-risk improves, log it in `docs/bench/`.
- Claims about which model or setting is better need measured numbers from `ej-bench`, with the data and machine stated.
