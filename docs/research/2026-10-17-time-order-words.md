# Time-order words in the wording gate

## Problem

The val2 audit (docs/audit/2026-10-17-val2-stated-errors.md) found a real error where the claim adds a time order the
quote does not state: "the night before he was fired" against "the night he was fired". `unstated_words` treats
"before", "after" and "during" as function words, so the gate ignores them.

## Sources

- Vashishtha et al., "Temporal Reasoning in Natural Language Inference", Findings of EMNLP 2020. NLI models are weak
  on event order and duration. https://aclanthology.org/2020.findings-emnlp.363/
- Thukral et al., "Probing Language Models' Understanding of Temporal Reasoning in NLI" (TRACIE line of work), and
  Zhou et al., "Temporal Reasoning on Implicit Events from Distant Supervision", NAACL 2021. Order relations are often
  paraphrased ("following", "prior to", "when ... later"). https://aclanthology.org/2021.naacl-main.107/

## Licence

Our own code only (MIT OR Apache-2.0). Data: VitaminC (CC BY-SA 3.0), fetched at run time, never stored in the repo.

## Candidates

1. Drop "before", "after" and "during" from `FUNCTION_WORDS`, so each must appear in a quote.
2. Map order paraphrases ("following", "prior to", "later", "succeeded") to "after"/"before" first. That's a word list
   that VitaminC's paraphrases would keep outgrowing.
3. Leave it to the judges or escalation.

## Decision

Measured candidate 1 (docs/bench/2026-10-17-time-order-words.md): it blocks 1 real error and loses 18 correct
approvals across the five sets, almost all paraphrases. Reverted. Candidate 2 is not worth its code for one error in
4,190 audited approvals; the class stays with the judges.
