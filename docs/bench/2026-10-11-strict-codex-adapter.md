# Strict Codex subscription adapter: diagnostic integration

Research: `docs/research/2026-10-11-strict-codex-adapter.md`. This is a transport/testing improvement, not a new
judge adoption or a 99% result. Production routing, policies, prices, standard bars and frozen splits are unchanged.

## Data and machine
- Synthetic protocol fixtures only; no calibration/test/stress/audit examples inspected or sent to the subscription.
- Cloud Linux, cgroup quota 4 CPUs, 32 GiB memory; Python 3.12.14, Rust 1.97.0, codex-cli 0.159.2.
- Offline tests ran alongside Rust checks/mutations. Their timing is not a quiet production latency benchmark.

## Measured before / after
Exercised the **actual legacy Python parser and summary** with a fake subprocess: exit 1, plausible Supports JSON,
confidence 0.79, exact citation, absent price. No real CLI invocation was made in this comparison.

| Synthetic fault | Legacy escalation script | New Codex diagnostic adapter |
|---|---|---|
| Nonzero exit with plausible ruling | Counted as acted Supports | Unsure, not acted; error recorded |
| Exit 0, confidence 0.79 with exact citation | Counted as acted Supports | Not acted, matching production's 0.8 bar |
| Missing price | Summary reports $0 | `cost_usd: null` (unknown) |
| Rejected reply with completed-turn usage | No completion protocol | Reported usage retained |

Scratch reproduction: `/workspace/setup/adapter-before-after.json`. These are fault fixtures, not accuracy or
price measurements. The legacy script is retained for historical runs; it cannot establish a new cascade claim.

## Offline verification
```sh
python3 -I -m unittest discover -s tools -p 'test_*.py'
python3 -I tools/check_codex_faults.py
source /workspace/setup/activate.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo deny check
PROPTEST_CASES=256 cargo mutants -j 4 --timeout 60 \
  -p ej-core -p ej-evidence -p ej-judge -p ej-delegate -p ej-habits -p ej-bench \
  --exclude crates/ej-bench/src/main.rs --output /workspace/setup/adapter-mutants
```

The Python suite has 22 passing tests. Checks cover malformed/duplicate/non-finite JSON, deep nesting,
schema errors, final-file/type mismatch, missing/reordered/multiple turns, unknown/tool/error events, item lifecycle,
citation rejection, independent binary32 confidence boundaries, invalid token totals, usage retention on rejection
and timeout, missing/version-mismatched CLI, output/encoding limits and descendant termination. All process tests
use fake executable files, not a model. CI now runs these tests and the declared family of 15 targeted injected
Python faults. This family is not exhaustive Python mutation coverage. The first injection run left one survivor
(negative cache-write tokens); the added independent test catches it. Final counts are recorded below.

Rust workspace tests report 117 passes (ONNX model smoke still returns early without EJ_NLI_DIR); fmt/clippy
and cargo-deny pass. The supply-chain check retains existing duplicate-version warnings. No model inference is
claimed from a passing smoke test without weights.

## Subscription probe and cost
Command shape (input was `The gate is locked.` / the same quotation):
```sh
python3 -I tools/codex_adapter.py --cli /opt/codex/bin/codex \
  --model gpt-6-luna --effort low --timeout 45 \
  --state-dir /workspace/setup/adapter-smoke/state < synthetic-input.json
```
The installed release's model catalogue lists this model and supports low effort. It was chosen for a bounded
connectivity diagnostic, not because its claim-checking quality or cost is proven optimal. Authentication stays in
the managed CODEX_HOME. The platform approved the runtime installation-id write; model sandbox remained read-only.

Two synthetic inference attempts: the first rejected after 2,851.684465 ms; a second after adding fixed diagnostic
capture rejected after 2,226.436280 ms, exit 1, `authentication_failure`. Neither returned usable final JSON or
completed-turn usage. Costs and tokens are **unknown**, not zero. Calibration/held-out subscription calls: **0**.
No retry loop or further paid probe was run. Scratch records: `/workspace/setup/adapter-smoke/result.json` and
`result-diagnostic.json`. `inference_attempted` records a CLI attempt, not proof the server ran inference.

## Limits and next experiment
- All v1 semantic gates are **unmeasured for Codex**. The prior whole-layer-1 baseline remains failing; see
  `2026-10-11-rigorous-99-baseline.md`. This transport change makes no accuracy, coverage, ECE or latency adoption claim.
- Exec JSONL omits some tool events; supported disables and an empty temporary directory reduce exposure but do not
  prove no-tool isolation. Replies carry `isolation_validated: false`. Do not use them for a 99% confirmation.
- No whole-cascade runner, resume-identity manifest, local policy/model measurement, independent label audit or new
  untouched confirmation is delivered by this adapter. The protocol specification remains `docs/validation/general-99.md`.
- Next implement the runner against actual `ej check` decisions with dataset/code/policy/prompt/schema identities
  and failure-accounting tests. Restore the managed authentication through the platform before any live calibration
  experiment; do not request credentials. Then measure at most 10 calibration cases serially, with explicit unknown
  prices where appropriate. Production Rust transport and complete isolation require separate tested changes.

## Final mutation results
Full Rust run: 856 generated, 794 caught, 62 unviable, **zero missed, zero timeouts**, exit 0
(cargo-mutants 27.1.0). Rust decision code is unchanged. The Python targeted family: **15 injected, 15 caught,
zero missed/unviable/timeouts**, exit 0. Both Python tests and targeted fault injection are required by CI.
Scratch logs and outcomes are under `/workspace/setup/adapter-*`.
