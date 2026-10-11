# Actual production capture: integrity and unchanged semantic baseline

Research: `docs/research/2026-10-11-offline-cascade-capture.md`. Usage/limits: `docs/validation/cascade-capture.md`.
The kept improvement is a tested measurement component. No decision logic, model, policy, standard bar, frozen data
or reference label changes; no improvement in semantic accuracy or v1 pass is claimed.

## Before / after
Before: standalone production baseline files, no supported strict input identity or complete-outcome capture runner.
After: one offline actual CLI batch with raw-byte identities, immutable executable/request snapshots, complete
validated findings or all-input operational failures, nominal statistics and explicit unmeasured gates. Existing run
directories cannot be overwritten or resumed. This is deterministic pre-escalation capture, not a complete Codex cascade.

## Data and measurement
Used the previously frozen VitaminC test JSONL outside Git, preserved unchanged. Verified every source-row identifier,
case ID and exact frozen line against saved original API pages before adding `row_sha256` to a **new** companion file.
The inspected old test set is diagnostic; 3,095 rows contain 935 cases, and previously measured cross-partition overlap
remains a limitation. The caller's sampling declaration acknowledges this. No individual error cases were read for tuning.

- Test SHA-256: `cb223f926c408361de98fa04f51714b115c899e3595140505c9e9db1ff06a6f9`.
- New companion: `764f0c3cf58adc99597a632d0978a59217d2f388e9e40cf92ddb7b41237b9e60`.
- Sampling declaration: `0f7c804779dee15b94376413ef9e864ccad57c86816c31f0ff4e563b01d746bd`.
- Built executable: `b2ed1bbebf55d1c6f274c0f39962ae96577a4c15b71ccf374443436df73a162a`.
- Final run: `7d7e069213bc60ffa214a441db067186f029eab4727a1137cff9e880bb8e6e34`.
- Runner: `6ddccb1716392e954e51d0f69e4c351589b4b35dc806d74ffc49ea659255522d`.
- Source head: `42d466b1a0f4bb9a0e8ff6a646d96b2c4961c133`, dirty worktree recorded; runner/helper hashes in manifest.

Actual `ej check` exit 1 was accepted as a valid mixed batch. All 3,095 findings match the prior actual CLI capture
on verdict, layer, confidence and reason; only caller-generated claim IDs differ. Zero operational failures:

| Diagnostic quantity | Result |
|---|---:|
| Stated correct / acted | 25 / 25 |
| Nominal v1 stated Wilson lower bound | 0.8668035060468212 |
| Blocks correct / acted | 292 / 411 |
| Nominal block Wilson lower bound | 0.6648244757368196 |
| False blocks on support references | 119 |
| Abstentions | 2,659 |
| Pre-escalation coverage | 0.1408723747980614 |

Before/after semantic results are identical to `2026-10-11-rigorous-99-baseline.md`. ECE, fresh stress and quiet local
p95 remain unmeasured, as do whole-cascade escalation and the exact supplement. The measured batch wall time is
779.720 ms including snapshot/startup/load; it is not per-ruling quiet latency. All conclusions are nominal diagnostics,
not independent-population accuracy. Source/date metadata are explicit placeholders and diagnostic opened dates.

Reproduction (scratch retains the prepared companions and immutable outputs):
```sh
source /workspace/setup/activate.sh
cargo build --locked -p ej-cli
python3 /workspace/setup/capture-prepare.py
python3 -I tools/cascade_capture.py \
  --data /workspace/setup/vitaminc/vitc_test.jsonl \
  --identities /workspace/setup/vitaminc/test-capture-identities.jsonl \
  --sampling /workspace/setup/vitaminc/capture-sampling.json \
  --ej target/debug/ej --repo . --today 2026-10-11 \
  --out /workspace/setup/cascade-capture-NEW
```

## Verification and cost
Cloud Linux, Rust 1.97.0, Python 3.12.14, 4 CPU quota, 32 GiB memory. Baseline fmt/clippy/locked workspace tests
passed before implementation (117 reported Rust passes; ONNX smoke returns early without weights). The new 19 offline
tests pass (41 Python tests total), including real compiled CLI all-pass/mixed/empty-quote controls and an unproven policy skipping a nonexistent
model before loading. Fake processes exercise strict identity/schema, exit/status, partial output, encoding/output-limit,
snapshot mutation, label stripping, atomic reports, refusal to overwrite and descendant termination on timeout.
Independent Fraction score inversion checks Wilson boundaries at .99/.95 without reproducing its formula.

The targeted runner fault family has an independently passing unmodified-source baseline; all 20 declared faults were
caught, zero missed/unviable/timeouts. This is targeted fault injection, not exhaustive Python mutation coverage.
No new subscription or model calls: attempts zero, usage/cost unreported by this offline component (null). Historical
authentication-failure costs remain unknown. No new price is invented; no semantic or isolation certification is claimed.

Final fmt/clippy/locked workspace tests passed: 117 reported Rust passes, ONNX smoke still returns early. cargo-deny
passed with existing duplicate-version warnings. All 15 existing adapter faults and 20 new capture faults caught,
zero missed/unviable/timeouts. The capture fault harness separately verifies its unmodified-source baseline passes.
Full Rust mutation run: 856 generated, 794 caught, 62 unviable, zero missed, zero timeouts, exit 0; start
2026-10-11T21:47:16.403736544Z, end 2026-10-11T21:58:33.624637618Z. Artifacts:
`/workspace/setup/cascade-mutants/mutants.out/outcomes.json`, `cascade-quiet-python-tests.log`,
`cascade-final-adapter-faults.jsonl`, `cascade-final-runner-faults.jsonl`, `cascade-capture-final/`.

The first combined Python run (39 tests before two additional capture tests) had one failure in the existing adapter
timeout fixture: its descendant-startup PID file was absent under concurrent Rust compilation/mutation load.
The unchanged existing test passed in the quiet 41-test run (5.145 s total). No deadline, assertion or test was weakened.
New capture timeout checks additionally require a spawned descendant and verify it is dead, preventing a missing
side effect alone from counting as termination evidence. Interruption leaves a manifest without a final report;
non-root repository paths are rejected rather than fingerprinting only a source subtree. The final frozen replay
again matches all 3,095 prior production findings, with no semantic change or paid calls.
