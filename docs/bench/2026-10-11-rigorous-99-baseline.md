# Rigorous 99% research baseline (2026-10-11)

## Setup and scope
- Source tree: main `7ce64571c688088577d930e89976254eb3538f06`, after PR #3 merged with green CI.
- 4-CPU-quota cloud machine; Rust 1.97.0, ONNX Runtime override from the verified previous setup.
- No decision code, thresholds, frozen split or label changed. No local policy currently passes v1; no NLI inference
  was run. This is a deterministic production-path baseline and sampling-integrity study, not the requested complete
  live cascade or a new 99% result.
- First ran fmt, clippy with warnings denied, and full workspace tests: passed, 117 reported tests. The NLI smoke
  returned early without a model; it is not counted as successful inference.
- Hugging Face datasets-server is now reachable. `python3 -I tools/fetch_general.py /workspace/setup/vitaminc`
  reproduced the historical 2,905 calibration / 3,095 test counts and label totals. A second fetch, ordered by the
  same 60 offsets, was byte-identical to the first. Public data stays outside the repository.

## Data integrity measurements

| Split | Rows | Distinct case IDs | Group sizes: 1 / 2 / 3 / 4 rows |
|---|---:|---:|---|
| Calibration | 2,905 | 898 | 28 / 295 / 13 / 562 |
| Test | 3,095 | 935 | 22 / 284 / 11 / 618 |

Shared case IDs: 0. Nevertheless, there are 21 shared exact claim–quote pairs (22 test rows), or 24 shared pairs after
lowercasing and collapsing whitespace (25 test rows). The label sets for these shared pairs match across splits.
There are 71 shared normalised claims and 169 shared normalised quotes. These are counts, not label corrections;
duplicate detection did not inspect individual examples or fit a policy. Shared text alone does not quantify the
amount of performance inflation, but case-ID separation is not sufficient to claim absence of overlap.

SHA-256 of rebuilt files:

```text
vitc_cal.jsonl  bc62be6621bb300a77cf60fcd2e2387178c96f3a0452e1f7cb456a2b1ea539f0
vitc_test.jsonl cb223f926c408361de98fa04f51714b115c899e3595140505c9e9db1ff06a6f9
```

Reproduction of companion identities: fetch offsets `k*((55197-100)//59)` for k=0..59 from the existing fetch script's
API, retaining `row_idx` and `row.case_id` before conversion. Apply exactly the existing label/empty-text filters and
SHA-256(case_id) first-byte parity. Compare converted JSONL bytes with both files before joining by row order. Count
case IDs and intersections of `(claim, quote)` keys, either exact or each component `" ".join(text.lower().split())`.
No regenerated or deduplicated file replaces a frozen split.

## Production layer-1 results on raw labels

Build: `source /workspace/setup/activate.sh && cargo build --locked -p ej-cli`.
Create a claim for each test row: string index as ID, unchanged claim and evidence, source `VitaminC supplied quotation`,
date `2026-10-11`. The date is a harness fixture, not a recovered publication date. Run:

```sh
target/debug/ej check /workspace/setup/vitaminc/all-claims.json --today 2026-10-11
target/debug/ej check /workspace/setup/vitaminc/seed7-claims.json --today 2026-10-11
```

Both exit 1 as expected for failing findings. No local/escalation flags are supplied. The second input uses the unchanged
historical sorted `random.Random(7).sample(range(3095),1000)` indices. Join every finding to its original label by ID.

| Verdict | Original label | Full test | Seed-7 sample |
|---|---|---:|---:|
| backed | supports | 25 | 13 |
| mismatch | supports | 119 | 33 |
| mismatch | contradicts | 191 | 51 |
| mismatch | unsure | 101 | 30 |
| unsure | supports | 1,408 | 469 |
| unsure | contradicts | 966 | 323 |
| unsure | unsure | 285 | 81 |

| Metric | Full test | Seed-7 sample |
|---|---:|---:|
| Correct stated / stated | 25 / 25 | 13 / 13 |
| Nominal row-level v1 pass Wilson bound | 0.866804 | 0.771898 |
| Correct not-stated / blocked, binary v1 interpretation | 292 / 411 | 81 / 114 |
| Nominal row-level v1 block Wilson bound | 0.664824 | 0.621492 |
| Deterministic settled share | 0.140872 | 0.127000 |

The 25 backed rows occupy 19 distinct case IDs. Row-level bounds are compatibility statistics, not an established
confidence guarantee for independent cases. With no eligible model, pre-escalation coverage is only the deterministic
settled share, below 0.50. ECE, fresh stress and quiet local p95 were not measured; v1 is not passed. Escalation
coverage cannot be substituted for coverage settled before escalation.

With these fixed layer-1 decisions, even perfect later routing cannot make the raw full-test block gate pass. There
are only 1,543 non-support labels available to count as correct blocks, while 119 false blocks are already final.
The best possible binary block statistic is therefore 1,543 / (1,543+119), Wilson lower bound 0.914991 < 0.95.
This upper bound concerns these frozen labels and routing, not every future distribution. Counting unsure labels as
correct not-stated blocks is deliberately the favourable v1 interpretation; three-way contradiction correctness is
also visible above. Layer-1 reliability and sampling integrity must be resolved alongside the stated gate.

## Subscription prerequisite and cost
One synthetic exact-quote probe through built-in codex-cli 0.159.2 used the existing managed authentication and the
previously approved startup-write route, 45-second timeout, read-only model sandbox, ephemeral session and JSON schema.
It exited 1 promptly: endpoint HTTP 401 and `turn.failed` reported that authentication could not be refreshed after a
logout/account change. No answer, completed-turn usage or dollar report. Cost/usage unknown; no held-out rows sent.
No repeated paid batch, credential copy or new key request. Endpoint access improved from the earlier CONNECT 403;
managed authentication now needs restoration outside the benchmark. Claude calls this session: 0.

## Directed logical challenge, separate from statistical validation
After the literature review, authored three fixtures around the same claim, `The gate is locked.` No dataset example
was used. All evidence uses a fresh authored challenge source and the harness date. Run the normal `ej check` path
with no models:

| Supplied quote | Logical expectation | Observed verdict |
|---|---|---|
| `The gate is locked.` | supports | backed |
| `The statement 'The gate is locked.' is false. The gate is unlocked.` | no stated pass | unsure |
| `If the alarm is armed, the gate is locked. Whether the alarm is armed is unknown.` | unsure | backed |

The conditional quote entails only an implication, not its consequent: without the antecedent, the claim is not
established. The third fixture is one false stated decision with confidence 1.0. The production process exits 1 due
to the refutation fixture's unsure finding; the false pass is visible in the row-level verdict. A whole-process exit
alone would miss this error. This is a purposive engineering counterexample, not a prevalence estimate, a statistical
held-out result or a new automated CI test. No code or policy was fitted to it. It is now disclosed and must not be
presented as an unseen stress example after a fix. Future development needs independent examples and fresh stress.

## Validation and outcome
The sample-size calculation searched n up to 19,999 for each specified error count; exact binomial tails used Python
`Decimal` precision 60 and the recurrence `t0=.99**n; tj=t(j-1)*(n-j+1)/j*.01/.99`. Every minimum and its predecessor
were checked, alongside the unchanged Wilson formula. Results appear in `docs/validation/general-99.md`.
An independent reference then used rational score-test inversion for Wilson and direct integer binomial sums,
agreeing on all five minima and their predecessors for each method:

```python
from fractions import Fraction as F
from math import comb

def score(e, n):
    margin = F(n, 100) - e
    return margin >= 0 and margin**2 >= F(196, 100)**2 * n * F(99, 100) * F(1, 100)

def exact(e, n):
    return F(sum(comb(n, j) * 99**(n-j) for j in range(e+1)), 100**n) <= F(1, 40)

for e, w, c in [(0,381,368), (1,563,555), (2,726,720), (5,1166,1164), (10,1836,1835)]:
    assert score(e, w) and not score(e, w-1)
    assert exact(e, c) and not exact(e, c-1)
```

Updated the reproducibility habit to distinguish grouping from measured overlap; retired the unsupported assumption
that disjoint case IDs prove leak-free data. This is a correction to evaluation practice, not a model improvement.
Saved additional full-paper research hosts and current startup findings in the environment draft: arxiv.org,
export.arxiv.org, aclanthology.org, openreview.net and proceedings.neurips.cc. Persistence is confirmed; publication and
active access to those hosts are not claimed. Existing domains and package-manager presets were retained.

Final local software checks: fmt and clippy with warnings denied passed; full workspace tests reported 117 passing,
including the early-return NLI smoke. cargo-deny passed advisories/bans/licences/sources with existing duplicate-version
warnings. A fresh full six-crate mutation run exited 0: 856 generated, 794 caught, 62 unviable, zero missed and zero
timeouts. Decision source and tests are unchanged; these checks do not refute the semantic counterexample above.

No accuracy improvement is claimed or adopted. Research defines an untouched, grouped, fixed-policy confirmation
procedure with multiplicity control and a separate blind label audit. Frozen tests remain diagnostics, never fitting
data. PR CI must be green before merge.

Artifacts outside the checkout: `/workspace/setup/vitaminc/` contains the frozen rebuilds, raw pages, companion
identities, integrity summary, complete findings, layer-1 summary and proof calculations. Sources read are under
`/workspace/setup/rigorous-sources/`; synthetic CLI failure is under `/workspace/setup/codex-rigour-smoke/`.
