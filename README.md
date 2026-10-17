# EXPIRA Judgement

Layered anti-hallucination engine. Nothing passes as fact unless it is proven or a judge with a recorded reason backs it.

| Layer | Crate | What decides | Cost |
|---|---|---|---|
| 1 | `ej-evidence` | Deterministic: source + valid date + quote present, every figure/unit in the claim appears in a quote, staleness, sums | free, certain |
| 2 | `ej-judge`, `ej-nli` | Local judge for paraphrases (pluggable `Judge` trait; `OverlapJudge` placeholder, `NliJudge` ONNX candidate; pick after the bake-off) | cheap |
| 3 | `ej-judge::claude` | Claude (Messages API), only for what layer 2 leaves `unsure` or high stakes; must give a reason; each ruling kept as a teacher pair | paid |

Supporting systems:
- `ej-delegate`: agent roster. Cheapest tier first (scout -> worker -> builder -> reviewer), escalate one tier only when the gate fails, high stakes start at reviewer, every attempt logged.
- `ej-habits`: in-repo self-improvement, no model training. Habits are candidates until enough observations agree, adopted, and retired on the first contradiction. Each Claude ruling is stored as a teacher pair for layer 2's eval set.
- `ej-bench`: bake-off harness; `ej-bench cases.jsonl --overlap --nli DIR --cmd ...` reports accuracy when acting, unsure rate, CPU latency. Runs in `docs/bench/`.
- `ej-cli`: `ej check claims.json --today 2026-10-11 --max-age 365 [--claude --pairs teacher_pairs.jsonl]` (exit 1 if any claim fails; `--claude` needs `ANTHROPIC_API_KEY`).

Verdicts: `backed`, `supported` (pass); `unbacked`, `mismatch`, `stale`, `unsure` (fail; escalate or leave out).

For study notes or generated explanations against a supplied source document, use
the separate [`ej-study` integration](crates/ej-study/README.md). It calls the existing
`ej check` binary and returns original findings with note locations and review flags.

The persistent [study workspace](studies/README.md) keeps labelled subject records,
learning progress, sources and teaching/lab procedures for Codex or Claude to resume.

## Roadmap
1. ~~Claude escalation client, rulings written as teacher pairs.~~ Done; not yet run against the live API.
2. Bake-off: harness and DeBERTa NLI done (first numbers on public MNLI); still LAYA via `--cmd`, a small constrained model, and ledger cases.
3. Layer 1 hardening: dates, ranges, currencies, negation done; keep adding real ledger cases as they come.
4. Habit store on disk + reporting; tier log.
5. Hosted API.

Later, separate: EXPIRA Design (palette, type and derivative solver under rules).

Dual-licensed under MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`), at your option.
