# Layer 1: an absent figure is not a contradicted one (2026-10-12)

## Problem
Layer 1 called a claim a "mismatch" whenever one of its figures was missing from every quote. On the VitaminC test
split (3,095 rows) that blocked 119 claims labelled "supports": numbers inside names ("Fallout 76", "Gears of War 3",
"127 Hours") when the quote says "the game" or "the film", clock times, scores written as "50/100", days without a year
("Before September 3"), decades ("the 1980s") and approximate amounts ("nearly $ 43 million").

## Sources
- Thorne et al., "FEVER: a Large-scale Dataset for Fact Extraction and VERification", NAACL 2018:
  https://aclanthology.org/N18-1074/ ; three verdicts, with "not enough info" kept apart from "refuted".
- Schuster et al., "Get Your Vitamin C! Robust Fact Verification with Contrastive Evidence", NAACL 2021:
  https://aclanthology.org/2021.naacl-main.52/ ; contrastive pairs where one figure or date changes the verdict.

## Licence
No code or data is copied; only the distinction between missing and contradicting evidence is used. VitaminC data
stays outside the repository (CC BY-SA 3.0, read from its dataset card when fetched by `tools/fetch_general.py`).

## Candidates
1. Keep "missing means mismatch" (status quo): 119 false blocks of 411 blocks.
2. Mismatch only when the quote states a rival figure of the same kind (same unit, currency or date kind, years
   against years or dates, or the same number in another unit); otherwise escalate.
3. Also parse days without a year, decades and approximate wording so they are not read as exact figures.

## Decision
Candidate 2, then 3 as separate measured commits (docs/bench/2026-10-12-layer1-false-blocks.md). Escalating never
creates a pass: an escalated claim still needs a proven judge or Claude to be called "stated".
