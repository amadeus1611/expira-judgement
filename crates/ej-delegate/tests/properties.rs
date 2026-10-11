//! Property tests for routing: the chosen start tier is never costlier than any other allowed start,
//! and a run always ends at a passed gate or the top tier.

use ej_delegate::cost::{best_start, expected_cost, Stats};
use ej_delegate::{run, Launch, Roster, Task, Tier};
use proptest::prelude::*;
use std::collections::BTreeMap;

const TIERS: [Tier; 4] = [Tier::Scout, Tier::Worker, Tier::Builder, Tier::Reviewer];

fn tier() -> impl Strategy<Value = Tier> {
    prop::sample::select(TIERS.to_vec())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn best_start_is_cheapest(log in prop::collection::vec((tier(), any::<bool>()), 0..200), costs in prop::collection::vec(0.01f64..100.0, 4), floor in tier()) {
        let launches: Vec<Launch> = log.iter().map(|(t, ok)| Launch { task_id: "t".into(), tier: *t, gate_passed: *ok }).collect();
        let stats = Stats::from_log(launches.iter().map(|l| ("k", l)));
        let cost: BTreeMap<Tier, f64> = TIERS.iter().copied().zip(costs).collect();
        let (best, e) = best_start("k", floor, &stats, &cost).unwrap();
        prop_assert!(best >= floor);
        for t in TIERS.iter().copied().filter(|t| *t >= floor) {
            prop_assert!(e <= expected_cost("k", t, &stats, &cost).unwrap() + 1e-9);
        }
    }

    #[test]
    fn run_ends_at_pass_or_top(kind in prop::sample::select(vec!["lookup", "ledger_rows", "draft", "other"]), stakes in 0.0f64..1e6, passes_at in prop::option::of(tier())) {
        let roster = Roster { models: BTreeMap::new(), high_stakes: 500_000.0 };
        let task = Task { id: "t".into(), kind: kind.into(), stakes, token_budget: 1000 };
        let log = run(&task, &roster, |t| passes_at.is_some_and(|p| t >= p));
        let last = log.last().unwrap();
        prop_assert!(last.gate_passed || last.tier == Tier::Reviewer);
        prop_assert!(log.windows(2).all(|w| !w[0].gate_passed && w[0].tier < w[1].tier));
    }
}
