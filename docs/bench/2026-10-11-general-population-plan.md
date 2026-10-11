# General population plan: sample-size calculation and baseline checks

Research: `docs/research/2026-10-11-general-population-plan.md`. Plan: `docs/validation/general-99-roadmap.md`.
This change specifies future experiments. It introduces no judge, prediction dataset, label correction or measured
semantic improvement. Standard v1 and frozen data are unchanged; the current general-layer verdict remains failing.

## Before / after
Before: the validation protocol states independence and untouched confirmation requirements but does not spell out
the source-population preparation, staged architecture work, independent labelling prerequisites or power planning.
After: the roadmap assigns each stage its output and acceptance evidence, distinguishes a licensed reference mixture
from actual deployment traffic, and requires power/budget registration before confirmation. This is a documentation
change, not a claim that a runner or sampling/label workflow exists.

## Fixed-count power calculation
For each hypothetical independent stated count n, find the largest false-pass count e for which **both** existing
Wilson(z=1.96) >= 0.99 and the exact boundary-error binomial tail <= 0.025 hold. Calculate the probability of observing
at most e errors at hypothetical true accuracies 0.999 and 0.998. There is one frozen candidate, no optional stopping.

| Fixed independent stated count | Maximum false passes | Pass probability if accuracy 99.9% | Pass probability if accuracy 99.8% |
|---:|---:|---:|---:|
| 381 | 0 | 68.30% | 46.64% |
| 1,000 | 3 | 98.11% | 85.73% |
| 2,000 | 11 | 99.99987% | 99.90982% |
| 3,000 | 19 | 99.999999992% | 99.99950% |

These are mathematical planning scenarios, not measured accuracy or a licence to stop at an accepted-count target.
Total input count depends on a separately estimated acceptance rate; fixed total-input planning must allow for its
random accepted count. Actual subscription cost remains unmeasured. The plan does not start a large paid run.

Independent checks: the Wilson expression was compared with its rational score-test inversion at p0=99/100:

`s - p0*n >= 0` and `(s - p0*n)^2 >= (196/100)^2 * p0 * (1-p0) * n`.

For every table row the displayed e passes and e+1 fails. Binomial recurrence calculations were repeated at
70-decimal precision; all probabilities agree with binary64 results within 1e-12. At boundary accuracy 99%, the
respective false-certification tails are 0.0217274691354261, 0.0100726547720143, 0.0209124314969863 and
0.0213832087910859, each <= 0.025. Scratch: `/workspace/setup/population-plan-power.json`.

Reproduce independently using the rational boundary above and this decimal tail, with `e` from the table:
```python
from decimal import Decimal, localcontext

def tail(n, e, error_probability):
    with localcontext() as context:
        context.prec = 70
        p = Decimal(error_probability)
        term = (1 - p) ** n
        total = term
        for k in range(e):
            term *= Decimal(n-k) / Decimal(k+1) * p / (1-p)
            total += term
        return total

print(tail(381, 0, "0.001"))
print(tail(2000, 11, "0.002"))
```

## Engineering verification and limits
Cloud Linux, Rust 1.97.0, Python 3.12.14, 4-CPU cgroup quota and 32 GiB memory. Baseline fmt, clippy and locked
workspace tests passed before editing; the Python suite passed 22 tests. ONNX smoke still returns early without
`EJ_NLI_DIR`: no actual model inference is claimed. No new semantic or subscription calls were made in this session.
No subscription cost was incurred by a new call; historical failed probes retain unknown usage/cost.

Final fmt/clippy/locked workspace tests passed: 117 reported Rust passes. Python: 22 passing tests, all 15 targeted
faults caught, zero missed/unviable/timeouts. cargo-deny passed with existing duplicate warnings. Full Rust mutations
(cargo-mutants 27.1.0): 856 generated, 794 caught, 62 unviable, zero missed, zero timeouts, exit 0; start
2026-10-11T20:44:51.433393449Z, end 2026-10-11T20:56:37.600382492Z. Scratch outcomes:
`/workspace/setup/population-plan-mutants/mutants.out/outcomes.json`.

Passing engineering checks do not
establish accuracy, source representativeness, independent labels, new hard-challenge success or a v1 pass.
