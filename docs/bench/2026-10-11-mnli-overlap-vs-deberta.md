# Bake-off run 1: overlap placeholder vs DeBERTa-v3-xsmall NLI (2026-10-11)

**This is not the decision run.** LAYA was not available in this container and the small constrained model is not wired yet,
and the cases are public MNLI, not ledger claims. It shows that the harness works and gives a first CPU latency figure.

## Setup
- Cases: first 500 rows of MNLI `validation_matched` (HF `nyu-mll/multi_nli`), premise -> quote, hypothesis -> claim,
  labels entailment/neutral/contradiction -> supports/unsure/contradicts (174 / 151 / 175).
- Model: `cross-encoder/nli-deberta-v3-xsmall`, `onnx/model.onnx` (fp32), ONNX Runtime via `ort` 2.0.0-rc.10, 1 intra-op thread.
- Machine: cloud container, Intel Xeon @ 2.80GHz, release build. Bar to act: 0.8.
- Caveat: that model was trained on MNLI train, so this set is in-distribution and flatters it.

## Results (measured)
| Judge | Acted | Accuracy when acting | Unsure rate | p50 ms | p95 ms |
|---|---|---|---|---|---|
| overlap (placeholder) | 22 / 500 | 0.636 (14/22) | 0.956 | 0.004 | 0.008 |
| DeBERTa-v3-xsmall NLI | 314 / 500 | 0.946 (297/314) | 0.372 | 36.4 | 59.5 |

## Reproduce
```
# model (git-ignored): config.json, tokenizer.json, onnx/model.onnx -> models/nli-deberta-v3-xsmall/model.onnx
curl "https://datasets-server.huggingface.co/rows?dataset=nyu-mll/multi_nli&config=default&split=validation_matched&offset=0&length=100"  # x5 offsets
cargo build --release -p ej-bench
target/release/ej-bench cases.jsonl --overlap --nli models/nli-deberta-v3-xsmall [--cmd <laya shim>]
```

## Still needed before picking a layer-2 judge
- LAYA through `--cmd` (needs a shim speaking the line protocol in `crates/ej-bench/src/command.rs`).
- A small constrained generative model.
- Labelled ledger claim/quote cases (real ones, from Claude teacher pairs) as the main set.
