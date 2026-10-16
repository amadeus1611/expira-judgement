# A second local judge from a different model family (2026-10-16)

## Problem
Both local judges are DeBERTa-v3; requiring their agreement did not raise the confirmation lower bound (0.9614 vs
0.9678 for large alone, docs/bench/2026-10-16-reproduction-and-agreement.md), consistent with correlated errors.
A judge with a different architecture and pre-training may err on different rows.

## Sources
- RoBERTa-large-MNLI: https://huggingface.co/FacebookAI/roberta-large-mnli (Liu et al. 2019, arXiv:1907.11692)
- ONNX export of the same weights: https://huggingface.co/Xenova/roberta-large-mnli
- BART-large-MNLI: https://huggingface.co/facebook/bart-large-mnli, ONNX: https://huggingface.co/Xenova/bart-large-mnli
- Ensemble diversity and error correlation: Kuncheva & Whitaker 2003, "Measures of Diversity in Classifier Ensembles".

## Licence
roberta-large-mnli: MIT (model card). bart-large-mnli: MIT. The Xenova repositories are format conversions of these
weights; no extra terms stated. Weights are downloaded at a pinned revision and never enter git.

## Candidates
1. `Xenova/roberta-large-mnli@f44e4108…` fp32 `onnx/model.onnx` (no token_type_ids; ej-nli feeds inputs by name and
   reads labels from `id2label`, so no code change).
2. `Xenova/bart-large-mnli` fp32 (encoder-decoder; larger and slower).

## Decision
Measure candidate 1 first, alone and as `large+roberta` with `tools/local_cascade.py --gate --confirm`. Keep only if the
confirmation lower bound beats 0.9678 with thresholds fitted on calibration alone. Try BART only if RoBERTa helps
in agreement (evidence that family diversity pays).
