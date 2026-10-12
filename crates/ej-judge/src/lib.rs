//! Layers 2 and 3: judges for the grey zone layer 1 cannot settle.
//! A judge answers supports / contradicts / unsure with a confidence. It never writes the claim or the evidence.

pub mod claude;

use ej_core::{Claim, Finding, Layer, Verdict};

/// A judge's answer on whether the evidence states the claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ruling {
    /// The evidence states or directly implies the claim.
    Supports,
    /// The evidence states something incompatible with the claim.
    Contradicts,
    /// The judge cannot decide.
    Unsure,
}

/// A layer 2 or 3 judge for claims layer 1 could not settle.
pub trait Judge {
    /// The layer this judge belongs to, recorded in its findings.
    fn layer(&self) -> Layer;
    /// (ruling, confidence 0..1, reason)
    fn rule(&self, claim: &Claim) -> (Ruling, f32, String);
    /// Raw probability that the evidence states the claim, for calibration and threshold fitting. None if the judge has no such score.
    fn p_stated(&self, _claim: &Claim) -> Option<f32> {
        None
    }
    /// True if this judge only ever returns a non-Unsure ruling after clearing its own fitted thresholds, so the
    /// shared `Bars` must not gate it a second time (a calibrated judge). False for raw judges.
    fn self_gated(&self) -> bool {
        false
    }
}

/// Bars a ruling must clear before it is acted on; anything below is Unsure and escalates.
#[derive(Debug, Clone, Copy)]
pub struct Bars {
    /// Minimum confidence for a raw judge's Supports or Contradicts to be acted on.
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
        // A local judge may not call a claim stated when one of its names, or any other content word, is missing from
        // every quote (ej_evidence::names_in_quotes, ej_evidence::unstated_words); the claim escalates instead. The
        // escalation judge checks the wording itself.
        let quotes: Vec<&str> = claim.evidence.iter().map(|e| e.quote.as_str()).collect();
        if r == Ruling::Supports && j.layer() == Layer::LocalJudge {
            let missing = ej_evidence::unstated_words(&claim.text, &quotes);
            if missing > 0
                || !ej_evidence::names_in_quotes(&claim.text, &quotes)
                || !ej_evidence::figures_backed(&claim.text, &quotes)
            {
                last = format!(
                    "{:?} said supports but {missing} claim word(s), a name or a figure are not in the quotes",
                    j.layer()
                );
                continue;
            }
        }
        if r != Ruling::Unsure && (j.self_gated() || c >= bars.act) {
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
    /// Platt slope on the logit of the raw score.
    pub a: f64,
    /// Platt intercept.
    pub b: f64,
    /// Calibrated score above which the judge rules Supports.
    pub yes: f64,
    /// Calibrated score below which the judge rules Contradicts.
    pub no: f64,
}

impl Calibration {
    /// Maps a raw P(stated) to a calibrated probability (raw score clamped away from 0 and 1 first).
    #[must_use]
    pub fn apply(&self, p: f64) -> f64 {
        let p = p.clamp(1e-6, 1.0 - 1e-6);
        1.0 / (1.0 + (-(self.a * (p / (1.0 - p)).ln() + self.b)).exp())
    }
}

/// Wraps a judge that reports `p_stated` and acts only past its fitted thresholds: Supports above `yes`,
/// Contradicts (not stated) below `no`, Unsure in between. Confidence is the calibrated probability of the call.
pub struct Calibrated<J: Judge> {
    /// The raw judge whose `p_stated` is calibrated.
    pub inner: J,
    /// The fitted calibration and thresholds.
    pub cal: Calibration,
}

impl<J: Judge> Judge for Calibrated<J> {
    fn layer(&self) -> Layer {
        self.inner.layer()
    }
    fn p_stated(&self, claim: &Claim) -> Option<f32> {
        self.inner.p_stated(claim)
    }
    fn self_gated(&self) -> bool {
        true
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
           // Claim words all in the quote, so only the thresholds decide (the wording gate is tested on its own).
        let cl = c("x", "x");
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
    fn default_judge_has_no_score_and_is_not_self_gated() {
        let f = Fixed(Ruling::Supports, 0.9);
        assert_eq!(f.p_stated(&c("x", "y")), None);
        assert!(!f.self_gated());
    }

    #[test]
    fn calibration_formula_and_wrapper_pass_through() {
        // sigmoid(2 * logit(0.7) + 1) = 1 / (1 + exp(-(2 ln(7/3) + 1)))
        let cal = Calibration {
            a: 2.0,
            b: 1.0,
            yes: 0.9,
            no: 0.1,
        };
        let want = 1.0 / (1.0 + (-(2.0 * (0.7f64 / 0.3).ln() + 1.0)).exp());
        assert!((cal.apply(0.7) - want).abs() < 1e-12);
        // Scores of exactly 0 or 1 are clamped inside (0, 1), so the result stays finite and short of certainty.
        let id = Calibration {
            a: 1.0,
            b: 0.0,
            yes: 0.9,
            no: 0.1,
        };
        assert!(id.apply(1.0) < 1.0 && id.apply(1.0) > 0.999_99 && id.apply(0.0) > 0.0);
        let w = Calibrated {
            inner: Score(0.42),
            cal: Calibration {
                a: 1.0,
                b: 0.0,
                yes: 0.7,
                no: 0.2,
            },
        };
        assert_eq!(w.p_stated(&c("x", "y")), Some(0.42));
        assert!(w.self_gated());
        let (r, conf, _) = Calibrated {
            inner: Score(0.1),
            cal: Calibration {
                a: 1.0,
                b: 0.0,
                yes: 0.7,
                no: 0.2,
            },
        }
        .rule(&c("x", "y"));
        assert_eq!(r, Ruling::Contradicts);
        assert!((conf - 0.9).abs() < 1e-6);
    }

    #[test]
    fn overlap_ignores_short_words() {
        // "is" and "ok" are too short to count; "the" and "fee" are both in the quote.
        assert_eq!(
            OverlapJudge.rule(&c("the fee is ok", "the fee")).0,
            Ruling::Supports
        );
    }

    #[test]
    fn overlap_sees_negation_through_punctuation() {
        for q in [
            "it does not, the fee does apply",
            "Not. The fee does apply",
            "the fee does apply (isn’t it)",
        ] {
            assert_eq!(
                OverlapJudge.rule(&c("the fee does apply", q)).0,
                Ruling::Unsure,
                "{q}"
            );
        }
    }

    #[test]
    fn overlap_scores_the_share_of_claim_words_found() {
        let (r, c, _) = OverlapJudge.rule(&c("violet lamp glows brightly", "the violet lamp"));
        assert_eq!(r, Ruling::Unsure);
        assert!((c - 0.5).abs() < 1e-6, "{c}");
    }

    #[test]
    fn local_supports_needs_every_content_word_and_name_in_the_quotes() {
        let cal = Calibration {
            a: 1.0,
            b: 0.0,
            yes: 0.7,
            no: 0.2,
        };
        let verdict = |claim: &str, quote: &str| {
            adjudicate(
                &c(claim, quote),
                &[&Calibrated {
                    inner: Score(0.99),
                    cal,
                }],
                Bars::default(),
            )
            .verdict
        };
        // One word swapped against the quote: escalates.
        assert_eq!(
            verdict(
                "The game got mixed reviews",
                "The game got negative reviews"
            ),
            Verdict::Unsure
        );
        // A bound the quote does not meet is never stated on wording alone.
        assert_eq!(
            verdict("He was born before 1950", "He was born 1951"),
            Verdict::Unsure
        );
        // A name that is also a function word ("May") is still checked as a name.
        assert_eq!(
            verdict("Then May arrived", "Then June arrived"),
            Verdict::Unsure
        );
        // Every word present: the calibrated local call stands.
        assert_eq!(
            verdict(
                "The game got negative reviews",
                "The game got negative reviews"
            ),
            Verdict::Supported
        );
    }

    #[test]
    fn local_supports_needs_every_name_in_the_quotes() {
        let cal = Calibration {
            a: 1.0,
            b: 0.0,
            yes: 0.7,
            no: 0.2,
        };
        let wrong_name = c(
            "Then Rush Fischer praised it",
            "In his review, Russ Fischer praised it",
        );
        let f = adjudicate(
            &wrong_name,
            &[&Calibrated {
                inner: Score(0.99),
                cal,
            }],
            Bars::default(),
        );
        assert_eq!(f.verdict, Verdict::Unsure);
        assert!(f.reason.contains("name"));
        // A local "not stated" is unaffected, and a matching name lets "supports" through.
        let f = adjudicate(
            &wrong_name,
            &[&Calibrated {
                inner: Score(0.05),
                cal,
            }],
            Bars::default(),
        );
        assert_eq!(f.verdict, Verdict::Mismatch);
        let right_name = c(
            "Then Russ Fischer praised it",
            "In his review, Russ Fischer praised it",
        );
        let f = adjudicate(
            &right_name,
            &[&Calibrated {
                inner: Score(0.99),
                cal,
            }],
            Bars::default(),
        );
        assert_eq!(f.verdict, Verdict::Supported);
        // The escalation layer is not overridden by the local rule.
        let f = adjudicate(
            &wrong_name,
            &[&Fixed(Ruling::Supports, 0.95)],
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
