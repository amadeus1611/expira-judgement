# EXPIRA Judgement architecture

EXPIRA Judgement is a **bring-your-own-model** verification architecture. It decides whether a claim is backed by its
evidence, and only acts where it can prove its error rate. It ships the decision logic, the standard, the measurement
harness and the improvement loop. Models, subscriptions and keys are supplied by whoever runs it: their own API keys,
their own agent subscriptions (Claude Code, Codex or similar run as a command), or local open models.

Owner decision 2026-10-11: the repo is an architecture people plug their own subscriptions and models into. The earlier
"Claude only" rule is the owner's **default configuration** (`config/roster.json`), not a limit of the architecture.

## The cascade

```
claim + evidence
   │
   ▼
Layer 1  ej-evidence     deterministic: source, date, staleness, every figure/unit/date in a quote, verbatim match,
   │                     negation. Free, certain. Settles "backed" / "mismatch" / "stale" / "unbacked" or passes on.
   ▼
Layer 2  local judges    open models on the user's CPU (ej-nli ONNX, or any process via the line protocol), wrapped in
   │                     a fitted calibration; they act only past thresholds proven on held-out data (policy file).
   ▼
Layer 3  escalation      a strong model the user supplies (API or subscription CLI). Must cite a span found in the
   │                     quotes; every ruling becomes a labelled teacher pair that recalibrates layer 2.
   ▼
verdict + reason + layer + confidence
```

Delegated work (not claims) uses the same idea: `ej-delegate` routes each task kind to the cheapest tier expected to pass
its gate, from measured pass rates and costs.

## Plug-in points (ports)

| Port | Where | What a user plugs in |
|---|---|---|
| `Judge` trait | `crates/ej-judge/src/lib.rs` | Any judge in Rust: `rule()`, optional `p_stated()` for calibration, `self_gated()` |
| Line protocol | `crates/ej-bench/src/command.rs` | Any judge as a long-running process: JSON request per line in, `{ruling, confidence, p, reason}` out (e.g. `tools/laya_shim.py`) |
| `Transport` trait | `crates/ej-judge/src/claude.rs` | How the escalation request is sent: HTTP to an API today (`crates/ej-cli/src/http.rs`); a subscription CLI is the next adapter (see below) |
| ONNX model directory | `ej-nli` | Any NLI cross-encoder exported to ONNX (`model.onnx`, `tokenizer.json`, `config.json` with `id2label`) |
| Policy file | `ej-bench fit --write-policy` | Calibration and thresholds fitted on the user's own labelled data; a judge is used only if it passed the standard |
| Roster | `config/roster.json` | Model per delegation tier and the allowed list (owner default: Claude Haiku 5.5, Sonnet 5.5, Opus 5.5) |
| Prices | `config/prices.json` | Per-model rates with source and date; unpriced models report unknown cost, never zero |
| Logs | `teacher_pairs.jsonl`, `usage.jsonl` | Written by runs; read back as calibration data and cost evidence |

### Subscription adapter (next step, not built yet)

Users without an API key run escalation through the agent they already pay for. Design: a `CommandTransport` that sends
the same request (system prompt, claim, quotes, JSON schema) as text on stdin to a configured command, for example
`claude -p --output-format json` or `codex exec`, and reads the reply JSON from stdout. Everything after the transport is
unchanged: schema check, cited-span check, teacher pairs, usage (token counts where the CLI reports them, otherwise
calls and latency only, cost "unknown").

### What changes when the roster is opened to other vendors

`ej_delegate::ALLOWED_MODELS` is the owner's Claude-only list compiled into the binary today. To make the roster fully
user-configurable: move the allowed list into `config/roster.json` (keeping the owner's three as the shipped default),
validate tiers and the escalation model against it, and keep a test that the shipped config is Claude-only.

## What is fixed, whatever is plugged in

- The standard (`crates/ej-bench/src/standard.rs`): bounded accuracy on held-out data, stress, calibration, coverage,
  latency. A plugged-in judge earns the right to act by passing it on the user's data; nothing is trusted by default.
- Layer 1 runs first and cannot be bypassed by a model.
- Escalation replies must cite text found in the quotes.
- Engineering standard in CI: fmt, clippy (all + pedantic as errors), tests, property tests, zero missed mutants.

## Crates

| Crate | Role |
|---|---|
| `ej-core` | Shared types: claim, evidence, verdict, layer, finding, negators |
| `ej-evidence` | Layer 1 |
| `ej-judge` | `Judge` trait, cascade (`adjudicate`), calibration wrapper, overlap judge, escalation judge + `Transport` |
| `ej-nli` | ONNX cross-encoder judge |
| `ej-bench` | Bench harness, calibration maths, standard, line-protocol adapter, `ej-bench` CLI |
| `ej-delegate` | Tier routing, roster, cost-aware start tier |
| `ej-habits` | Teacher pairs, usage records, prices, habit adoption |
| `ej-cli` | `ej check` CLI |
