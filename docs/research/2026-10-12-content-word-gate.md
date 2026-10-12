# Content-word gate on local "stated" calls (2026-10-12)

## Problem
The whole local cascade (layer 1, then DeBERTa-v3-xsmall) made 6 wrong "stated" calls in 220 on VitaminC test
(lower bound 0.9418). Five changed one word against the quote: "mixed" for "negative" reviews (three), "Rush" for
"Russ", "Luka Mathai" for "Matthai". VitaminC is built from exactly such one-word contrastive revisions, and a small
cross-encoder misses them.

## Sources
- Schuster et al., "Get Your Vitamin C! Robust Fact Verification with Contrastive Evidence", NAACL 2021:
  https://aclanthology.org/2021.naacl-main.52/ ; contrastive revisions differ in a word or a figure.
- McCoy, Pavlick and Linzen, "Right for the Wrong Reasons: Diagnosing Syntactic Heuristics in Natural Language
  Inference", ACL 2019: https://aclanthology.org/P19-1334/ ; NLI models lean on word overlap, so high overlap alone
  does not mean entailment, while a word the premise lacks is a real signal.

## Licence
Ideas only; no code or data copied. The stop-word list and suffix stemmer are written fresh in `ej-evidence`.

## Candidates
1. Status quo (names rule only): test 253/261 at the name-rule threshold, lower bound 0.9407.
2. Require every claim content word (function words and plain numbers aside, crude suffix stemming) to appear in a
   quote before a local model's "stated" counts; allow k missing words, k fitted on calibration.
3. A larger local model (DeBERTa-v3-large): test scoring unfinished, 10x latency.

## Decision
Candidate 2 with k = 0 (best calibration bound for k in 0, 1, 2, unlimited; then confirmed once on test). It only
removes local "stated" calls, so it cannot add a wrong pass; everything it removes escalates.
