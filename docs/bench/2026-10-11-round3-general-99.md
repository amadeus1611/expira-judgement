# Round 3: general layer toward a proven 99% on VitaminC (2026-10-11, stopped at hand-over)

## Setup
- Data: VitaminC test split via `tools/fetch_general.py` (60 evenly spaced pages, split by `case_id`): 2,905 calibration,
  3,095 test rows. Out of distribution for every checker used. The 1,000-row sample for layer 3 and the cascade is
  `random.Random(7).sample(range(3095), 1000)` (as `tools/escalation_bench.py` draws it).
- Machine: 4-vCPU cloud container, heavily loaded throughout (latency not measured this round).
- Layer 3 through `claude -p` on the owner's subscription (no API key): checker system prompt, JSON schema, cited span
  checked against the quote in code (`tools/escalation_bench.py`).

## Results (measured; labels as given by VitaminC)
| Layer / judge | Rows | "Supports" calls right | 95% Wilson lower bound |
|---|---|---|---|
| DeBERTa-v3-xsmall, fitted at 0.95 (AUROC 0.794) | 3,095 test | 298 / 310 | 0.9336 |
| + exact-name rule | 3,095 test | 290 / 299 (at 0.96) | 0.9438 |
| + negation pre-check (rejected) | 3,095 test | 294 / 306 | 0.9327 |
| Claude Haiku 5.5 via CLI | 650 of 1,000 | 277 / 289 | 0.9288 |
| Claude Opus 5.5 via CLI | 374 of 1,000 | 185 / 194 | 0.9142 |
| Haiku and Opus both "supports" with cited spans | 374 | 164 / 171 | 0.9179 |
| Layer 1 "backed" (deterministic) | 1,000 | 13 / 13 | — |
| Overlap placeholder (removed from `ej check`) | 1,000 | 73 / 82 | — |

Cost: Haiku $0.29 for 650 calls (~$0.00044 each); Opus $6.04 for 374 (~$0.016 each), list prices reported by the CLI.

Layer 1 "mismatch" on true claims (blocking, not a hallucination risk): 145 of the 1,000 sample before the comparator and
bare-year changes, 33 after (`bounded_quantities`, `Cmp`, `year_of`); "backed" stayed 13/13.

MiniCheck-DeBERTa-v3-Large exported to ONNX (`tools/export_onnx.py --labels unsure,supports`): matches the PyTorch model
to 1e-6 and scores AUROC 0.593 on a 40-row smoke sample, the same in its reference input format. Not scored at scale.

## Findings
1. Measuring layers in isolation is wrong: the system is a cascade. Layer 1 would stop several of Claude's errors (e.g.
   "$75,000,000" against "$42,307,260"). The 99% must be measured on the whole cascade (open item).
2. Label noise binds. Of the confident "supports" calls marked wrong, about six of ten look like VitaminC label errors
   (e.g. "Barnyard ... unimaginative and unfunny" against a quote saying exactly that, labelled contradicts; "borders
   South Korea" vs "Republic of Korea"). Against noisy labels 99% may be unreachable for any system: a documented label
   audit (Northcutt et al., "Pervasive Label Errors in Test Sets", NeurIPS 2021 Datasets track) with raw and audited
   scores reported side by side is needed (open item).
3. Real model errors that remain: unstated details accepted ("almost completed his training"), and near-synonym or scope
   shifts ("two years" vs "two seasons").

## Standard v1 verdict
No pass. Binding gate: `pass_lower_bound` 0.99. Next: whole-cascade measurement, label audit, then the escalation design
(agreement, effort, prompt) re-measured on audited labels.
