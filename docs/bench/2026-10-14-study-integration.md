# Study-text integration verification (2026-10-14)

## Scope
One separate integration: `crates/ej-study`, which converts supplied text and a source
snapshot into the existing `Claim` input, calls production `ej check`, and preserves
its findings. No decision-block code, standard, judge thresholds, model weights,
calibration policies/data or existing decision tests are changed. This log records
integration behaviour, not accuracy on study material or any model comparison.

## Environment and commands
Existing cloud Linux workspace; Rust/Cargo 1.97.0 and the previously installed ONNX
Runtime 1.22.0 native library. Activate `/workspace/setup/activate.sh` before commands
in this environment. No new model or subscription calls are needed.

```sh
source /workspace/setup/activate.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --locked -p ej-cli -p ej-study
EJ_STUDY_ENGINE="$PWD/target/debug/ej" cargo test --locked -p ej-study --test real_engine -- --ignored
cargo deny check
python3 -I -m unittest discover -s tools -p 'test_*.py'
python3 -I tools/check_codex_faults.py
cargo mutants -j 4 --timeout 60 -p ej-core -p ej-evidence -p ej-judge -p ej-delegate -p ej-habits -p ej-bench -p ej-study --exclude crates/ej-bench/src/main.rs
```

## Initial-checkout checks (42d466b)
Baseline formatting, clippy and locked workspace tests passed before editing. Initial
formatting and clippy passed; the locked workspace suite reports **127 passed, zero
failed**, plus one explicitly ignored test requiring the production executable.
The new integration suite contributes **10** offline tests. The ignored production
parity test was then run explicitly and **passed**. CI now builds the production
binary and runs it explicitly too, and includes `ej-study` in mutation testing.
Existing Python checks report **22 passing tests**; all **15** targeted adapter fault
injections are caught. cargo-deny passes with the existing duplicate-version warnings.

Integration tests cover unchanged source/claim forwarding (including literal shell
characters and paths containing spaces), date/input validation, candidates with
Unicode/decimal/repeated/trailing text, explicit line boundaries, every verdict,
out-of-order IDs, missing/extra/duplicate/unknown IDs, invalid output/confidence,
engine failures, status disagreement, cleanup, CLI options and exit codes.

The first integration mutation run exposed a missing positive CLI test for paired
`--nli`/`--policy` options. Added that test, including zero-day age forwarding and
source-path defaults, before the final full mutation run. No decision logic was
changed to address the test gap.

A focused final run on `ej-study` (`cargo mutants -p ej-study -j 1 --timeout 30`)
exited 0: **48 generated, 45 caught, 3 unviable, zero missed, zero timeouts**.
Its output is `/workspace/setup/study-final-integration-mutants/mutants.out/outcomes.json`.

The initial full run exhausted `/tmp` and stopped one worker. Its results are not
treated as a successful complete run. Reran 104 disk-affected mutants and the one
unattempted mutant on the larger workspace disk; all reruns passed. A name-by-name
join covers all 904 listed mutations: 839 caught, 65 compiler-unviable, zero missed
and zero timeouts. The complete-set assertion and summary are saved in
`/workspace/setup/study-valid-mutation-summary.json`.

## Current-main integration checks (d208133)
Before publishing, remote `main` was found to have advanced since environment setup.
Rebased the integration onto `d208133`, preserving all upstream decision changes and
existing CI checks. Only the CI additions and appended handoff section needed conflict
resolution; the wrapper's production interface and implementation required no change.
The integration diff against this base has no decision-block or existing test files.

Current-base formatting and clippy pass, and locked workspace tests report **135
passed, zero failed**, plus the optional real-engine test. That test was run explicitly
against the rebuilt current production binary and **passed**. The base itself has
green check and mutation jobs:
https://github.com/amadeus1611/expira-judgement/actions/runs/38509510797
The integration's full current-base mutation check uses `TMPDIR` under
`/workspace/setup/study-latest-main-tmp`, `PROPTEST_CASES=256` as in CI, and
`--copy-target true` to reuse existing build artifacts. All source changes remain
limited to the integration and its wiring/documentation.

The full current-base run exited **0**: **961 generated, 896 caught, 65
compiler-unviable, zero missed, zero timeouts**, with no disk errors. Results:
`/workspace/setup/study-latest-main-mutants/mutants.out/outcomes.json`.
The source code and existing tests in all decision crates are identical to the
current-main base; mutation changes occurred only in disposable copies.

The current-base Python suite passes **41 tests**. The first capture fault-check
baseline failed while mutation workers were active (the script correctly refused to
score injected faults against a failing baseline). Its log is retained at
`/workspace/setup/study-current-main-capture-faults.jsonl`; existing tests are unchanged.
After the mutation workers finished, the quiet rerun exited **0**: **41 Python tests
passed**, all **15 adapter faults** caught, and all **20 capture faults** caught after
a passing unmodified baseline. Quiet logs use the `study-current-main-*-quiet`
prefix under `/workspace/setup`. No existing test was changed or weakened.
Final formatting, clippy and locked workspace tests also pass before push. Compared
all 25 tracked files in the core/CLI/evidence/judge/NLI/benchmark/config paths with
current main: identical bytes; the integration changes none of them.

## Real-engine comparison
`tests/real_engine.rs` uses fresh synthetic prose, not an exam source or calibrated
dataset. For each case, it runs the wrapper, builds the identical ordinary claims
JSON, and compares every original `Finding` against direct `ej check` output.

| Supplied evidence/settings | Observed findings in original note order |
|---|---|
| Synthetic document with one exact statement and an exercise duration | `backed`, `mismatch`, `unsure` |
| Same source, opened yesterday, with `--max-age 0` | `stale`, `stale`, `stale` |
| Empty document | `unbacked`, `unbacked`, `unbacked` |
| Original document with a synthetic non-passing policy and missing model directory | `backed`, `mismatch`, `unsure`; original binary skips model loading |

All comparisons passed. A separate invocation of the actual `ej-study` CLI against
the synthetic document returned exit 1 and the expected `backed`/`mismatch`/`unsure`
report, with only the latter two flagged. Scratch report:
`/workspace/setup/study-cli-smoke/report.json`. The cloud shared-library activation
is necessary: without it the original binary exits 127, which the wrapper correctly
reports as setup failure (exit 2), rather than emitting a successful check.

## Limits
No actual DeBERTa inference, eligible-model coverage, live escalation, Security+
accuracy, throughput, certification or cost is measured here. Synthetic fixtures
verify transport parity and flags only. Mechanical sentence candidates can contain
multiple facts or awkward abbreviation boundaries; reviewed one-claim-per-line input
provides explicit boundaries. The source snapshot's reliability and the production
model's context limits remain relevant to interpreting its original verdicts.
