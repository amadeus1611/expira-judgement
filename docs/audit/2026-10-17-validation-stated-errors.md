# Audit: validation-set rows large+roberta calls stated against the label (2026-10-17)

All 36 rows of docs/bench/2026-10-17-validation-set.md where the cascade says stated and the VitaminC label is not
supports, read by hand against the quote. Index = line in `vitc_val.jsonl`.

## Real errors (9)
| Row | Why |
|---|---|
| 1220 | "kills herself": quote says knocking herself out |
| 6148 | "sold in 2016": quote says by 2016 (cumulative) |
| 7363 | "more than $102 million": quote total is $101.7 million |
| 8663 | "central government published assurances": quote says state media |
| 18426 | "four phases will last two weeks": quote says phases 1-3, at least two weeks |
| 19216 | "more than 65 ... over 54": quote says exactly 65 and 54 |
| 24585 | "Keith Urban is ... second studio album": the album is Keith Rural, by Keith Urban |
| 26555 | "Raabta is a 2012 film": a song from a 2012 film |
| 27491 | "is a competition": a movie about a competition |

## Arguable (2)
| Row | Why |
|---|---|
| 3711 | quote says "Jessica released"; surname only by context |
| 4512 | "Prime Minister of North Macedonia": quote says "Macedonian Prime Minister" |

## Label errors (25)
1603, 1694, 1822, 1879 (Equalizer 2 is a thriller film), 2498 (same date, other format), 2507, 2513, 3321, 4235,
5448, 6149 ("in 2016" implies "by 2016"), 8108, 9447, 13156 ("on" for "in"), 13479, 15037, 15624, 17171 (was/is),
17247, 20425, 20689, 23711, 26865 (quote says it; layer 1 backed), 27703, 29477 (degenerate "R" / "R" row).
