//! Layers 2 and 3: judges for the grey zone layer 1 cannot settle.
//! A judge answers supports / contradicts / unsure with a confidence. It never writes the claim or the evidence.

pub mod claude;

use ej_core::{Claim, Finding, Layer, Verdict};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ruling {
    Supports,
    Contradicts,
    Unsure,
}

pub trait Judge {
    fn layer(&self) -> Layer;
    /// (ruling, confidence 0..1, reason)
    fn rule(&self, claim: &Claim) -> (Ruling, f32, String);
    /// Raw probability that the evidence states the claim, for calibration and threshold fitting. None if the judge has no such score.
    fn p_stated(&self, _claim: &Claim) -> Option<f32> {
        None
    }
    /// Confidence this judge's acted rulings must reach, if it sets its own (a calibrated judge already gates by its
    /// fitted thresholds). None uses the shared `Bars`.
    fn bar(&self) -> Option<f32> {
        None
    }
}

/// Bars a ruling must clear before it is acted on; anything below is Unsure and escalates.
#[derive(Debug, Clone, Copy)]
pub struct Bars {
    pub act: f32,
}
impl Default for Bars {
    fn default() -> Self {
        Bars { act: 0.8 }
    }
}

/// Runs judges in order (cheapest first). The first confident ruling wins; otherwise Unsure.
pub fn adjudicate(claim: &Claim, judges: &[&dyn Judge], bars: Bars) -> Finding {
    let mut last = String::from("no judge available");
    for j in judges {
        let (r, c, why) = j.rule(claim);
        if r != Ruling::Unsure && c >= j.bar().unwrap_or(bars.act) {
            let verdict = if r == Ruling::Supports {
                Verdict::Supported
            } else {
                Verdict::Mismatch
            };
            return Finding {
                claim_id: claim.id.clone(),
                verdict,
                layer: j.layer(),
                confidence: c,
                reason: why,
            };
        }
        last = format!("{:?} below bar ({c:.2}): {why}", j.layer());
    }
    Finding {
        claim_id: claim.id.clone(),
        verdict: Verdict::Unsure,
        layer: judges.last().map_or(Layer::Deterministic, |j| j.layer()),
        confidence: 0.0,
        reason: last,
    }
}

/// Fitted calibration for one judge: Platt scaling on its raw P(stated), then act thresholds on the calibrated score.
/// Produced by `ej-bench fit --write-policy` from labelled cases (public sets, ledger cases, Claude teacher pairs).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Calibration {
    pub a: f64,
    pub b: f64,
    pub yes: f64,
    pub no: f64,
}

impl Calibration {
    #[must_use]
    pub fn apply(&self, p: f64) -> f64 {
        let p = p.clamp(1e-6, 1.0 - 1e-6);
        1.0 / (1.0 + (-(self.a * (p / (1.0 - p)).ln() + self.b)).exp())
    }
}

/// Wraps a judge that reports `p_stated` and acts only past its fitted thresholds: Supports above `yes`,
/// Contradicts (not stated) below `no`, Unsure in between. Confidence is the calibrated probability of the call.
pub struct Calibrated<J: Judge> {
    pub inner: J,
    pub cal: Calibration,
}

impl<J: Judge> Judge for Calibrated<J> {
    fn layer(&self) -> Layer {
        self.inner.layer()
    }
    fn p_stated(&self, claim: &Claim) -> Option<f32> {
        self.inner.p_stated(claim)
    }
    fn bar(&self) -> Option<f32> {
        Some(0.0)
    }
    fn rule(&self, claim: &Claim) -> (Ruling, f32, String) {
        let Some(raw) = self.inner.p_stated(claim) else {
            return (Ruling::Unsure, 0.0, "judge gave no score".into());
        };
        let p = self.cal.apply(f64::from(raw));
        if p >= self.cal.yes {
            (
                Ruling::Supports,
                p as f32,
                format!("calibrated p={p:.3} >= {:.3}", self.cal.yes),
            )
        } else if p <= self.cal.no {
            (
                Ruling::Contradicts,
                (1.0 - p) as f32,
                format!("not stated: calibrated p={p:.3} <= {:.3}", self.cal.no),
            )
        } else {
            (
                Ruling::Unsure,
                0.0,
                format!(
                    "calibrated p={p:.3} between {:.3} and {:.3}",
                    self.cal.no, self.cal.yes
                ),
            )
        }
    }
}

/// Lexical overlap judge: a dependency-free stand-in for the local model until the bake-off picks one.
/// It only ever says Supports on strong overlap, never Contradicts (layer 1 already caught figure mismatches).
pub struct OverlapJudge;
impl Judge for OverlapJudge {
    fn layer(&self) -> Layer {
        Layer::LocalJudge
    }
    fn rule(&self, claim: &Claim) -> (Ruling, f32, String) {
        let words = |s: &str| {
            s.to_lowercase()
                .split(|c: char| !c.is_alphanumeric() && c != '%')
                .filter(|w| w.len() > 2)
                .map(String::from)
                .collect::<std::collections::HashSet<_>>()
        };
        // Overlap cannot see polarity: "X is not Y" shares every word with "X is Y". Leave any negation to a real judge.
        let negs = |s: &str| {
            s.to_lowercase().split_whitespace().any(|w| {
                ej_core::is_negator(
                    w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'' && c != '’'),
                )
            })
        };
        if negs(&claim.text) || claim.evidence.iter().any(|e| negs(&e.quote)) {
            return (
                Ruling::Unsure,
                0.0,
                "negation present: overlap cannot judge polarity".into(),
            );
        }
        let want = words(&claim.text);
        if want.is_empty() {
            return (Ruling::Unsure, 0.0, "no content words".into());
        }
        let best = claim
            .evidence
            .iter()
            .map(|e| {
                let have = words(&e.quote);
                want.iter().filter(|w| have.contains(*w)).count() as f32 / want.len() as f32
            })
            .fold(0.0, f32::max);
        if best >= 0.9 {
            (
                Ruling::Supports,
                best,
                format!("{:.0}% of claim words in quote", best * 100.0),
            )
        } else {
            (
                Ruling::Unsure,
                best,
                format!("only {:.0}% overlap", best * 100.0),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ej_core::Evidence;
    struct Fixed(Ruling, f32);
    impl Judge for Fixed {
        fn layer(&self) -> Layer {
            Layer::Claude
        }
        fn rule(&self, _: &Claim) -> (Ruling, f32, String) {
            (self.0, self.1, "fixed".into())
        }
    }

    fn c(text: &str, quote: &str) -> Claim {
        Claim {
            id: "c".into(),
            text: text.into(),
            evidence: vec![Evidence {
                source: "s".into(),
                date: "2026-01-01".into(),
                quote: quote.into(),
            }],
        }
    }

    #[test]
    fn escalates_when_local_unsure() {
        let cl = c("orders carry the fee", "a management fee applies");
        let f = adjudicate(
            &cl,
            &[&OverlapJudge, &Fixed(Ruling::Supports, 0.95)],
            Bars::default(),
        );
        assert_eq!((f.verdict, f.layer), (Verdict::Supported, Layer::Claude));
    }

    #[test]
    fn unsure_when_nobody_confident() {
        let f = adjudicate(
            &c("x y z words", "other"),
            &[&OverlapJudge, &Fixed(Ruling::Supports, 0.5)],
            Bars::default(),
        );
        assert_eq!(f.verdict, Verdict::Unsure);
    }

    struct Score(f32);
    impl Judge for Score {
        fn layer(&self) -> Layer {
            Layer::LocalJudge
        }
        fn rule(&self, _: &Claim) -> (Ruling, f32, String) {
            (Ruling::Unsure, 0.0, String::new())
        }
        fn p_stated(&self, _: &Claim) -> Option<f32> {
            Some(self.0)
        }
    }

    #[test]
    fn calibrated_judge_acts_only_past_fitted_thresholds() {
        let cal = Calibration {
            a: 1.0,
            b: 0.0,
            yes: 0.7,
            no: 0.2,
        }; // identity scaling
        let cl = c("x", "y");
        assert_eq!(
            Calibrated {
                inner: Score(0.75),
                cal
            }
            .rule(&cl)
            .0,
            Ruling::Supports
        );
        assert_eq!(
            Calibrated {
                inner: Score(0.5),
                cal
            }
            .rule(&cl)
            .0,
            Ruling::Unsure
        );
        assert_eq!(
            Calibrated {
                inner: Score(0.1),
                cal
            }
            .rule(&cl)
            .0,
            Ruling::Contradicts
        );
        // Its own bar (0) applies, not the shared 0.8: a 0.75 call past a 0.7 threshold settles the claim.
        let f = adjudicate(
            &cl,
            &[&Calibrated {
                inner: Score(0.75),
                cal,
            }],
            Bars::default(),
        );
        assert_eq!(f.verdict, Verdict::Supported);
    }

    #[test]
    fn overlap_never_settles_negation() {
        for (t, q) in [
            (
                "the fee does not apply to orders",
                "the fee does apply to orders",
            ),
            (
                "the fee applies to orders",
                "the fee never applies to orders",
            ),
            ("fee isn't applied to orders", "fee applied to orders"),
        ] {
            assert_eq!(OverlapJudge.rule(&c(t, q)).0, Ruling::Unsure, "{t}");
        }
    }

    #[test]
    fn local_settles_strong_overlap() {
        let f = adjudicate(
            &c("fee applies to orders", "the fee applies to all orders"),
            &[&OverlapJudge],
            Bars::default(),
        );
        assert_eq!(
            (f.verdict, f.layer),
            (Verdict::Supported, Layer::LocalJudge)
        );
    }
}
