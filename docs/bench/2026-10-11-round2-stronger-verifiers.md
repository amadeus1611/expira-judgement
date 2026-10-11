# Round 2: stronger open verifiers (2026-10-11)

Same protocol, splits, stress set and fitting as round 1 (`2026-10-11-round1-laya-vs-deberta-calibrated.md`).

## Candidates
- `cross-encoder/nli-deberta-v3-base` fp32 ONNX (Apache-2.0)
- `MoritzLaurer/DeBERTa-v3-large-mnli-fever-anli-ling-wanli` (MIT): the repo's 8-bit `model_quantized.onnx`, and fp32 `model.onnx`

## Results on the test split (measured)
| Judge | AUROC (all) | AUROC FEVER / MNLI | Best "stated" calls (target 0.90) | Stress wrong passes at that policy |
|---|---|---|---|---|
| DeBERTa-v3-xsmall (round 1) | 0.956 | 0.927 / 0.983 | 198/210, pass bound 0.903 | 2 |
| DeBERTa-v3-base | 0.958 | 0.931 / 0.984 | none (only "not stated") | 0 |
| DeBERTa-v3-large, 8-bit | 0.916 | 0.918 / 0.935 | none | 0 |
| DeBERTa-v3-large, fp32 | **0.973** | — / 0.990 | 244/256, pass bound 0.920 | 5 |
| Mean of large fp32 + xsmall | **0.975** | — | target 0.95: 123/126, pass bound 0.932 | 2 |

Raw p >= 0.6 on stress (a "stated" call under the old fixed bar): large fp32 passes 12/200 negated and 2/30 changed-figure claims.

Latency (median per claim, this 4-vCPU container, other jobs running): base 124 ms, large 8-bit 152-200 ms,
large fp32 358 ms median and 1,240 ms p95 under heavy load (mutation testing ran alongside). Re-measure on a quiet machine.

## Findings
- The 8-bit ONNX build of the large model is damaged (MNLI AUROC 0.935 vs 0.990 for fp32): quantisation, not the model.
  Do not use that file; quantise ourselves with calibration data if speed is needed, and re-measure.
- Base is no better than xsmall for ~2.6x the time: rejected.
- Large fp32 is the best single verifier and the large + xsmall mean the best overall, but nothing reaches the 0.99 bound,
  and large fp32 likely fails the 250 ms gate on this CPU.

## Standard v1 verdict
No pass. Binding gate unchanged: `pass_lower_bound`. Next levers, in order: more held-out labelled ledger cases (the bound
needs ~460+ clean "stated" calls), a task-specific checker (MiniCheck, needs ONNX export), a properly calibrated int8 large
model for latency, and layer-1 negation handling in front of the judges (stress wrong passes are negations).
