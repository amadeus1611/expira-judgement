# Reproduction from scratch and two-judge agreement (2026-10-16)

## Setup
- Inputs rebuilt from public sources with `tools/rebuild_inputs.sh /home/user/data xsmall large` (frozen settings of
  docs/bench/2026-10-12-content-word-gate.md). Cal/test SHA-256 match the frozen `bc62be66…` and `cb223f92…`.
- Confirmation set: `fetch_general.py --confirm --pages 200` now drops exact (claim, quote) pairs present in cal/test:
  11,393 rows, SHA-256 `2cbf8712…5fa24`. Correction: the hash `7ab5814e…` in
  docs/audit/2026-10-12-vitaminc-stated-errors.md is the 11,400-row file before that dedupe.
- Machine: 4-vCPU cloud container, Intel Xeon @ 2.10GHz. No model calls; no subscription usage.
- Command: `python3 -I tools/local_cascade.py /home/user/data JUDGE --gate --confirm`.

## Results (thresholds fitted on calibration only)
| Judge | Threshold | Calibration | Test | Test lower bound | Confirmation | Confirmation lower bound |
|---|---|---:|---:|---:|---:|---:|
| xsmall | 0.9641 | 161/162 | 235/236 | 0.9764 | 851/877 | 0.9569 |
| large | 0.9866 | 143/144 | 206/208 | 0.9656 | 748/763 | **0.9678** |
| large+xsmall (both pass) | 0.8203, 0.9644 | 158/158 | 230/231 | 0.9759 | 835/857 | 0.9614 |

xsmall and large reproduce the 2026-10-13 numbers exactly.

## Decision
Agreement mode (`JUDGE+JUDGE2`, joint threshold fit over 200 calibration quantiles per judge) is **rejected**: its
perfect calibration score does not carry over; the confirmation lower bound falls 0.9678 -> 0.9614. The tool mode
stays for later pairs (a different model family). Best local setup remains large alone, lower bound 0.9678.

## Standard v1 verdict
No pass. Binding gate: `pass_lower_bound` (0.99). Next: a judge from a different family, then paid escalation of the
unsure middle (owner approval needed).

## Other
- CI: cargo-deny advisories on rustls fixed by `cargo update -p rustls` (0.23.45 -> 0.23.46).
