# General layer to a proven 99% (2026-10-11)

## Problem
Owner, 2026-10-11: before any domain work (legal, finance), the general web-text layer must reach a proven 99%: the
standard v1 `pass_lower_bound` (95% Wilson lower bound on held-out "stated" calls >= 0.99). With zero errors that needs
381 held-out "stated" calls (380 error-free calls only prove 0.98999); every error raises the count. Rounds 1-2 had
~330 held-out "supports" cases in total and a best pass bound of 0.932, so both data volume and checker quality bind.

## Sources
- VitaminC (Schuster, Fisch, Barzilay, NAACL 2021): contrastive claim/evidence pairs from Wikipedia revisions, built so a
  small edit (often a number or date) flips the verdict: https://huggingface.co/datasets/tals/vitaminc
- Paper: https://aclanthology.org/2021.naacl-main.52/
- WANLI (CC BY 4.0) and ANLI (CC BY-NC 4.0) cards: https://huggingface.co/datasets/alisawuffles/WANLI , https://huggingface.co/datasets/facebook/anli
- LLM-AggreFact (gated, CC BY-ND 4.0): https://huggingface.co/datasets/lytang/LLM-AggreFact
- Wilson score interval: https://en.wikipedia.org/wiki/Binomial_proportion_confidence_interval#Wilson_score_interval

## Licence
- VitaminC: CC BY-SA 3.0 (card). Fetched at run time by `tools/fetch_general.py`; not stored in the repo.
- WANLI: CC BY 4.0, but the large checker (MoritzLaurer ...-ling-wanli) was trained on it, so it cannot test that model.
- ANLI: non-commercial; skipped because EXPIRA may be used commercially.
- LLM-AggreFact: gated and no-derivatives; skipped.

## Candidates
1. Data: VitaminC test, 60 evenly spaced pages (6,000 rows), split by `case_id` hash so contrastive pairs never straddle
   calibration and test (2,905 cal / 3,095 test; 1,552 test "supports"). Out of distribution for every checker we use.
2. Checkers: DeBERTa-v3-xsmall (baseline), DeBERTa-v3-large fp32 (best so far), their calibrated mean; MiniCheck next.
3. A layer-1 polarity guard in front of the checkers (negation is every stress wrong pass so far).
4. Stricter fitting for many candidate thresholds (Learn-then-Test style) once a configuration passes.

## Decision
Data (1) adopted: reproducible fetch verified (two runs byte-identical). Checker results pending the VitaminC scoring run;
logged in `docs/bench/` with the verdict per gate.
