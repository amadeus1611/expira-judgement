# LAYA and the decision block (2026-10-11)

## Problem
LAYA (the owner's engine's local judge) was never benchmarked or calibrated, and the engine acts on fixed raw thresholds
(support yes >= 0.6, no <= 0.3). Question: how should the local judges, their thresholds and Claude escalation be set so the
whole block meets a stated error bound?

## Sources
- LAYA package page and README (architecture, checkpoints, calibration, limits): https://pypi.org/project/laya/
- DeBERTa-v3 NLI cross-encoder used as the second candidate (ONNX export included): https://huggingface.co/cross-encoder/nli-deberta-v3-xsmall
- MNLI (labelled premise/hypothesis pairs, used as public test data): https://huggingface.co/datasets/nyu-mll/multi_nli
- FEVER as NLI (claim/evidence pairs, used as public test data): https://huggingface.co/datasets/pietrolesci/nli_fever
- Platt scaling (calibrating a classifier's scores with a fitted logistic curve): https://en.wikipedia.org/wiki/Platt_scaling
- Wilson score interval (confidence bound on a rate, used so thresholds are not fitted on lucky samples): https://en.wikipedia.org/wiki/Binomial_proportion_confidence_interval#Wilson_score_interval
- Anthropic API docs for the escalation call (structured outputs, effort, refusal fallbacks): https://platform.claude.com/docs

## Licence
- LAYA: Apache-2.0 (package page). The checkpoints' own licence must be checked on their model cards before shipping them.
- cross-encoder/nli-deberta-v3-xsmall: Apache-2.0 (model card, checked via the HF API).
- MNLI: card lists cc-by-3.0, cc-by-sa-3.0, mit and other (varies by genre); local evaluation only, not redistributed.
- FEVER (pietrolesci/nli_fever copy): the card declares no licence; local evaluation only, not redistributed. Check the original FEVER release's terms before any other use.

## Candidates
1. Keep LAYA with its current raw thresholds (baseline).
2. LAYA with Platt calibration and thresholds fitted to a bounded error target.
3. DeBERTa-v3-xsmall NLI, same fitting, run in Rust through ONNX Runtime.
4. Mean of the two calibrated scores.
Documented LAYA limits to test directly: over-confidence as released (mean calibration error 0.466, 0.081 after fitting,
per its README), weakness on negation, yes/no answers swayed by option wording.

## Decision
Pending the measured run (scores on FEVER and MNLI calibration/test splits, plus a figure/negation stress set).
Each candidate is adopted only if it passes the standard in `crates/ej-bench/src/standard.rs`; results go to `docs/bench/`.
