//! Layers 2 and 3: judges for the grey zone layer 1 cannot settle.
//! A judge answers supports / contradicts / unsure with a confidence. It never writes the claim or the evidence.

pub mod claude;

use ej_core::{Claim, Finding, Layer, Verdict};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ruling { Supports, Contradicts, Unsure }

pub trait Judge {
    fn layer(&self) -> Layer;
    /// (ruling, confidence 0..1, reason)
    fn rule(&self, claim: &Claim) -> (Ruling, f32, String);
}

/// Bars a ruling must clear before it is acted on; anything below is Unsure and escalates.
#[derive(Debug, Clone, Copy)]
pub struct Bars { pub act: f32 }
impl Default for Bars { fn default() -> Self { Bars { act: 0.8 } } }

/// Runs judges in order (cheapest first). The first confident ruling wins; otherwise Unsure.
pub fn adjudicate(claim: &Claim, judges: &[&dyn Judge], bars: Bars) -> Finding {
    let mut last = String::from("no judge available");
    for j in judges {
        let (r, c, why) = j.rule(claim);
        if r != Ruling::Unsure && c >= bars.act {
            let verdict = if r == Ruling::Supports { Verdict::Supported } else { Verdict::Mismatch };
            return Finding { claim_id: claim.id.clone(), verdict, layer: j.layer(), confidence: c, reason: why };
        }
        last = format!("{:?} below bar ({c:.2}): {why}", j.layer());
    }
    Finding { claim_id: claim.id.clone(), verdict: Verdict::Unsure, layer: judges.last().map_or(Layer::Deterministic, |j| j.layer()), confidence: 0.0, reason: last }
}

/// Lexical overlap judge: a dependency-free stand-in for the local model until the bake-off picks one.
/// It only ever says Supports on strong overlap, never Contradicts (layer 1 already caught figure mismatches).
pub struct OverlapJudge;
impl Judge for OverlapJudge {
    fn layer(&self) -> Layer { Layer::LocalJudge }
    fn rule(&self, claim: &Claim) -> (Ruling, f32, String) {
        let words = |s: &str| s.to_lowercase().split(|c: char| !c.is_alphanumeric() && c != '%').filter(|w| w.len() > 2).map(String::from).collect::<std::collections::HashSet<_>>();
        let want = words(&claim.text);
        if want.is_empty() { return (Ruling::Unsure, 0.0, "no content words".into()); }
        let best = claim.evidence.iter().map(|e| { let have = words(&e.quote); want.iter().filter(|w| have.contains(*w)).count() as f32 / want.len() as f32 }).fold(0.0, f32::max);
        if best >= 0.9 { (Ruling::Supports, best, format!("{:.0}% of claim words in quote", best * 100.0)) } else { (Ruling::Unsure, best, format!("only {:.0}% overlap", best * 100.0)) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ej_core::Evidence;
    struct Fixed(Ruling, f32);
    impl Judge for Fixed { fn layer(&self) -> Layer { Layer::Claude } fn rule(&self, _: &Claim) -> (Ruling, f32, String) { (self.0, self.1, "fixed".into()) } }

    fn c(text: &str, quote: &str) -> Claim { Claim { id: "c".into(), text: text.into(), evidence: vec![Evidence { source: "s".into(), date: "2026-01-01".into(), quote: quote.into() }] } }

    #[test]
    fn escalates_when_local_unsure() {
        let cl = c("orders carry the fee", "a management fee applies");
        let f = adjudicate(&cl, &[&OverlapJudge, &Fixed(Ruling::Supports, 0.95)], Bars::default());
        assert_eq!((f.verdict, f.layer), (Verdict::Supported, Layer::Claude));
    }

    #[test]
    fn unsure_when_nobody_confident() {
        let f = adjudicate(&c("x y z words", "other"), &[&OverlapJudge, &Fixed(Ruling::Supports, 0.5)], Bars::default());
        assert_eq!(f.verdict, Verdict::Unsure);
    }

    #[test]
    fn local_settles_strong_overlap() {
        let f = adjudicate(&c("fee applies to orders", "the fee applies to all orders"), &[&OverlapJudge], Bars::default());
        assert_eq!((f.verdict, f.layer), (Verdict::Supported, Layer::LocalJudge));
    }
}
