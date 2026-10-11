# Habits

Owner, 2026-10-11: self-reports and recommendations contain only adopted (good) habits and measured results, with the
research behind them. Retired (bad) habits are never recommended, never repeated, and are checked against before any
report or recommendation goes out. `cargo test` checks this file's structure; `ej_habits::recommendable` enforces the same
rule in code for logged habits. Habits move here only with evidence; a retired habit stays retired.

## Adopted

- **Thresholds are fitted, never hand-set.** Calibrate a model's score on a calibration split, choose act thresholds whose
  95% Wilson lower bound meets the target, report on a separate test split. Evidence: the hand-set LAYA bar was right on
  257 of 378 "stated" calls (docs/bench/2026-10-11-round1-laya-vs-deberta-calibrated.md).
- **Measure every model build before trusting it.** Evidence: the published 8-bit DeBERTa-v3-large scored MNLI AUROC
  0.935 against 0.990 for the fp32 build (docs/bench/2026-10-11-round2-stronger-verifiers.md).
- **Test strength is measured.** Property tests on invariants and zero surviving mutants in the decision crates. Evidence:
  mutation testing exposed 86 untested faults and two real layer-1 bugs (HANDOFF.md, test-rigour section).
- **Research before change, licence from the source.** A note in docs/research/ with links before a fix; licences read
  from the model or dataset card or LICENSE file, never recalled.
- **One change at a time, kept only if measured better,** on the same frozen splits, logged in docs/bench/.
- **Frozen data is reproducible; grouping and overlap are checked separately.** Fetch byte-identical files and split
  contrastive data by case, never by row (tools/fetch_general.py). Retain identities and measure text overlap before
  claiming independence. Evidence: disjoint case IDs still left 21 exact claim–quote pairs across cal/test
  (docs/bench/2026-10-11-rigorous-99-baseline.md). Preserve old frozen splits and report their limitations.
- **Compute a number before stating it.** Any count, bound or price in a report comes from a calculation or a measurement
  shown in the same place.
- **Loops are bounded and toolchains pinned.** Every scan has an iteration bound; CI and local runs use the same pinned
  compiler and lints.
- **Friction in the structure is a defect.** When the repo's structure makes improvement annoying, slow or error-prone,
  it is researched and restructured in small tested steps, not worked around (owner, 2026-10-11).
- **Latency is measured on a quiet machine,** and figures taken under load are labelled as such.

## Retired

- **Treating CLI stdout as a verdict without checking completion, or missing price as zero.** Why retired: the
  legacy escalation script ignores process exit, omits the production confidence gate and totals unreported cost
  as $0. Evidence: docs/bench/2026-10-11-strict-codex-adapter.md. Replaced by: strict process/turn/schema/citation
  checks and explicit unknown costs in the offline-tested Codex diagnostic adapter. The legacy Claude script
  remains historical and must not establish new cascade or 99% claims; whole-cascade integration is still open.
- **Treating disjoint case IDs as proof of leak-free data.** Why retired: different cases can repeat the same text;
  grouping alone misses overlap and does not make contrastive rows independent. Evidence: 21 exact / 24 normalised
  claim–quote pairs cross calibration/test despite zero shared case IDs, and 3,095 test rows represent only 935 cases
  (docs/bench/2026-10-11-rigorous-99-baseline.md). Replaced by: separate reproducibility, grouping and overlap checks;
  untouched confirmation with a declared sampling unit (docs/validation/general-99.md), preserving frozen files.
- **Acting on a raw, uncalibrated score with a hand-set bar.** Why retired: LAYA's 0.6 bar passed many unsupported claims
  (257/378 right) and 51 of 200 negated ones. Evidence: docs/bench/2026-10-11-round1-laya-vs-deberta-calibrated.md.
  Replaced by: fitted thresholds (Adopted).
- **Trusting a published quantised model without measuring it.** Why retired: the 8-bit large checker lost most of its
  accuracy. Evidence: docs/bench/2026-10-11-round2-stronger-verifiers.md. Replaced by: measure every build.
- **A loose relative tolerance for "same figure".** Why retired: 1e-9 made 1,000,000,001 equal 1,000,000,000.
  Evidence: crates/ej-evidence tests `figures_compare_exactly_but_tolerate_float_noise`. Replaced by: 1e-12 plus tests.
- **Letting a less precise quote back a more precise claim.** Why retired: a month-only quote backed a claimed day.
  Evidence: test `date_precision_and_kind_are_respected`. Replaced by: a claimed day needs the day in the evidence.
- **A judge that cannot see polarity acting on negated text.** Why retired: the overlap judge passed 5 negated claims.
  Evidence: stress run, HANDOFF.md. Replaced by: abstain whenever negation is present.
- **Magic numbers standing in for a rule.** Why retired: a self-gating bar of 0 behaved exactly like -1 (equivalent
  mutant). Evidence: `Judge::self_gated()` commit. Replaced by: an explicit flag.
- **Moving a fitted threshold onto data it was not checked on.** Why retired: overlapping thresholds were nudged
  (no = yes - epsilon), breaking the accuracy promise. Evidence: property test `fitted_thresholds_meet_target_in_sample`.
  Replaced by: search each side only where it was validated.
- **Unbounded scan loops.** Why retired: a stalled index looped forever and exhausted CI memory. Evidence: CI mutants job,
  commit "Layer 1 date scan has a hard iteration bound". Replaced by: bounded loops.
- **An unpinned toolchain.** Why retired: CI's newer clippy failed code that passed locally. Evidence: commit "Pin the Rust
  toolchain". Replaced by: pinned toolchain, upgraded deliberately.
- **Leaving an unproven judge in the decision path.** Why retired: the overlap placeholder still decided in `ej check`
  and passed 9 of 82 unsupported claims on VitaminC. Evidence: docs/bench/2026-10-11-round3-general-99.md. Replaced by:
  only judges that passed the standard decide; everything else escalates.
- **Stating figures or licences from memory.** Why retired: a cases-needed count was given as ~460 (computed: 381) and a
  dataset licence was written from memory (its card declares none). Evidence: docs/research/2026-10-11-general-layer-99.md,
  docs/research/2026-10-11-laya-and-the-decision-block.md. Replaced by: compute or read the source first.
