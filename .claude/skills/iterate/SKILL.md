---
name: iterate
description: Continuously improve EXPIRA Judgement - its decision block (layer-1 checks, local judges, calibration, Claude escalation), its cost and usage, its delegation, and its own structure - against the standard in crates/ej-bench/src/standard.rs, researching open academic and open-source work whenever a problem or an opportunity appears. Use whenever a judge, model, threshold, check or escalation setting is added or changed, when new labelled data or teacher pairs arrive, or when the standard is not yet passed.
---

# Iterate: always optimise

The loop improves the repo's harness (code, checks, judges, calibration, data, routing, docs, structure), never a model's
weights in place. It runs on every change to these systems and whenever a session has spare capacity: there is always a
next gate to tighten, a cost to cut, or a slower path to remove. Out-of-the-box changes (a different architecture, a new
layer, removing a layer) are in scope when measurement supports them.

What is measured, all from logs in this repo:
- quality: the standard's gates (`ej-bench fit`);
- cost and usage: `usage.jsonl` (tokens, cache hits, latency, route per Claude call) priced with `config/prices.json`;
  the number to drive down is cost per settled claim and per completed delegated task, never cost per request;
- delegation: the tier log (attempts and gate passes per task kind and tier) feeding `ej-delegate/src/cost.rs`.
Models: Claude only, Haiku 5.5 / Sonnet 5.5 / Opus 5.5 (`config/roster.json`, enforced in code). No other vendor's models.

1. **Baseline.** Score every candidate judge on the frozen splits and fit:
   `ej-bench score <split>.jsonl --out <split>.<judge>.scores ...` then
   `ej-bench fit --cal <cal scores> --test <test scores> --stress <stress scores> --write-policy policy.json`.
   Exit 0 means at least one configuration passes the standard. Record the verdicts.
2. **Find the binding gate or the biggest cost.** Read which gate fails (`pass_lower_bound`, `block_lower_bound`, `stress_wrong_passes`, `max_ece`, `min_coverage`, `max_p95_ms`) and why: too few held-out cases, a failure class (figures, negation, dates), a slow model, poor ranking (AUROC).
3. **Research, every time a problem shows up.** A failing gate, a failing test, a judge error class, a slow path or a surprise in the numbers triggers external research before any fix. Search as widely as the problem needs, open sources first:
   - papers and preprints (arXiv, ACL Anthology, OpenReview), with their code;
   - open model hubs (Hugging Face model and dataset cards, eval results), open-source repos (GitHub: issues, benchmarks, changelogs);
   - official docs for anything already in use (Anthropic API docs, ONNX Runtime, `ort`, `tokenizers`, LAYA).
   Prefer recent, reproducible, permissively licensed work (Apache-2.0, MIT, BSD; check model and dataset licences too, they often differ from the code's). Treat every claim found as a hypothesis to measure here, never as a result.
   Write a note in `docs/research/<date>-<topic>.md` with the sections `## Problem`, `## Sources` (links), `## Licence`, `## Candidates`, `## Decision`. `cargo test` fails on a note missing any of them. Each candidate then goes through step 4 on its own.
4. **Change one thing** aimed at that gate, in its own commit with a test:
   - data: more labelled cases (Claude teacher pairs load directly as cases), a new stress class;
   - layer 1: a deterministic check that removes a failure class before any model sees it;
   - a judge: a new model (ONNX in Rust where possible), a different question wording, an ensemble;
   - fitting: target, minimum sample, calibration method;
   - escalation: Claude model (Haiku before Sonnet before Opus where measured accuracy allows), effort, prompt, schema;
   - cost: prompt caching, the Batch API for work nobody waits on, fewer or shorter calls, tier start points;
   - structure: merge, split or remove crates, layers or steps when the numbers say so.
5. **Re-measure on the same splits and the same usage replay.** Never fit on the test split. Never tune against the stress set and then report it as unseen; add a fresh stress class instead.
6. **Keep or revert.** Keep only if the standard verdict improves, or with the same verdict the coverage at the same risk rises, or cost per settled claim or latency falls with nothing else worse. Otherwise revert and log why.
7. **Log.** Append a dated run to `docs/bench/` (data, machine, commands, every gate value) and a section to `HANDOFF.md`.

If the standard cannot be met with what is available, say exactly which gate binds and what would move it (usually more held-out labelled cases); do not lower a bar to pass.

## How it runs: alongside the owner, filling the gaps

There is no schedule (owner decision 2026-10-11). The loop runs inside the owner's sessions as a collaborator: the
owner's request comes first; around it, the loop fills gaps without being asked - it picks the next problem itself (the
binding gate, the biggest cost line, the slowest path, an open item in `HANDOFF.md`, the newest relevant open research),
researches, changes, measures and logs - and it carries what each session learned into the next through the logs and
notes below. It needs no human researcher to choose, run or judge a round; the owner directs where effort goes. It learns
from any model's or system's published efficiency work (distillation, quantisation, caching, cascades, routing, early
exit, retrieval) and from its own logs (teacher pairs, usage, tier log, past research notes), and can change the loop
itself (this skill, the bench, the standard's measurement code) when that makes it better at improving.

Guardrails that hold in every round, whether or not the owner is watching it, because an optimiser that can edit its own scorer will otherwise find it:
- Never lower a bar in `standard.rs`, delete or weaken a test, edit a frozen split, or fit on test data to pass.
  Raising a bar, adding a gate or a stress class, or adding held-out data is always allowed.
- Work on a branch and open a PR with the measured before/after; never push to `main` and never merge.
- Every kept change carries its research note, its bench log and a passing `cargo test`.
- Stay inside the session's budget: stop a round that is not converging, log what was learned, and leave the next step in `HANDOFF.md`.
- Claude models only (`config/roster.json`); local judges must be open-licence and run here.
- What cannot be changed from the repo (a model's weights, account settings, other repositories) is reported, not attempted.
