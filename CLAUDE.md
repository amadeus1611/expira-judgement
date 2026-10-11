# EXPIRA Judgement

Private Rust anti-hallucination engine. Read `HANDOFF.md` first; add your own Done / Open / Next section at the end before you stop.

- Rules: repo stays private; fresh code only (nothing copied from gssc-system/, client data, memory or pricing); no invented figures; one small tested change per commit; `cargo test` green before every push.
- The decision block (layer 1 `ej-evidence`, local judges, Claude escalation) is adopted only when it passes the standard in `crates/ej-bench/src/standard.rs`, measured on held-out data with `ej-bench fit`. The standard is code: change a bar only in a deliberate commit that says why.
- Improving the decision block (new model, tuning, new check, new data) follows the `iterate` skill: measure, change one thing, re-measure on the same frozen splits, keep it only if the standard verdict or coverage-at-risk improves, log it in `docs/bench/`.
- Claims about which model or setting is better need measured numbers from `ej-bench`, with the data and machine stated.
