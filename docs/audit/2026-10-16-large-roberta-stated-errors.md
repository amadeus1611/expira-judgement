# Label audit: rows large+roberta called "stated" against the label (2026-10-16)

Same method and auditor as docs/audit/2026-10-12-vitaminc-stated-errors.md: the question asked of each row is
whether the quote states the claim; a label changes only when the quote plainly states it; anything arguable keeps
its label. Never used to fit anything. Auditor: Claude, not blind to the system's output.

Configuration: layer 1, then large+roberta with the wording gate, thresholds fitted on calibration
(0.000207, 0.9733), docs/bench/2026-10-16-roberta-second-family.md. The large threshold is near zero, so in
practice this is RoBERTa alone behind the gate.

## Disagreements
| Split | Row | Label | Audited | Reason |
|---|---:|---|---|---|
| test | 2000 | contradicts | contradicts | quote says the Brat Pack is an award, not a group: system error |
| conf | 146 | contradicts | supports | as 2026-10-12 audit |
| conf | 438 | unsure | unsure | as 2026-10-12 audit ("many of them"): system error |
| conf | 790 | contradicts | supports | as 2026-10-12 audit |
| conf | 2442 | contradicts | contradicts | as 2026-10-12 audit (announcement): kept |
| conf | 2443 | contradicts | contradicts | as above: kept |
| conf | 2914 | contradicts | contradicts | as 2026-10-12 audit (genus vs species): kept |
| conf | 3598 | contradicts | contradicts | as 2026-10-12 audit (ungrammatical claim): kept |
| conf | 4196 | unsure | supports | as 2026-10-12 audit |
| conf | 6608 | unsure | supports | as 2026-10-12 audit |
| conf | 10104 | contradicts | supports | as 2026-10-12 audit |
| conf | 10326 | contradicts | supports | as 2026-10-12 audit |
| conf | 10484 | contradicts | supports | as 2026-10-12 audit |
| conf | 11244 | contradicts | supports | as 2026-10-12 audit |

All 13 confirmation rows were already decided in the 2026-10-12 audit; those decisions are reused unchanged.
Changed: 8 confirmation rows. Kept as errors: 1 test row and 5 confirmation rows (1 clear system error, 4 arguable).

## Other direction
A random sample of 40 confirmation rows called "stated" with label "supports" (seed 16) was read the same way:
0 of 40 needed a change (closest: row 5252, "by 6:00 p.m." backs "before 7 p.m.").

## Scores
| Set | Raw labels | Raw lower bound | Audited labels | Audited lower bound |
|---|---:|---:|---:|---:|
| Test | 198/199 | 0.9721 | 198/199 | 0.9721 |
| Confirmation | 786/799 | 0.9724 | 794/799 | 0.9854 |

## Conclusion
Even with audited labels the lower bound (0.9854) stays below the 0.99 bar: the 5 remaining errors are real or
arguable, and at about 800 calls the bar needs at most 2 errors. Cleaning labels alone cannot pass the standard;
the raw column stays the standard's number. Next: escalate the gated-unsure middle and the arguable classes
(announcements, genus/species, ungrammatical claims) to Haiku, with owner approval for the cost.

## Limits
As in the 2026-10-12 audit: reading only disagreements can only raise the score; the 40-row sample bounds that
bias. The auditor is not independent of the system.

## Second sample (2026-10-17)
A further 100 confirmation rows called "stated" with label "supports" (seed 17, drawn after removing the seed-16
40) were read the same way. 0 of 100 plainly wrong. 1 arguable: row 9347 ("Musala is a place for prayer"; the
quote says the name means "place for prayer", and Musala is a peak). Closest calls kept: 1032 ("he" resolved to
Billy Duffy from context), 3088 (cover recorded 1986, single release year not stated). Over 140 sampled
agreements: 0 plain, 1 arguable. Audited precision on approvals stays about 794/799; the remaining errors are
the system's, not the labels'.
