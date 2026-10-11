//! The EXPIRA Judgement standard: the bar the whole decision block (layer 1 + local judge, with Claude behind it)
//! must clear on held-out data before a configuration is adopted. It lives in code, not prose, so it is checked
//! every run and cannot drift. Changing a bar is a deliberate, reviewed commit.
//!
//! Why these gates:
//! - A wrong "stated" call lets an unbacked claim through: that is the hallucination this engine exists to stop, so
//!   pass decisions carry the hardest bar, as a 95% lower confidence bound (not a raw rate) on held-out cases.
//! - A wrong "not stated" call costs an escalation or a needless block, so its bar is lower.
//! - Figure swaps and negation are where small models are known to fail; the stress set allows zero wrong passes.
//! - Calibration error bounds how far a reported probability can be trusted when thresholds move.
//! - Coverage stops the trivial pass (never acting): the local layer must settle a real share before Claude is paid.
//! - Latency keeps the local layer local.

use crate::calib::{wilson_lower, Selective, Z95};
use serde::Serialize;

/// The bars a configuration must clear on held-out data (see the module docs for why).
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct Standard {
    /// Minimum 95% Wilson lower bound on the accuracy of "stated" calls.
    pub pass_lower_bound: f64,
    /// Minimum 95% Wilson lower bound on the accuracy of "not stated" calls.
    pub block_lower_bound: f64,
    /// Maximum wrong "stated" calls allowed on the stress set.
    pub stress_wrong_passes: usize,
    /// Maximum expected calibration error on the test split.
    pub max_ece: f64,
    /// Minimum share of test cases the local layer must act on.
    pub min_coverage: f64,
    /// Maximum 95th-percentile latency per ruling, in milliseconds.
    pub max_p95_ms: f64,
}

/// Version 1, set 2026-10-11.
pub const V1: Standard = Standard {
    pass_lower_bound: 0.99,
    block_lower_bound: 0.95,
    stress_wrong_passes: 0,
    max_ece: 0.05,
    min_coverage: 0.50,
    max_p95_ms: 250.0,
};

/// One gate of the standard: the measured value, its bar and whether it passed.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Gate {
    /// Gate name (the [`Standard`] field it checks).
    pub name: &'static str,
    /// The measured value (NaN when not measured).
    pub value: f64,
    /// The bar from the [`Standard`].
    pub bar: f64,
    /// Whether the value clears the bar.
    pub passed: bool,
}

/// The result of checking one candidate against a [`Standard`].
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Verdict {
    /// Candidate name.
    pub judge: String,
    /// True only if every gate passed.
    pub passed: bool,
    /// Every gate with its value and bar.
    pub gates: Vec<Gate>,
}

/// Measurements for one candidate on held-out data. `stress` and `p95_ms` are None when not measured;
/// an unmeasured gate fails, because the standard cannot be met by not looking.
pub struct Evidence<'a> {
    /// Selective results on the test split.
    pub test: &'a Selective,
    /// Expected calibration error on the test split.
    pub test_ece: f64,
    /// Selective results on the stress set, if measured.
    pub stress: Option<&'a Selective>,
    /// 95th-percentile latency in milliseconds, if measured.
    pub p95_ms: Option<f64>,
}

/// Checks one candidate's measurements against `std`; unmeasured gates fail.
#[must_use]
pub fn check(judge: &str, std: &Standard, ev: &Evidence) -> Verdict {
    let t = ev.test;
    let mut gates = vec![
        Gate {
            name: "pass_lower_bound",
            value: wilson_lower(t.yes_right, t.yes_acted, Z95),
            bar: std.pass_lower_bound,
            passed: false,
        },
        Gate {
            name: "block_lower_bound",
            value: wilson_lower(t.no_right, t.no_acted, Z95),
            bar: std.block_lower_bound,
            passed: false,
        },
        Gate {
            name: "max_ece",
            value: ev.test_ece,
            bar: std.max_ece,
            passed: ev.test_ece <= std.max_ece,
        },
        Gate {
            name: "min_coverage",
            value: t.coverage,
            bar: std.min_coverage,
            passed: t.coverage >= std.min_coverage,
        },
        Gate {
            name: "stress_wrong_passes",
            value: ev.stress.map_or(f64::NAN, |s| s.wrong_yes as f64),
            bar: std.stress_wrong_passes as f64,
            passed: ev
                .stress
                .is_some_and(|s| s.wrong_yes <= std.stress_wrong_passes),
        },
        Gate {
            name: "max_p95_ms",
            value: ev.p95_ms.unwrap_or(f64::NAN),
            bar: std.max_p95_ms,
            passed: ev.p95_ms.is_some_and(|m| m <= std.max_p95_ms),
        },
    ];
    gates[0].passed = gates[0].value >= std.pass_lower_bound;
    // A judge that never says "not stated" escalates those cases instead; that is allowed, it only costs coverage.
    gates[1].passed = t.no_acted == 0 || gates[1].value >= std.block_lower_bound;
    Verdict {
        judge: judge.into(),
        passed: gates.iter().all(|g| g.passed),
        gates,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sel(ya: usize, yr: usize, na: usize, nr: usize, cases: usize) -> Selective {
        Selective {
            cases,
            coverage: (ya + na) as f64 / cases as f64,
            acted: ya + na,
            right: yr + nr,
            accuracy_when_acting: None,
            yes_acted: ya,
            yes_right: yr,
            no_acted: na,
            no_right: nr,
            wrong_yes: ya - yr,
            wrong_no: na - nr,
        }
    }

    #[test]
    fn small_perfect_sample_does_not_pass() {
        let t = sel(100, 100, 100, 100, 300); // 100/100 right only proves ~96% at 95% confidence
        let v = check(
            "j",
            &V1,
            &Evidence {
                test: &t,
                test_ece: 0.01,
                stress: Some(&sel(0, 0, 10, 10, 10)),
                p95_ms: Some(40.0),
            },
        );
        assert!(!v.passed && !v.gates[0].passed);
    }

    #[test]
    fn large_clean_sample_with_everything_measured_passes() {
        let t = sel(500, 500, 300, 299, 1000);
        let v = check(
            "j",
            &V1,
            &Evidence {
                test: &t,
                test_ece: 0.03,
                stress: Some(&sel(0, 0, 50, 50, 60)),
                p95_ms: Some(40.0),
            },
        );
        assert!(v.passed, "{v:?}");
    }

    #[test]
    fn block_gate_passes_when_never_blocking_and_fails_on_a_weak_bound() {
        let ev = |t| Evidence {
            test: t,
            test_ece: 0.03,
            stress: None,
            p95_ms: None,
        };
        let never_blocks = sel(500, 500, 0, 0, 1000);
        assert!(check("j", &V1, &ev(&never_blocks)).gates[1].passed);
        let weak = sel(500, 500, 10, 5, 1000);
        assert!(!check("j", &V1, &ev(&weak)).gates[1].passed);
    }

    #[test]
    fn unmeasured_gates_fail() {
        let t = sel(500, 500, 300, 299, 1000);
        let v = check(
            "j",
            &V1,
            &Evidence {
                test: &t,
                test_ece: 0.03,
                stress: None,
                p95_ms: None,
            },
        );
        assert!(!v.passed);
    }
}
