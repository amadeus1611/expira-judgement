# Cost and usage optimisation with delegation (2026-10-11)

## Problem
Paid model calls (Claude escalation, delegated agent work) need to cost as little as possible without lowering the bar any
gate enforces. The roster is Claude only (Haiku 5.5, Sonnet 5.5, Opus 5.5, owner decision 2026-10-11). Before this change
nothing recorded what a call cost, and each task kind started at a fixed tier whatever its history.

## Sources
- FrugalGPT: How to Use Large Language Models While Reducing Cost and Improving Performance (LLM cascades: try cheaper models first, stop when a scorer accepts): https://arxiv.org/abs/2305.05176
- RouteLLM: Learning to Route LLMs with Preference Data (route each query to a strong or weak model from learned win rates): https://arxiv.org/abs/2406.18665
- RouteLLM code: https://github.com/lm-sys/RouteLLM
- Anthropic pricing (per-model rates, cache and batch multipliers): https://platform.claude.com/docs/en/about-claude/pricing
- Anthropic prompt caching (prefix caching; minimum cacheable prefix 512 tokens on the 5.5 models): https://platform.claude.com/docs/en/build-with-claude/prompt-caching
- Anthropic Message Batches (asynchronous, 50% of standard price): https://platform.claude.com/docs/en/build-with-claude/batch-processing

## Licence
- Papers: read for method only; nothing copied.
- RouteLLM code: Apache-2.0 (LICENSE file in the repo). Not vendored; the cascade maths here is our own.

## Candidates
1. Usage log per call (tokens in/out, cache read/write, latency, route) and cost from a sourced, dated price file; unknown when unpriced. Done.
2. Cascade start tier per task kind chosen by lowest expected cost to a passed gate, from measured pass rates (Beta(1,1) prior) and measured mean attempt cost. Done (`ej-delegate/src/cost.rs`).
3. Escalation cascade inside the judge: Haiku 5.5 first, Sonnet 5.5 / Opus 5.5 only when the cheaper ruling is unsure or fails the span check. To measure: accuracy-when-acting per model on teacher pairs, cost per settled claim.
4. Batch API for non-urgent ledger checks (stop-gate runs, nightly re-checks): 50% price, latency not needed.
5. Prompt caching: today's escalation prompt is well under the 512-token minimum, so caching would do nothing per claim; it pays only if several claims share one long cached prefix (e.g. a whole ledger page as context). To measure before adopting.
6. Learned router (RouteLLM style) once enough logged attempts exist to train one; until then the expected-cost rule above.

## Decision
1 and 2 adopted (tested). 3 to 6 queued in the iterate loop; each is adopted only if cost per settled claim falls with every
standard gate unchanged or better, measured from the usage log.
