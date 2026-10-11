# Codex subscription escalation prerequisite (2026-10-11)

## Problem
The owner directed this session to use the in-house Codex CLI and subscription instead of Claude for escalation.
The built-in Codex CLI is authenticated, but a bounded synthetic smoke request could not complete in this cloud
machine. Identify the actual runtime and network prerequisites before changing the benchmark or spending on held-out rows.

## Sources
- Installed `codex-cli 0.159.2`: `codex exec --help` documents stdin prompts, JSONL events, output-schema,
  output-last-message, ephemeral execution and read-only sandboxing.
- Noninteractive documentation pointer, read from the official repository:
  https://github.com/openai/codex/blob/main/docs/exec.md
  https://developers.openai.com/codex/noninteractive
- Official configuration schema, read for supported sqlite_home and log_dir overrides:
  https://github.com/openai/codex/blob/main/codex-rs/core/config.schema.json
- App-server startup and installation identifier implementation, read to diagnose the failed write:
  https://github.com/openai/codex/blob/main/codex-rs/app-server/src/lib.rs
  https://github.com/openai/codex/blob/main/codex-rs/core/src/installation_id.rs

## Licence
The official Codex repository LICENSE was read: Apache-2.0. No implementation code was copied into EXPIRA.
The preinstalled CLI is used as a tool; authentication remains with the platform's existing binding.

## Candidates
1. Request a new API key. Rejected: `codex login status` already succeeds and reports ChatGPT authentication.
2. Copy credentials or relocate CODEX_HOME. Rejected: preserve the existing authentication and managed runtime.
3. Use supported sqlite_home/log_dir overrides under /workspace, ephemeral execution and the existing CLI.
4. Use automatic approval for the CLI's required startup write, then verify the actual subscription endpoint.

## Decision
Use Codex as requested by the owner; the historical Claude-only default does not override that direction.
Keep the existing schema and cited-span requirement when adding the benchmark adapter, and use only calibration
rows to fit any policy. No benchmark adapter or model ranking is adopted on this unsuccessful smoke request.

The installed CLI reports ChatGPT authentication without printing credential values. Default exec fails during
app-server initialization because the managed Codex home is read-only. Supported sqlite_home and log_dir overrides
move databases/logs to /workspace, but a file-only syscall trace shows the remaining failed O_RDWR|O_CREAT open is
`/run/codex-environment/codex-home/installation_id`. The official implementation requires write access even to reuse
an existing identifier. A single automatically approved exec invocation got past initialization without moving or
copying credentials. Use that supported approval route when needed; do not bypass sandbox policy.

The one synthetic smoke invocation then encountered HTTP CONNECT 403 for
`wss://chatgpt.com/backend-api/codex/responses` and timed out at 120 seconds. No structured reply, completed turn,
token-usage event or cost report was produced. Usage and dollar cost are unreported, not invented as zero.
No held-out row was sent, and no model accuracy is claimed. Full smoke artifacts are under /workspace/setup/codex-smoke.

Saved chatgpt.com as an additional custom network destination, preserving the existing saved domains and presets.
Retry only after a meaningful runtime policy change. Once the endpoint works, test a Codex benchmark adapter with
schema/exit-status/usage/citation failures, then measure the ordered whole-cascade experiment in small batches.
