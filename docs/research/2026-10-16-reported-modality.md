# Reported or planned statements read as fact (2026-10-16)

## Problem
Among the 13 confirmation errors of large+roberta (docs/bench/2026-10-16-roberta-second-family.md), the clearest real
engine error: "Major Lazer's fourth studio album was titled Lazerism" accepted from "Walshy Fire announced that the
album would be titled Lazerism"; a rival title was accepted from a near-identical quote. An announcement or plan is
not evidence that the event happened.

## Sources
- Saurí & Pustejovsky 2009, "FactBank: a corpus annotated with event factuality", Lang. Resources & Evaluation 43.
- Farkas et al. 2010, "The CoNLL-2010 Shared Task: Learning to Detect Hedges and their Scope in Natural Language
  Text", https://aclanthology.org/W10-3001/
- Lexical cue lists for hedges and future/reported modality from both.

## Licence
Ideas only; no data or code copied. The cue list is our own short list.

## Candidates
1. Wording gate addition: a local "stated" is refused when the quote has a reported/planned cue ("would be",
   "will be", "announced", "plans to", "expected to", "scheduled to", "proposed", "reportedly", ...) and the claim has
   none. Prototype in Python over the `ej-bench words` rows before any Rust.

## Decision
Rejected (docs/bench/2026-10-16-reported-modality.md). The error class was found on the confirmation set, so that set
cannot judge it; on calibration and test the cue removes one correct call each and no error. Revisit only with a
fresh held-out set or a model-based factuality signal.
