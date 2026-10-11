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
