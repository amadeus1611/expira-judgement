# Audit: val2 rows large+roberta calls stated against the label (2026-10-17)

All 28 rows of docs/bench/2026-10-17-nearest-bound.md (nearest-figure gate, frozen thresholds) where the cascade says
stated on val2 and the VitaminC label is not supports, read by hand against the quote. Index = line in `vitc_val2.jsonl`.

## Real errors (3)
| Row | Why |
|---|---|
| 4276 | "news conference the night before he was fired": quote says the night he was fired |
| 11186 | "kills most of Boddicker's henchmen": quote says most of the factory workers and one henchman |
| 26550 | "Recovery is an album": the quote treats Recovery as a song on an album |

## Arguable (4)
| Row | Why |
|---|---|
| 365 | "best selling Latin album of 2001": quote limits it to the United States |
| 7193 | "production budget": quote says production and advertisement budget |
| 19257 | "population of Liverpool": quote says the Liverpool city council area |
| 23511 | "developed in the United Kingdom": quote says largely in the UK and the US |

## Label errors (21)
869, 1058, 1144, 1783, 2542 (Doom 1), 3433, 3588 (4.3 million is over 4 million), 4625, 7747, 7909, 10725, 10808,
13205, 13259, 13434, 15321, 17857, 17858, 20887 and 22294 (vandalised quotes that still state the claim), 27548.

## Result
Counting arguable rows as errors: 1685/1692 (Wilson 95% lower bound 0.9915). Counting them as right: 1689/1692 (0.9948).
Correct approvals were not audited. This is the first audited set above the 0.99 bar; confirmation and val (old gate)
were 0.988 and 0.9895 pooled, so the bar is not yet met across sets.
