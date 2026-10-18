# 2026-10-17: carry a full date's year to a later yearless day in quotes (kept)

Change: `ej-evidence` `dates` reads "Month D" / "D Month" with no year as a date in the year of the last full date or
month before it, in quotes only (research note docs/research/2026-10-17-year-carried-dates.md).
Data and machine as docs/bench/2026-10-17-all-gates.md. Layer 1 and the wording gate rerun on every set; judge scores
reused (rows newly unsure have none, so they count as 0 and are not approved).

Layer-1 verdict changes (old -> new, label): no new mismatch anywhere.
- conf: mismatch -> unsure 5 supports, 1 contradicts; val: 2 supports, 1 contradicts, 2 unsure; val2: 2 supports;
  fresh: 7 supports, 2 contradicts. cal and test: none.

| set | block right / blocks before | LB | after | LB |
|---|---:|---:|---:|---:|
| test | 80/84 | 0.8839 | 80/84 | 0.8839 |
| confirmation | 446/499 | 0.8637 | 445/493 | 0.8733 |
| val | 911/1051 | 0.8449 | 908/1046 | 0.8462 |
| val2 | 996/1181 | 0.8215 | 996/1179 | 0.8230 |
| fresh | 831/937 | 0.8650 | 829/928 | 0.8718 |

Stated calls unchanged on every set (cal 157/157, test 198/199, conf 788/801, val 1663/1697, val2 1664/1692,
fresh 1305/1320). Kept: block accuracy rises on every set touched, nothing else changes; coverage falls by 22 rows of
94,159.
