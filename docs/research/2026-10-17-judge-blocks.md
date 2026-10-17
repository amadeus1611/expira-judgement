# 2026-10-17: letting the local judges block, not only pass

## Problem
min_coverage binds: the local layer settles ~10% of claims against a 0.50 bar (docs/bench/2026-10-17-all-gates.md).
Judges are only used to call claims stated; every "not stated" call comes from layer 1, whose blocks are ~85% right.
Escalated claims do not count as local coverage, so Claude escalation cannot move this gate.

## Sources
- Geifman and El-Yaniv, "Selective Classification for Deep Neural Networks", NeurIPS 2017:
  https://arxiv.org/abs/1705.08500 (a rejection threshold chosen on held-out data to guarantee a risk bound).
- Chow, "On optimum recognition error and reject tradeoff", IEEE Trans. Information Theory 16(1), 1970:
  https://doi.org/10.1109/TIT.1970.1054406 (two-sided reject rule: act on both confident ends, defer the middle).

## Licence
Methods only; no code or data taken.

## Candidates
1. Block unsure rows whose every judge score is at or below a threshold fitted on calibration for the block bar
   (Wilson lower bound >= 0.95, at least 20 blocks).
2. As 1, but never judge-block a claim whose figures layer 1 finds backed: a sample of 20 wrong judge blocks on
   confirmation had 10 real errors, 7 of them "more than X" claims met by the quote's figure.

## Decision
Candidate 2 (docs/bench/2026-10-17-judge-blocks.md), in the bench tool first; `ej check` does not block on judge
scores yet.
