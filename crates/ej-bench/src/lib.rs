//! Bake-off harness: run any `Judge` over labelled (claim, quote, label) cases and score it the way layer 2 is used.
//! A judge "acts" when its ruling is not Unsure and clears the bar; otherwise it escalates.
//! Scores: accuracy when acting (the number that matters: wrong acts let errors through), unsure rate (escalation cost),
//! and wall-clock latency per case on this machine.

pub mod calib;
pub mod command;
pub mod standard;

use ej_core::{Claim, Evidence};
use ej_judge::{Bars, Judge, Ruling};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// One labelled case. `label` is "supports", "contradicts" or "unsure" (neither).
/// A teacher-pair file from `ej check --claude` loads as cases too (its `ruling` is the label), so Claude's rulings
/// become the calibration set for the local judge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Case {
    /// The claim to check.
    pub claim: String,
    /// The quote offered as evidence.
    pub quote: String,
    /// The expected ruling: "supports", "contradicts" or "unsure".
    #[serde(alias = "ruling")]
    pub label: String,
}

/// Index of the `p` quantile (0..=1) in a sorted slice of length `len` (> 0), nearest rank.
#[allow(clippy::cast_sign_loss)] // p is clamped to [0, 1], so the rounded product is never negative
#[must_use]
pub fn percentile_index(len: usize, p: f64) -> usize {
    ((len - 1) as f64 * p.clamp(0.0, 1.0)).round() as usize
}

/// Parses JSON-lines cases (blank lines skipped).
///
/// # Errors
/// A line that is not a valid case.
pub fn load_cases(text: &str) -> Result<Vec<Case>, serde_json::Error> {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(serde_json::from_str)
        .collect()
}

/// How one judge did on a case set (see [`run`]).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Report {
    /// Judge name as given to [`run`].
    pub judge: String,
    /// Number of cases run.
    pub cases: usize,
    /// Cases where the judge acted (not Unsure, confidence at or above the bar).
    pub acted: usize,
    /// Acted cases whose ruling matched the label.
    pub correct_when_acting: usize,
    /// `correct_when_acting / acted`; `None` when it never acted.
    pub accuracy_when_acting: Option<f64>,
    /// Share of cases it did not act on (escalated).
    pub unsure_rate: f64,
    /// Median wall-clock time per ruling, in milliseconds.
    pub latency_ms_p50: f64,
    /// 95th-percentile wall-clock time per ruling, in milliseconds.
    pub latency_ms_p95: f64,
}

fn label(r: Ruling) -> &'static str {
    match r {
        Ruling::Supports => "supports",
        Ruling::Contradicts => "contradicts",
        Ruling::Unsure => "unsure",
    }
}

/// Runs `judge` over `cases` and scores it: accuracy when acting at `bars`, unsure rate and latency.
pub fn run(name: &str, judge: &dyn Judge, cases: &[Case], bars: Bars) -> Report {
    let (mut acted, mut correct, mut ms) = (0, 0, Vec::with_capacity(cases.len()));
    for (i, c) in cases.iter().enumerate() {
        let claim = Claim {
            id: i.to_string(),
            text: c.claim.clone(),
            evidence: vec![Evidence {
                source: "bench".into(),
                date: "2000-01-01".into(),
                quote: c.quote.clone(),
            }],
        };
        let t = Instant::now();
        let (r, conf, _) = judge.rule(&claim);
        ms.push(t.elapsed().as_secs_f64() * 1e3);
        if r != Ruling::Unsure && conf >= bars.act {
            acted += 1;
            if label(r) == c.label {
                correct += 1;
            }
        }
    }
    ms.sort_by(f64::total_cmp);
    let pct = |p: f64| {
        if ms.is_empty() {
            0.0
        } else {
            ms[percentile_index(ms.len(), p)]
        }
    };
    Report {
        judge: name.into(),
        cases: cases.len(),
        acted,
        correct_when_acting: correct,
        accuracy_when_acting: (acted > 0).then(|| correct as f64 / acted as f64),
        unsure_rate: if cases.is_empty() {
            0.0
        } else {
            (cases.len() - acted) as f64 / cases.len() as f64
        },
        latency_ms_p50: pct(0.5),
        latency_ms_p95: pct(0.95),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ej_core::Layer;

    /// Says Supports at 0.9 when the quote contains "yes", Contradicts at 0.5 when it contains "no", else Unsure.
    struct Keyword;
    impl Judge for Keyword {
        fn layer(&self) -> Layer {
            Layer::LocalJudge
        }
        fn rule(&self, c: &Claim) -> (Ruling, f32, String) {
            let q = &c.evidence[0].quote;
            if q.contains("yes") {
                (Ruling::Supports, 0.9, String::new())
            } else if q.contains("no") {
                (Ruling::Contradicts, 0.5, String::new())
            } else {
                (Ruling::Unsure, 0.0, String::new())
            }
        }
    }

    #[test]
    fn scores_only_acted_cases_and_counts_unsure() {
        let cases = load_cases(concat!(
            "{\"claim\":\"a\",\"quote\":\"yes\",\"label\":\"supports\"}\n",
            "{\"claim\":\"b\",\"quote\":\"yes\",\"label\":\"contradicts\"}\n",
            "\n",
            "{\"claim\":\"c\",\"quote\":\"no\",\"label\":\"contradicts\"}\n",
            "{\"claim\":\"d\",\"quote\":\"-\",\"label\":\"unsure\"}\n",
        ))
        .unwrap();
        let r = run("kw", &Keyword, &cases, Bars::default());
        assert_eq!((r.cases, r.acted, r.correct_when_acting), (4, 2, 1));
        assert_eq!(r.accuracy_when_acting, Some(0.5));
        assert!((r.unsure_rate - 0.5).abs() < f64::EPSILON); // the low-confidence Contradicts does not act
    }

    #[test]
    fn teacher_pairs_load_as_cases() {
        let c = load_cases(
            "{\"claim\":\"a\",\"quote\":\"q\",\"ruling\":\"supports\",\"reason\":\"r\"}",
        )
        .unwrap();
        assert_eq!(c[0].label, "supports");
    }

    #[test]
    fn empty_set_has_no_accuracy() {
        let r = run("kw", &Keyword, &[], Bars::default());
        assert_eq!((r.accuracy_when_acting, r.unsure_rate), (None, 0.0));
    }
}
