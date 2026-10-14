# Checking study notes with EXPIRA

`ej-study` is a separate CLI and Rust library that prepares study text for the existing
`ej check` binary. It does not implement a judge, fit a policy, change thresholds, or
modify the benchmark. All findings come from the production cascade.

## Build and run

From the repository root:

In the current managed cloud workspace, first run
`source /workspace/setup/activate.sh` in the shell to activate the pinned Rust
toolchain and the already installed ONNX shared library. This external helper is
documented in `docs/research/2026-10-11-cloud-runtime.md`. Other machines need their
normal EXPIRA/ONNX runtime setup. `ej-study` itself introduces no model download.

```sh
cargo build --locked -p ej-cli -p ej-study
./target/debug/ej-study \
  --text /path/to/notes.txt \
  --source /path/to/source.txt \
  --source-id 'https://original-source.example/document' \
  --source-date 2026-10-14 \
  --today 2026-10-14 \
  --ej ./target/debug/ej > /path/to/report.json
```

The paths and URL are placeholders. Supply the **actual date the source was opened**
and today's date; the command does not infer or invent them. `--source-id` defaults
to the source file path. `--max-age DAYS` forwards an optional age limit to EXPIRA.
Both input files must be UTF-8 text. Extract text from PDF/HTML outside this wrapper,
and inspect the extraction for lost tables, symbols or context. Source text is passed
intact as one verbatim evidence quote per candidate, and retained once in the report.
An empty source is passed to EXPIRA so it can return its own verdict.

To reuse an existing DeBERTa setup, append:

```sh
  --nli /path/to/existing/model --policy /path/to/existing/policy.json
```

The policy must already exist; this command never creates or changes it. The original
binary controls eligibility (`judges.nli.passed_v1`) and calibration. A model without
a passing policy is skipped by that binary, with its diagnostic on stderr. Without
an eligible model, paraphrases can remain `unsure`. This initial wrapper exposes the
existing local route; it does not enable paid escalation or the diagnostic Codex adapter.

`--ej` defaults to `ej` on PATH. `--help` prints the complete CLI usage.

## Candidates and results

By default, every nonempty input segment is checked. Boundaries are newlines or `.`,
`?`, `!` followed by whitespace/end of input. Decimals and URLs without intervening
whitespace stay intact. This is mechanical segmentation: it does not classify factual
sentences or split compound statements into atomic facts. Headings, questions and list
markers are retained too. Abbreviations, closing quotes and wrapped lines can produce
awkward segments.

For exact claim boundaries, put **one complete factual claim per line** and add
`--lines`. Each line is kept intact, including abbreviations and punctuation. Review
that every factual statement in the original explanation is represented; a compound
claim gets a single verdict unless you separate it yourself. Keep necessary context
in each claim. No extraction model rewrites your text.

The JSON report contains `source`, `today`, `segmentation`, and `claims`. Each claim has:

- `candidate`: unique positional ID, original text, and zero-based UTF-8 byte offsets
  `start`/`end` into the input (`end` is exclusive).
- `finding`: original EXPIRA `claim_id`, `verdict`, `layer`, `confidence`, and `reason`.
- `flagged`: the inverse of the existing `Verdict::passes()` rule.

| EXPIRA verdict | Review flag |
|---|---|
| `backed`, `supported` | false |
| `unbacked`, `mismatch`, `stale`, `unsure` | true |

Exit **0** means all candidates pass. Exit **1** means the report was produced and
at least one candidate needs review. Exit **2** means input/setup/protocol failure;
no success report is emitted. Failed processes, invalid JSON, incomplete/duplicate/
unknown IDs, invalid confidences and inconsistent exit statuses are rejected.
Results are joined by ID and emitted in the original study order.

A review flag is not proof that a claim is false: evidence may be missing, old or
insufficient to decide. A pass is the existing engine's judgement against the supplied
source, not independent validation of the source or a new accuracy guarantee. The
wrapper sends the full document; any model context limit still applies. For large
documents, supply a relevant self-contained section and identify that section in
`--source-id`, preserving its surrounding conditions and qualifications.

## Using this during study or research

When pulling sources for an explanation:

1. Save the inspected source text, original URL/document and date opened. Check facts
   against an appropriate authoritative source, including its version and context.
2. Save the draft explanation. Use the sentence mode for an initial scan, or prepare
   reviewed complete claims one per line with `--lines`.
3. Run `ej-study` and inspect every flagged claim and reason before using it to learn.
   Retrieve better evidence, correct the draft from that evidence, or leave the claim
   unresolved. Rerun the corrected text; do not describe an `unsure` claim as verified.
4. Keep the report with the source snapshot so the check can be inspected later.

For several documents, run separately against each appropriate source/section.
The integration does not fetch URLs, rank passages, merge sources or train on notes.

## Rust API and tests

Add a path dependency on `crates/ej-study`, then call
`ej_study::check_study(text, &source, mode, &engine)` using `ej_core::Evidence` for
the source snapshot, `Segmentation::{Sentences, Lines}`, and `Engine` for the binary,
checking date, optional age limit and optional existing `LocalModel`.
It returns `Result<StudyReport, String>`; `report.has_flags()` follows the engine's
pass rule. The public types and functions carry API documentation.

```sh
cargo test --locked -p ej-study
cargo test --workspace --locked
# Optional real-engine parity test (build first, as above):
EJ_STUDY_ENGINE="$PWD/target/debug/ej" cargo test --locked -p ej-study --test real_engine -- --ignored
```

New tests live in `tests/integration.rs` and `tests/real_engine.rs`. Process fixtures
test the integration contract offline; their canned findings are not model measurements.
The optional smoke checks real-binary parity, empty/stale evidence and skipped-model
eligibility with a synthetic non-passing policy. Results are recorded in
`../../docs/bench/2026-10-14-study-integration.md`.
