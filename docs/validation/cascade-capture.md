# Offline production capture

`tools/cascade_capture.py` captures one fixed deterministic pre-escalation batch from the actual `ej check` binary.
It is the first runner component, not a complete cascade or a certification procedure. It never enables a local model
or subscription, retries, resumes or reports a v1 pass. `certified` is always false. Research and measured results:
`docs/research/2026-10-11-offline-cascade-capture.md`, `docs/bench/2026-10-11-offline-cascade-capture.md`.

## Inputs and invocation
Data uses the frozen `{claim,quote,label}` JSONL format; labels are `supports`, `contradicts` or `unsure`. Empty quotes
are retained. Every data line has a companion JSONL entry `{case_id,row_idx,row_sha256}`: `row_idx` is a unique original
source-row identifier, `case_id` may repeat, and `row_sha256` hashes exact line bytes including the newline. Bind
companions against original source pages before use. The runner checks the declared row-byte binding; it cannot
authenticate case IDs, original source text, licence rights or independent sampling from hashes alone.

Sampling is a nonempty JSON object describing the actual population, selection and limitations. It is recorded and
hashed, not attested. Do not relabel the already inspected VitaminC test set as untouched confirmation.

```sh
cargo build --locked -p ej-cli
python3 -I tools/cascade_capture.py \
  --data DATA.jsonl --identities IDENTITIES.jsonl --sampling SAMPLING.json \
  --ej target/debug/ej --repo . --today 2026-10-11 --out NEW_RUN_DIRECTORY
```

The source field defaults to an explicit diagnostic placeholder. Evidence dates use `--today` as an **opened** date,
not an original publication date. `--source` changes that diagnostic metadata; `--max-age` and a bounded `--timeout`
are optional. Inputs are capped at 32 MiB per file and 10,000 cases. The fixed input set is never shortened on failure.

## Outputs and failure semantics
- `manifest.json` is written before execution. It binds raw data/companion/sampling bytes, request bytes, ordered
  identities, executable snapshot, runner/helper, Python version, tracked source bytes, Git head/dirty state and options.
  Binary/source correspondence and sampling are explicitly unattested; the manifest is not a reproducible-build proof.
  Untracked source bytes are not covered by the tracked fingerprint; runner/helper are independently hashed.
- The executable and label-free request are snapshotted in a temporary working directory. Only production Claim
  fields enter the request. The process group is killed and reaped on timeout, without automatic retries.
- Valid mixed findings have CLI exit 1; all-backed findings have exit 0. Setup failure, timeout, invalid JSON/schema,
  invalid encoding, excessive captured output, reordered/missing/foreign findings or impossible routes invalidate the
  whole batch. Every input then appears as an operational failure, with no successful-prefix accuracy calculation.
- `report.json` is published by atomic rename after validation. An absent report means an incomplete run. Existing
  directories are refused even if they hold only a manifest; there is no resume implementation. Runner exit 0 means
  capture completed, exit 1 means a recorded operational failure, and exit 2 means preflight/publication failed.
- The 1 MiB helper limit bounds stdout bytes accepted/read after process completion. Temporary process-output files
  may grow until the timeout; this is not a disk quota. `captured_stdout_sha256` hashes decoded captured stdout, which
  is empty when output-limit/encoding failures prevent capture, rather than hashing uncaptured provider bytes.

## Measurement limits
Report stated/block counts, abstentions, operational failures, group count, pre-escalation coverage and the complete
verdict/reference-label table. Binary block correctness pools contradicts/unsure under v1; it does not turn unbacked
or stale into semantic contradictions. Wilson(z=1.96) values are nominal compatibility diagnostics, not population
guarantees. There is no candidate selection, exact confirmation, independence attestation or label audit in this tool.

ECE, fresh stress and quiet local per-ruling p95 stay unmeasured; batch wall time includes startup/load. Costs/tokens
stay unknown, with subscription attempts explicitly zero. Do not substitute an external Codex chain for an actual
production route. Structured score/timing traces, eligible-local artifact qualification, the Rust subscription
transport and the separate sealed confirmation procedure remain subsequent tested changes.
