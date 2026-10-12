# Label audit: VitaminC rows the local cascade called "stated" against the label (2026-10-12)

Scope: every row where the frozen configuration (layer 1, then DeBERTa-v3-xsmall at threshold 0.9641 with the
wording and figures gate) said "stated" and the VitaminC label was not "supports": 1 test row, 26 confirmation rows
(`tools/fetch_general.py --confirm --pages 200`, SHA-256 `7ab5814e...`, 7 rows sharing text with the frozen splits
removed, 11,393 rows). The question asked of each row is the system's question: does this quote state this claim?
A label is changed only when the quote plainly states the claim; anything arguable keeps its label. Never used to
fit anything. Auditor: Claude (Opus 5.5), not blind to the system's output; see Limits.

| Split | Row | Label | Audited | Reason |
|---|---:|---|---|---|
| test | 2769 | contradicts | supports | "taped and broadcast Monday through Friday" includes Monday |
| conf | 146 | contradicts | supports | "begins a relationship with her" |
| conf | 438 | unsure | unsure | "many of them", not all artists: system error |
| conf | 790 | contradicts | supports | "overhears Prince Phillip proposing to Aurora" |
| conf | 1350 | unsure | unsure | quote stylises "KSHМЯ", not "KSHMR": system error |
| conf | 1845 | unsure | supports | "a pop, disco and dancehall song" |
| conf | 2104 | contradicts | contradicts | "Old Hampshire", arguable: kept |
| conf | 2442 | contradicts | contradicts | "would be titled", an announcement: kept |
| conf | 2443 | contradicts | contradicts | as above: kept |
| conf | 2575 | contradicts | contradicts | "the nut" vs "the peanut": system error |
| conf | 2914 | contradicts | contradicts | genus vs one species, arguable: kept |
| conf | 3497 | unsure | supports | "Auckland Airport is the largest and busiest airport in New Zealand" |
| conf | 3598 | contradicts | contradicts | claim is ungrammatical: kept |
| conf | 4196 | unsure | supports | quote names the Culkin family children |
| conf | 4635 | contradicts | contradicts | "punk rock" vs "rock", arguable: kept |
| conf | 6608 | unsure | supports | "an American voice actor" is an actor |
| conf | 7582 | contradicts | contradicts | roles swapped (Catalonia/Barcelona): system error |
| conf | 7672 | contradicts | contradicts | "Large Town of Birmingham Symphony Orchestra": system error |
| conf | 9412 | contradicts | supports | "one of the lions" |
| conf | 9528 | contradicts | contradicts | quote says "immunity": system error |
| conf | 9848 | contradicts | contradicts | "rejected ... to be an actress": system error |
| conf | 10104 | contradicts | supports | "is an English actor" |
| conf | 10326 | contradicts | supports | "a 2010 Broadway musical" |
| conf | 10328 | contradicts | supports | "computer-animated musical fantasy-comedy film" |
| conf | 10484 | contradicts | supports | "his final adventure in The Beginning of Time" |
| conf | 11244 | contradicts | supports | "retired professional basketball player" |
| conf | 11320 | contradicts | supports | "is an American singer" |

Changed: 1 test row, 13 confirmation rows. Kept as errors: 13 confirmation rows (7 clear system errors, 6 arguable).

## Other direction
A random sample of 40 confirmation rows the system called "stated" with label "supports" (seed 12) was read the
same way: 0 of 40 needed a change, so no correction was made in the direction that would lower the score.

## Scores (frozen configuration)
| Set | Raw labels | Raw lower bound | Audited labels | Audited lower bound |
|---|---:|---:|---:|---:|
| Test | 235/236 (99.58%) | 0.9764 | 236/236 (100%) | 0.9840 |
| Confirmation | 851/877 (97.04%) | 0.9569 | 864/877 (98.52%) | 0.9748 |

## Limits
The audit reads only the disagreements, so it can only raise the score; the 40-row sample bounds, but does not
remove, that bias. The auditor is not independent of the system. Both columns are reported; the raw column is the
standard's number.
