# Research: a subscription transport that can be measured

## Problem
The owner selected the existing Codex subscription in place of Claude. The current Python escalation benchmark
only speaks Claude: it accepts stdout without checking process success, omits the production confidence gate,
and sums missing prices as zero. Its output cannot establish whole-cascade accuracy. Before adding any paid
measurements, implement one bounded call with strict completion, schema, citation and accounting checks.

## Sources
- [Codex exec event definitions, release 0.159.2](https://github.com/openai/codex/blob/rust-v0.159.2/codex-rs/exec/src/exec_events.rs):
  thread/turn events, agent-message items and completed-turn usage. Read the release source; cached copies are
  outside this checkout. The annotated release resolves to commit `ff6aec96948b70d94983af2641a6b67c94faeff5`.
- [JSONL event processor at that release](https://github.com/openai/codex/blob/rust-v0.159.2/codex-rs/exec/src/event_processor_with_jsonl_output.rs):
  a completion includes usage; some utility and collaboration events are omitted. Absence of tool events therefore
  cannot prove absence of all tool activity. No price is present in the usage type.
- [Release configuration schema](https://github.com/openai/codex/blob/rust-v0.159.2/codex-rs/core/config.schema.json) and
  [tool construction](https://github.com/openai/codex/blob/rust-v0.159.2/codex-rs/core/src/tools/spec_plan.rs): supported
  shell, app, plugin, web, collaboration and code-mode switches. A read-only sandbox restricts writes; it is not a
  no-tools guarantee. Patch tools can be model-dependent; system-managed MCP configuration can still be inherited.
- Installed `codex exec --help`, version `codex-cli 0.159.2`: `--ignore-user-config` preserves authentication,
  `--ignore-rules`, `--ephemeral`, schema output, JSONL events and final-message output are available.
- [Release model catalogue](https://github.com/openai/codex/blob/rust-v0.159.2/codex-rs/models-manager/models.json):
  lists `gpt-6-luna` and its supported low effort, used only for synthetic endpoint diagnostics.
- CPython's [subprocess source](https://github.com/python/cpython/blob/v3.12.12/Lib/subprocess.py) and
  [JSON decoder source](https://github.com/python/cpython/blob/v3.12.12/Lib/json/decoder.py), read from GitHub
  because docs.python.org is proxy-blocked: `run` kills only the immediate child on timeout; `communicate` leaves
  termination to its caller. POSIX process groups are needed to bound descendants. Default JSON accepts non-finite numbers and
  duplicate object keys, so validation must reject both explicitly.
- Repository production checks: `crates/ej-judge/src/claude.rs` (reason and cited span) and
  `crates/ej-judge/src/lib.rs` / `crates/ej-cli/src/main.rs` (escalation confidence bar 0.8).

## Licence
Read [Codex's release LICENSE](https://github.com/openai/codex/blob/rust-v0.159.2/LICENSE): Apache-2.0.
This adapter is fresh code using its command protocol, not copied implementation. No model weights or dataset
are introduced; the user's managed authentication stays in its original home. Python libraries are built in.

## Candidates
1. Copy the old Claude stdout parser and rename the command. Rejected: wrong event framing, unchecked process
   failure and misleading cost would carry over.
2. Use the Codex app-server protocol. It exposes richer events but adds a persistent protocol/session lifecycle
   before even one measurement. Consider it if complete no-tool attestation is required and exec cannot provide it.
3. One ephemeral `codex exec` per claim, version-pinned parsing, strict output and completion checks, temporary
   working directory, supported tool disables, process-group timeout, unknown cost and usage preservation on
   rejected replies. Smallest independently testable subscription integration; not yet a production judge.

## Decision
Implement candidate 3 as a separate benchmark adapter; leave production routing and all standard bars unchanged.
No model is called optimal: no Codex model comparison has been measured here. Match the existing escalation
confidence/citation gates without treating self-reported confidence or a copied span as proof of entailment.
Production converts confidence to `f32` and stores the 0.8 bar in `f32`; compare after binary32 rounding and test
the predecessor and halfway rounding boundary independently. Comparing Python doubles would change routing.
Reject malformed/duplicate/non-finite JSON, unknown stream events, any emitted tool/error item, incomplete or
multiple turns, nonzero exit, mismatched final output and missing reason. Keep usage from a completed turn even
when the ruling is rejected. Never retry a paid call automatically or substitute missing price with zero.

Test these faults offline with canned streams and real fake processes, including a descendant holding output pipes.
Use the repository's fmt/clippy/tests and zero-missed Rust decision mutations, plus targeted Python fault injection.
An explicit `isolation_validated: false` remains until a separate endpoint/isolation experiment succeeds: JSONL alone
cannot certify tool-free operation. Do not use this adapter for a 99% confirmation yet. Next, build the whole-cascade
runner around actual production decisions with hashed run identity; do not bolt it onto the old unsafe resume logic.
