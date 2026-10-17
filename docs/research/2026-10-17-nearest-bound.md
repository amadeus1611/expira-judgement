# Bounds are about the nearest figure

## Problem

The validation-set audit (docs/audit/2026-10-17-validation-stated-errors.md) found real "stated" errors on comparative
figures that the wording gate let through to the judges:

- "grossed more than $ 102 million" against a total of $ 101.7 million. A $ 208 million budget in the same quote met the bound.
- "won more than 65 ... over 54" against exactly 65 and 54. A year (2018) met "more than 65".

`figures_backed` counted a bound as met when any comparable figure in the quote satisfied it. NLI judges are known to be
weak at this kind of quantity comparison, so the gate must not hand them claims like these as if the figures were settled.

## Sources

- Naik et al., "Stress Test Evaluation for Natural Language Inference", COLING 2018. Numerical reasoning is one of the
  stress classes on which NLI models fail. https://aclanthology.org/C18-1198/
- Ravichander et al., "EQUATE: A Benchmark Evaluation Framework for Quantitative Reasoning in Natural Language
  Inference", CoNLL 2019. NLI models score near chance on comparisons between quantities. https://arxiv.org/abs/1901.03735
- V et al., "QuanTemp: A real-world open-domain benchmark for fact-checking numerical claims", SIGIR 2024. Comparative
  claims are a named class that is hard for verifiers. https://arxiv.org/abs/2403.17169

## Licence

The rule is our own code (MIT OR Apache-2.0). No model or dataset is added. The data is VitaminC (CC BY-SA 3.0),
fetched at run time and never stored in the repo.

## Candidates

1. Nearest figure. A bound is met only when the comparable quote figure nearest in value to it satisfies it. The nearest
   figure is the most likely referent, and an unrelated larger figure (a budget, a year) no longer counts.
2. Same noun. Match the figure by the noun that follows it ("competitions", "worldwide gross"). This is more precise,
   but it needs parsing that VitaminC's wording breaks often.
3. Treat any strict bound whose exact figure appears in the quote as a mismatch in layer 1. Rejected: "more than 50"
   can follow from a quote that says "more than 50", so this would create false contradictions.

## Decision

Measure candidate 1 in the gate only (not in layer 1's verdicts). The gate can only block approvals and never needs new
judge scores. The rule was found by reading the first validation set, so it is judged on a third, unread slice of
VitaminC validation pages (`fetch_general.py --validation2`), with the frozen thresholds.
