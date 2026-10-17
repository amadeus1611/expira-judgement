# 2026-10-17: a day written without its year in a quote

## Problem
Layer 1 blocks supported claims when the quote writes a later day without its year ("began on August 2 , 1777 , and
ended August 22"): the claim's "August 22 , 1777" finds only the rival date 1777-08-02 and is called a mismatch.
Seen in the hand check of fresh-set wrong blocks (docs/bench/2026-10-17-all-gates.md: rows 18405, 12045).

## Sources
- Mani and Wilson, "Robust Temporal Processing of News", ACL 2000: https://aclanthology.org/P00-1010/ (resolving
  underspecified dates against a reference time in the text).
- HeidelTime temporal tagger, which anchors underspecified expressions to a reference date found in the document:
  https://github.com/HeidelTime/heideltime
- Stanford SUTime: https://nlp.stanford.edu/software/sutime.html

## Licence
Ideas only; no code taken. HeidelTime is GPL-3.0 and SUTime is GPL; nothing from either is copied.

## Candidates
1. Carry the year of the last full date or month into a later yearless day, in quotes only.
2. Also carry bare four-digit years. Measured: wrong, it reads counts and codes ("2500 recoveries", "2019-nCoV") as years.
3. Also carry in claims. Measured: wrong, it makes a claim more precise than written and blocked 7 supported claims.

## Decision
Candidate 1 (docs/bench/2026-10-17-year-carried-dates.md).
