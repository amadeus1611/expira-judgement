//! Property tests for the statistics that decide when a judge may act.

use ej_bench::calib::{fit_thresholds, selective, wilson_lower, Platt, Scored, Z95};
use proptest::prelude::*;

fn scored() -> impl Strategy<Value = Vec<Scored>> {
    prop::collection::vec(
        (0.0f64..=1.0, any::<bool>()).prop_map(|(p, stated)| Scored { p, stated }),
        1..300,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// The lower bound never exceeds the observed rate, stays in [0, 1], and rises with more successes.
    #[test]
    fn wilson_is_a_lower_bound(n in 1usize..5000, k_frac in 0.0f64..=1.0) {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
        let k = ((n as f64) * k_frac) as usize;
        #[allow(clippy::cast_precision_loss)]
        let rate = k as f64 / n as f64;
        let lb = wilson_lower(k, n, Z95);
        prop_assert!((0.0..=1.0).contains(&lb));
        prop_assert!(lb <= rate + 1e-12);
        if k < n {
            prop_assert!(wilson_lower(k + 1, n, Z95) >= lb);
        }
    }

    /// Fitted thresholds keep their promise on the data they were fitted on: each acting side's
    /// Wilson lower bound meets the target, and the two sides never overlap.
    #[test]
    fn fitted_thresholds_meet_target_in_sample(xs in scored(), target in 0.5f64..0.99, min_acted in 1usize..30) {
        let t = fit_thresholds(&xs, target, min_acted);
        prop_assert!(t.no < t.yes);
        let s = selective(&xs, t);
        if s.yes_acted > 0 {
            prop_assert!(s.yes_acted >= min_acted && wilson_lower(s.yes_right, s.yes_acted, Z95) >= target);
        }
        if s.no_acted > 0 {
            prop_assert!(s.no_acted >= min_acted && wilson_lower(s.no_right, s.no_acted, Z95) >= target);
        }
        prop_assert_eq!(s.acted, s.yes_acted + s.no_acted);
        prop_assert_eq!(s.wrong_yes + s.wrong_no, s.acted - s.right);
    }

    /// Platt scaling with a positive slope keeps the order of scores, so thresholds stay meaningful.
    #[test]
    fn platt_preserves_order(xs in scored(), a in 0.01f64..10.0, b in -5.0f64..5.0) {
        let f = Platt { a, b };
        let mut ps: Vec<f64> = xs.iter().map(|s| s.p).collect();
        ps.sort_by(f64::total_cmp);
        for w in ps.windows(2) {
            prop_assert!(f.apply(w[0]) <= f.apply(w[1]) + 1e-12);
        }
    }
}
