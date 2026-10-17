# Layer-1 check for genus vs species claims (2026-10-17)

## Problem
Two of the five real or arguable errors the local cascade approves on the confirmation set are missed by both
large+roberta and the Haiku second check (docs/bench/2026-10-16-haiku-verify.md):
- 2914: claim "Fraxinus is tall ash tree" from a quote about "Fraxinus excelsior" (property of one species
  stated of the whole genus);
- 3598: claim "William Michael Albert Broad is known the stage name of Billy Idol" (garbled, roles swapped).
Question: can a free deterministic layer-1 check remove either class?

## Sources
- VitaminC (Schuster et al., 2021), the data these errors come from: https://arxiv.org/abs/2103.08541
- Binomial nomenclature, genus name followed by a lowercase specific epithet:
  https://en.wikipedia.org/wiki/Binomial_nomenclature

## Licence
No code or data copied. VitaminC is used as already set up in the bench (CC BY-SA 3.0 per its card).

## Candidates
1. Genus-only rule: send to unsure when a capitalised claim word appears in the quote only as the genus of a
   parenthesised binomial ("( Fraxinus excelsior )") and the epithet is absent from the claim.
2. Role-swap / garbled grammar (3598): no deterministic candidate found that separates it from correct
   paraphrases without a parser; left to judges.

## Decision
Candidate 1 prototyped (scratch Python, no model calls) and rejected: over cal (2,905), test (3,095) and
confirmation (11,393) rows it fires exactly once, on 2914, the row it was written from. It moves the raw
confirmation lower bound 0.9724 -> 0.9739, but that is not held-out evidence and the rule is fitted to n=1.
See docs/bench/2026-10-17-genus-species-check.md. Candidate 2 not attempted.
