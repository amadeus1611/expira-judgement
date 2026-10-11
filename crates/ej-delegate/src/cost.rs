//! Cost-aware routing. The static rule (start_tier) is a prior; once attempts are logged, each task kind starts at the
//! tier with the lowest expected total cost of reaching a passed gate, given the measured pass rate and the measured
//! average cost of an attempt at each tier. Quality is not traded: every path still ends at a passed gate or the top tier.
//!
//! Expected cost of starting at tier s: E(s) = sum over t >= s of cost(t) * P(every tier from s to t-1 failed).
//! Pass rates use a Beta(1,1) prior, (passes + 1) / (attempts + 2), so a kind with no history is not judged on nothing.
//! This is the cascade idea studied in FrugalGPT and RouteLLM (see docs/research), applied to our own tier log.

use crate::{Launch, Tier};
use std::collections::BTreeMap;

const TIERS: [Tier; 4] = [Tier::Scout, Tier::Worker, Tier::Builder, Tier::Reviewer];

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Counts { pub attempts: u32, pub passes: u32 }

/// Pass counts per (task kind, tier), from the launch log.
#[derive(Debug, Clone, Default)]
pub struct Stats(pub BTreeMap<(String, Tier), Counts>);

impl Stats {
    pub fn from_log<'a>(log: impl IntoIterator<Item = (&'a str, &'a Launch)>) -> Stats {
        let mut m: BTreeMap<(String, Tier), Counts> = BTreeMap::new();
        for (kind, l) in log {
            let c = m.entry((kind.to_string(), l.tier)).or_default();
            c.attempts += 1;
            c.passes += l.gate_passed as u32;
        }
        Stats(m)
    }
    pub fn pass_rate(&self, kind: &str, t: Tier) -> f64 {
        let c = self.0.get(&(kind.to_string(), t)).copied().unwrap_or_default();
        (c.passes as f64 + 1.0) / (c.attempts as f64 + 2.0)
    }
}

/// Expected cost of starting `kind` at `start`, with `cost[t]` the measured mean cost of one attempt at tier t.
pub fn expected_cost(kind: &str, start: Tier, stats: &Stats, cost: &BTreeMap<Tier, f64>) -> Option<f64> {
    let (mut total, mut reach) = (0.0, 1.0);
    for t in TIERS.into_iter().filter(|t| *t >= start) {
        total += reach * cost.get(&t)?;
        if t == Tier::Reviewer { break; }
        reach *= 1.0 - stats.pass_rate(kind, t);
    }
    Some(total)
}

/// The start tier with the lowest expected cost, never below `floor` (high-stakes work keeps its floor).
/// None when a tier's cost has not been measured yet: then the static rule applies.
pub fn best_start(kind: &str, floor: Tier, stats: &Stats, cost: &BTreeMap<Tier, f64>) -> Option<(Tier, f64)> {
    TIERS.into_iter().filter(|t| *t >= floor)
        .map(|t| expected_cost(kind, t, stats, cost).map(|e| (t, e)))
        .collect::<Option<Vec<_>>>()?
        .into_iter().min_by(|a, b| a.1.total_cmp(&b.1))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn launches(tier: Tier, n: u32, pass: u32) -> Vec<Launch> {
        (0..n).map(|i| Launch { task_id: "t".into(), tier, gate_passed: i < pass }).collect()
    }
    fn costs() -> BTreeMap<Tier, f64> { [(Tier::Scout, 1.0), (Tier::Worker, 3.0), (Tier::Builder, 10.0), (Tier::Reviewer, 40.0)].into() }

    #[test]
    fn cheap_tier_kept_when_it_usually_passes() {
        let log = launches(Tier::Scout, 50, 48);
        let s = Stats::from_log(log.iter().map(|l| ("lookup", l)));
        assert_eq!(best_start("lookup", Tier::Scout, &s, &costs()).unwrap().0, Tier::Scout);
    }

    #[test]
    fn skips_tiers_that_keep_failing() {
        // Scout and Worker almost never pass this kind: paying for them first only adds cost.
        let mut log = launches(Tier::Scout, 40, 0);
        log.extend(launches(Tier::Worker, 40, 1));
        log.extend(launches(Tier::Builder, 40, 39));
        let s = Stats::from_log(log.iter().map(|l| ("draft", l)));
        let (t, e) = best_start("draft", Tier::Scout, &s, &costs()).unwrap();
        assert_eq!(t, Tier::Builder);
        assert!(e < expected_cost("draft", Tier::Scout, &s, &costs()).unwrap());
    }

    #[test]
    fn floor_and_missing_costs_are_respected() {
        let s = Stats::default();
        assert_eq!(best_start("x", Tier::Reviewer, &s, &costs()).unwrap().0, Tier::Reviewer);
        let mut partial = costs();
        partial.remove(&Tier::Builder);
        assert!(best_start("x", Tier::Scout, &s, &partial).is_none());
    }

    #[test]
    fn expected_cost_formula() {
        // No history: every pass rate is 1/2. From Builder: 10 + 0.5 * 40 = 30.
        assert_eq!(expected_cost("x", Tier::Builder, &Stats::default(), &costs()), Some(30.0));
    }
}
