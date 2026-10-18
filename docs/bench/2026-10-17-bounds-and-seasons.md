# 2026-10-17: "later/earlier than" bounds and "2000-01" seasons (both reverted)

Measured against the kept year-carry build (docs/bench/2026-10-17-year-carried-dates.md); same data, machine and
method. Research context: docs/research/2026-10-17-year-carried-dates.md (same wrong-block hand check).

1. "later than" / "earlier than" read as Gt / Lt bounds. Changes only on fresh: 2 wrong blocks of supported claims
   become unsure, and 2 right blocks of contradicted claims become unsure too (an unmet bound escalates by design).
   Block accuracy 829/928 -> 827/924, coverage -4 rows. No gain: reverted.
2. "YYYY-YY" whose second part is the next year's last digits read as the years YYYY to YYYY+1, not a month.
   Supported claims: 3 wrong blocks removed, 2 new wrong blocks (a quote's "2003" now rivals the claim's "2003-04"
   second year); 7 more rows of other labels change both ways. Not a clean gain: reverted.
Approvals unchanged in both. Neither is kept; the season case needs "a season is backed by either of its years" first.
