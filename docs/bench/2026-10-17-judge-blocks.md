# 2026-10-17: judge blocks in the local cascade (kept in the bench tool)

Change: `tools/local_cascade.py --block` (research note docs/research/2026-10-17-judge-blocks.md). Build: layer 1
with the year carry (docs/bench/2026-10-17-year-carried-dates.md). Data and machine as docs/bench/2026-10-17-all-gates.md.
Only cal, test and confirmation have judge scores on every row, so only they are measured here.

    python3 -I tools/local_cascade.py DATA large+roberta --gate --confirm --block

Block threshold fitted on calibration: every judge score <= 0.02065 (cal judge blocks 464/478 right).
Stated calls unchanged (test 198/199, confirmation 788/801).

| set | blocks before | block LB | coverage | blocks after | block LB | coverage |
|---|---:|---:|---:|---:|---:|---:|
| cal (fit) | 99/107 | 0.8689 | 0.0909 | 563/585 | 0.9437 | 0.2554 |
| test | 80/84 | 0.8839 | 0.0914 | 602/633 | 0.9313 | 0.2688 |
| confirmation | 445/493 | 0.8733 | 0.1136 | 2414/2538 | 0.9421 | 0.2931 |

Without the figure exclusion (candidate 1): test 643/682 (LB 0.9228, coverage 0.2847), confirmation 2501/2664
(LB 0.9291, coverage 0.3041).

Hand check of 20 wrong judge blocks on confirmation (candidate 1, seed 20261017): 10 label errors (the quote does
not state the claim), 10 real; 7 of the real ones were "more than / less than X" claims the quote's figure meets, which
the exclusion now protects.

Verdict: both gates improve and nothing else changes; kept. Still failing: block_lower_bound (0.93-0.94 raw vs 0.95;
about half the sampled errors are label noise) and min_coverage (0.27-0.29 vs 0.50). On this data the two bars
cannot both be met by moving the threshold: blocking the lowest 50% of unsure rows reaches ~0.57 coverage at ~80%
block accuracy.
