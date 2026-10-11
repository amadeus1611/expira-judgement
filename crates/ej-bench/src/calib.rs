//! Calibration and selective-prediction maths for the binary question layer 2 answers: does the quote state the claim?
//! Fit on one labelled split, report on another. Nothing here looks at test data while fitting.

use serde::Serialize;

/// One scored case: the judge's raw P(stated) and whether the quote really states the claim.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scored {
    /// The judge's raw probability that the quote states the claim.
    pub p: f64,
    /// The label: true if the quote really states the claim.
    pub stated: bool,
}

/// Area under the ROC curve (probability a stated case outscores an unstated one; ties count half).
#[must_use]
pub fn auroc(xs: &[Scored]) -> Option<f64> {
    let (pos, neg): (Vec<&Scored>, Vec<&Scored>) = xs.iter().partition(|s| s.stated);
    if pos.is_empty() || neg.is_empty() {
        return None;
    }
    let mut wins = 0.0;
    for a in &pos {
        for b in &neg {
            wins += match a.p.total_cmp(&b.p) {
                std::cmp::Ordering::Greater => 1.0,
                std::cmp::Ordering::Equal => 0.5,
                std::cmp::Ordering::Less => 0.0,
            };
        }
    }
    Some(wins / (pos.len() * neg.len()) as f64)
}

/// Equal-width bin for a probability. The value is clamped to [0, 1] first, so the cast cannot see a negative or NaN.
#[allow(clippy::cast_sign_loss)]
fn bin_index(p: f64, bins: usize) -> usize {
    let p = if p.is_nan() { 0.0 } else { p.clamp(0.0, 1.0) };
    ((p * bins as f64) as usize).min(bins - 1)
}

/// Expected calibration error over equal-width bins: how far stated-rates sit from the probabilities claimed.
#[must_use]
pub fn ece(xs: &[Scored], bins: usize) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut sum = vec![(0.0, 0.0, 0usize); bins];
    for s in xs {
        let b = bin_index(s.p, bins);
        sum[b].0 += s.p;
        sum[b].1 += f64::from(u8::from(s.stated));
        sum[b].2 += 1;
    }
    sum.iter()
        .filter(|b| b.2 > 0)
        .map(|(p, y, n)| (*n as f64 / xs.len() as f64) * (p / *n as f64 - y / *n as f64).abs())
        .sum()
}

fn logit(p: f64) -> f64 {
    let p = p.clamp(1e-6, 1.0 - 1e-6);
    (p / (1.0 - p)).ln()
}
fn sigmoid(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}

/// Platt scaling on the logit: calibrated p = sigmoid(a * logit(p) + b). Fitted by Newton's method on log loss.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Platt {
    /// Slope on the logit of the raw score.
    pub a: f64,
    /// Intercept.
    pub b: f64,
}
impl Platt {
    /// Fits `a` and `b` to `xs` by minimising log loss (damped Newton steps).
    ///
    /// Every loop is bounded: a fixed number of Newton iterations, and each step length is the best of a fixed ladder
    /// (1, 1/2, ..., 2^-19, and 0 = stay), so the loss never goes up and no input can make the fit loop forever.
    #[must_use]
    pub fn fit(xs: &[Scored]) -> Platt {
        let (mut a, mut b) = (1.0, 0.0);
        for _ in 0..NEWTON_ITERATIONS {
            let (da, db) = Self::newton_direction(xs, a, b);
            let loss_at = |step: f64| Self::loss(xs, a - step * da, b - step * db);
            let step = (0..=STEP_HALVINGS)
                .map(|k| {
                    if k == STEP_HALVINGS {
                        0.0
                    } else {
                        0.5f64.powi(k)
                    }
                })
                .map(|step| (loss_at(step), step))
                .min_by(|x, y| x.0.total_cmp(&y.0))
                .map_or(0.0, |(_, step)| step);
            a -= step * da;
            b -= step * db;
        }
        Platt { a, b }
    }

    /// The Newton step H^-1 g for the log loss at (a, b). A tiny ridge on the diagonal keeps H positive definite
    /// (det >= 1e-18 > 0 by Cauchy-Schwarz), so the solve never divides by zero.
    fn newton_direction(xs: &[Scored], a: f64, b: f64) -> (f64, f64) {
        let (mut ga, mut gb, mut haa, mut hab, mut hbb) = (0.0, 0.0, 1e-9, 0.0, 1e-9);
        for s in xs {
            let x = logit(s.p);
            let q = sigmoid(a * x + b);
            let r = q - f64::from(u8::from(s.stated));
            let w = q * (1.0 - q);
            ga += r * x;
            gb += r;
            haa += w * x * x;
            hab += w * x;
            hbb += w;
        }
        let det = haa * hbb - hab * hab;
        ((hbb * ga - hab * gb) / det, (haa * gb - hab * ga) / det)
    }

    fn loss(xs: &[Scored], a: f64, b: f64) -> f64 {
        xs.iter()
            .map(|s| {
                let q = sigmoid(a * logit(s.p) + b).clamp(1e-12, 1.0 - 1e-12);
                if s.stated {
                    -q.ln()
                } else {
                    -(1.0 - q).ln()
                }
            })
            .sum()
    }

    /// The calibrated probability for a raw score `p`.
    #[must_use]
    pub fn apply(&self, p: f64) -> f64 {
        sigmoid(self.a * logit(p) + self.b)
    }
}

/// Newton iterations in [`Platt::fit`]; the loss is convex in (a, b), so this is far more than convergence needs.
const NEWTON_ITERATIONS: usize = 100;
/// Step lengths tried per iteration: 1, 1/2, ..., 2^-(STEP_HALVINGS - 1), then 0.
const STEP_HALVINGS: i32 = 20;

/// Act "stated" when p >= yes, "not stated" when p <= no, otherwise escalate.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Thresholds {
    /// Calibrated score at or above which the judge acts "stated".
    pub yes: f64,
    /// Calibrated score at or below which the judge acts "not stated".
    pub no: f64,
}

/// Wilson score lower bound on a success rate k/n at normal quantile `z` (1.96 for a two-sided 95% interval).
/// Used so a threshold is never chosen on a small lucky sample: the bound, not the raw rate, must clear the target.
#[must_use]
pub fn wilson_lower(k: usize, n: usize, z: f64) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let (n, p) = (n as f64, k as f64 / n as f64);
    let d = 1.0 + z * z / n;
    // At k = 0 the exact bound is 0; rounding can land a hair below it, so clamp to the valid range.
    (((p + z * z / (2.0 * n)) - z * ((p * (1.0 - p) / n) + z * z / (4.0 * n * n)).sqrt()) / d)
        .clamp(0.0, 1.0)
}

/// Normal quantile for a two-sided 95% interval.
pub const Z95: f64 = 1.96;

/// Widest thresholds whose acted cases on `xs` are right at a rate whose 95% Wilson lower bound is at least `target`,
/// on each side separately. `min_acted` is a floor on sample size as well. A side that cannot reach it never acts.
#[must_use]
pub fn fit_thresholds(xs: &[Scored], target: f64, min_acted: usize) -> Thresholds {
    let mut ps: Vec<f64> = xs.iter().map(|s| s.p).collect();
    ps.sort_by(f64::total_cmp);
    ps.dedup();
    let ok = |acted: Vec<&Scored>, want: bool| {
        acted.len() >= min_acted
            && wilson_lower(
                acted.iter().filter(|s| s.stated == want).count(),
                acted.len(),
                Z95,
            ) >= target
    };
    let yes = ps
        .iter()
        .copied()
        .find(|&t| ok(xs.iter().filter(|s| s.p >= t).collect(), true))
        .unwrap_or(f64::INFINITY);
    // The "not stated" side is searched only below the "stated" threshold, so the two acting regions never overlap
    // and each is exactly the set of cases its bound was checked on (found by the property test).
    let no = ps
        .iter()
        .rev()
        .copied()
        .filter(|&t| t < yes)
        .find(|&t| ok(xs.iter().filter(|s| s.p <= t).collect(), false))
        .unwrap_or(f64::NEG_INFINITY);
    Thresholds { yes, no }
}

/// Selective-prediction results under fixed thresholds (see [`selective`]).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Selective {
    /// Number of cases scored.
    pub cases: usize,
    /// Share of cases acted on.
    pub coverage: f64,
    /// Cases acted on (either side).
    pub acted: usize,
    /// Acted cases that were right.
    pub right: usize,
    /// `right / acted`; `None` when nothing was acted on.
    pub accuracy_when_acting: Option<f64>,
    /// "Stated" calls and how many were right; a wrong one passes an unbacked claim (the dangerous error).
    pub yes_acted: usize,
    /// "Stated" calls that were right.
    pub yes_right: usize,
    /// "Not stated" calls and how many were right; a wrong one only costs an escalation or a needless block.
    pub no_acted: usize,
    /// "Not stated" calls that were right.
    pub no_right: usize,
    /// "Stated" calls that were wrong (unstated claims passed).
    pub wrong_yes: usize,
    /// "Not stated" calls that were wrong (stated claims blocked or escalated).
    pub wrong_no: usize,
}

/// Coverage and accuracy when acting under fixed thresholds. `wrong_yes` (unstated claim passed) is the dangerous error.
#[must_use]
pub fn selective(xs: &[Scored], t: Thresholds) -> Selective {
    let (mut ya, mut yr, mut na, mut nr) = (0, 0, 0, 0);
    for s in xs {
        if s.p >= t.yes {
            ya += 1;
            yr += usize::from(s.stated);
        } else if s.p <= t.no {
            na += 1;
            nr += usize::from(!s.stated);
        }
    }
    let (acted, right) = (ya + na, yr + nr);
    Selective {
        cases: xs.len(),
        coverage: if xs.is_empty() {
            0.0
        } else {
            acted as f64 / xs.len() as f64
        },
        acted,
        right,
        accuracy_when_acting: (acted > 0).then(|| right as f64 / acted as f64),
        yes_acted: ya,
        yes_right: yr,
        no_acted: na,
        no_right: nr,
        wrong_yes: ya - yr,
        wrong_no: na - nr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn s(p: f64, stated: bool) -> Scored {
        Scored { p, stated }
    }

    #[test]
    fn auroc_orders() {
        assert_eq!(auroc(&[s(0.9, true), s(0.1, false)]), Some(1.0));
        assert_eq!(auroc(&[s(0.1, true), s(0.9, false)]), Some(0.0));
        assert_eq!(auroc(&[s(0.5, true), s(0.5, false)]), Some(0.5));
        assert_eq!(auroc(&[s(0.5, true)]), None);
    }

    #[test]
    fn ece_zero_when_rates_match() {
        let xs: Vec<_> = (0..10).map(|i| s(0.95, i < 9)).collect(); // 90% stated at p=0.95 -> ece 0.05
        assert!((ece(&xs, 10) - 0.05).abs() < 1e-9);
    }

    #[test]
    fn platt_pulls_overconfident_scores_in() {
        // Says 0.99 but is right 70% of the time; says 0.01 and is right 70% of the time.
        let mut xs = Vec::new();
        for i in 0..100 {
            xs.push(s(0.99, i < 70));
            xs.push(s(0.01, i >= 70));
        }
        let f = Platt::fit(&xs);
        assert!(
            (f.apply(0.99) - 0.70).abs() < 0.01 && (f.apply(0.01) - 0.30).abs() < 0.01,
            "{f:?}"
        );
    }

    #[test]
    fn logit_and_apply_match_hand_values() {
        assert!((logit(0.8) - 4f64.ln()).abs() < 1e-12);
        assert!(logit(0.5).abs() < 1e-12);
        assert!((logit(0.0) - (1e-6f64 / (1.0 - 1e-6)).ln()).abs() < 1e-9);
        assert!((logit(1.0) + (1e-6f64 / (1.0 - 1e-6)).ln()).abs() < 1e-9);
        let f = Platt { a: 2.0, b: 0.5 };
        let want = 1.0 / (1.0 + (-(2.0 * 4f64.ln() + 0.5)).exp());
        assert!((f.apply(0.8) - want).abs() < 1e-12);
    }

    #[test]
    fn loss_matches_hand_value_and_stays_finite() {
        let sig = |z: f64| 1.0 / (1.0 + (-z).exp());
        let xs = [s(0.8, true), s(0.2, false)];
        let want = -sig(4f64.ln() + 0.5).ln() - (1.0 - sig(-(4f64.ln()) + 0.5)).ln();
        assert!((Platt::loss(&xs, 1.0, 0.5) - want).abs() < 1e-12);
        // A saturated wrong call is clamped to q = 1 - 1e-12, so its loss is large but finite.
        let l = Platt::loss(&[s(1.0, false)], 1000.0, 0.0);
        assert!((l + (1e-12f64).ln()).abs() < 1e-3, "{l}");
    }

    #[test]
    fn newton_direction_matches_finite_differences() {
        let xs = [
            s(0.9, true),
            s(0.7, false),
            s(0.4, true),
            s(0.2, false),
            s(0.6, true),
        ];
        let (a, b, h) = (0.7, -0.3, 1e-4);
        let l = |a: f64, b: f64| Platt::loss(&xs, a, b);
        let ga = (l(a + h, b) - l(a - h, b)) / (2.0 * h);
        let gb = (l(a, b + h) - l(a, b - h)) / (2.0 * h);
        let haa = (l(a + h, b) - 2.0 * l(a, b) + l(a - h, b)) / (h * h);
        let hbb = (l(a, b + h) - 2.0 * l(a, b) + l(a, b - h)) / (h * h);
        let hab =
            (l(a + h, b + h) - l(a + h, b - h) - l(a - h, b + h) + l(a - h, b - h)) / (4.0 * h * h);
        let det = haa * hbb - hab * hab;
        let (wa, wb) = ((hbb * ga - hab * gb) / det, (haa * gb - hab * ga) / det);
        let (da, db) = Platt::newton_direction(&xs, a, b);
        assert!(
            (da - wa).abs() < 1e-3 && (db - wb).abs() < 1e-3,
            "{da} {db} vs {wa} {wb}"
        );
    }

    #[test]
    fn fit_reaches_the_two_point_optimum() {
        // Two distinct scores: the log-loss optimum reproduces each score's observed rate exactly (0.7 and 0.2).
        let mut xs = Vec::new();
        for i in 0..10 {
            xs.push(s(0.8, i < 7));
            xs.push(s(0.3, i < 2));
        }
        let f = Platt::fit(&xs);
        assert!(
            (f.apply(0.8) - 0.7).abs() < 1e-6 && (f.apply(0.3) - 0.2).abs() < 1e-6,
            "{f:?}"
        );
        // No data: nothing to move towards, the identity map stays.
        assert_eq!(Platt::fit(&[]), Platt { a: 1.0, b: 0.0 });
    }

    #[test]
    fn auroc_counts_all_pairs_and_ece_bins() {
        // 2 stated x 1 unstated: one win, one loss.
        assert_eq!(
            auroc(&[s(0.9, true), s(0.1, true), s(0.5, false)]),
            Some(0.5)
        );
        assert_eq!(
            auroc(&[s(0.9, true), s(0.6, true), s(0.5, false)]),
            Some(1.0)
        );
        // 2 x 2: three wins of four pairs.
        assert_eq!(
            auroc(&[s(0.9, true), s(0.1, true), s(0.5, false), s(0.05, false)]),
            Some(0.75)
        );
        assert_eq!(bin_index(0.0, 10), 0);
        assert_eq!(bin_index(0.25, 10), 2);
        assert_eq!(bin_index(0.55, 10), 5);
        assert_eq!(bin_index(1.0, 10), 9);
        assert_eq!(bin_index(f64::NAN, 10), 0);
        // Two bins that each look calibrated on their own, but not if pooled.
        let xs = [s(0.95, true), s(0.05, false)];
        assert!((ece(&xs, 10) - 0.05).abs() < 1e-12);
        assert!(ece(&[], 10).abs() < f64::EPSILON);
    }

    #[test]
    fn selective_coverage_and_none_when_idle() {
        let t = Thresholds { yes: 0.8, no: 0.2 };
        let r = selective(
            &[s(0.9, true), s(0.5, true), s(0.5, false), s(0.1, true)],
            t,
        );
        assert!((r.coverage - 0.5).abs() < 1e-12);
        assert_eq!(r.accuracy_when_acting, Some(0.5));
        let idle = selective(&[s(0.5, true)], t);
        assert_eq!((idle.acted, idle.accuracy_when_acting), (0, None));
        assert!(selective(&[], t).coverage.abs() < f64::EPSILON);
    }

    #[test]
    fn no_side_never_reuses_the_yes_threshold() {
        // Target 0 accepts any side with one case: yes takes the lowest score, so no must stay below it (none left).
        let t = fit_thresholds(&[s(0.1, false), s(0.9, true)], 0.0, 1);
        assert!(
            (t.yes - 0.1).abs() < f64::EPSILON && t.no == f64::NEG_INFINITY,
            "{t:?}"
        );
    }

    #[test]
    fn wilson_bound_is_conservative() {
        assert!((wilson_lower(10, 10, Z95) - 0.7225).abs() < 1e-3); // 10/10 right only proves ~72%
        assert!(wilson_lower(1000, 1000, Z95) > 0.996);
        assert!(wilson_lower(0, 0, Z95).abs() < f64::EPSILON);
        // Found by the property test in CI: rounding made the k = 0 bound slightly negative.
        let lb = wilson_lower(0, 303, Z95);
        assert!((0.0..=1e-12).contains(&lb), "{lb}");
    }

    #[test]
    fn thresholds_meet_target_and_escalate_the_middle() {
        // 60 clean stated cases high, 60 clean unstated low, a mixed band in the middle.
        let mut xs = Vec::new();
        for i in 0..60 {
            xs.push(s(0.9 + f64::from(i) * 1e-3, true));
            xs.push(s(0.05 + f64::from(i) * 1e-3, false));
        }
        for i in 0..20 {
            xs.push(s(0.4 + f64::from(i) * 1e-2, i % 2 == 0));
        }
        let t = fit_thresholds(&xs, 0.9, 10);
        assert!(
            t.yes > 0.55 && t.yes <= 0.9 && t.no >= 0.109 && t.no < 0.45,
            "{t:?}"
        );
        let r = selective(&xs, t);
        assert_eq!(r.wrong_yes + r.wrong_no, r.acted - r.right);
        assert!(r.accuracy_when_acting.unwrap() >= 0.95);
        // An unreachable target never acts.
        let t = fit_thresholds(&[s(0.9, false), s(0.8, false)], 0.99, 1);
        assert!(t.yes.is_infinite() && t.yes > 0.0);
    }

    #[test]
    fn fit_report_uses_cal_for_fitting_and_test_for_scores() {
        let cal: Vec<_> = (0..40)
            .map(|i| s(if i % 2 == 0 { 0.9 } else { 0.2 }, i % 2 == 0))
            .collect();
        let test = vec![s(0.9, true), s(0.2, false), s(0.9, false)];
        let r = fit_report(
            "j",
            &cal,
            &test,
            0.8,
            5,
            Some(Thresholds { yes: 0.6, no: 0.3 }),
            true,
        );
        assert_eq!(r.test_fitted.cases, 3);
        assert_eq!(r.test_fitted.wrong_yes, 1);
        assert_eq!(r.test_current.unwrap().acted, 3);
        let m = mean_ensemble(&[vec![s(0.8, true)], vec![s(0.4, true)]]);
        assert!((m[0].p - 0.6).abs() < 1e-12);
    }
}

/// What one judge (or ensemble) achieves on the test split when everything is fitted on the calibration split.
#[derive(Debug, Clone, Serialize)]
pub struct FitReport {
    /// Judge (or ensemble) name.
    pub judge: String,
    /// AUROC of the raw scores on the test split.
    pub test_auroc: Option<f64>,
    /// Expected calibration error of the raw test scores (10 bins).
    pub test_ece_raw: f64,
    /// Expected calibration error of the calibrated test scores (10 bins).
    pub test_ece_calibrated: f64,
    /// The Platt fit from the calibration split, when calibration was asked for.
    pub platt: Option<Platt>,
    /// Thresholds on the calibrated score, fitted on the calibration split for `target`.
    pub fitted: Thresholds,
    /// The fitted thresholds scored on the (calibrated) test split.
    pub test_fitted: Selective,
    /// Fixed raw thresholds in use today (if given), scored on the same test split for comparison.
    pub test_current: Option<Selective>,
}

/// Fits calibration (optional) and thresholds on `cal`, then reports on `test` alongside the `current` raw thresholds.
#[must_use]
pub fn fit_report(
    judge: &str,
    cal: &[Scored],
    test: &[Scored],
    target: f64,
    min_acted: usize,
    current: Option<Thresholds>,
    calibrate: bool,
) -> FitReport {
    let platt = calibrate.then(|| Platt::fit(cal));
    let tr = |xs: &[Scored]| -> Vec<Scored> {
        xs.iter()
            .map(|s| Scored {
                p: platt.map_or(s.p, |f| f.apply(s.p)),
                stated: s.stated,
            })
            .collect()
    };
    let (cal_c, test_c) = (tr(cal), tr(test));
    let fitted = fit_thresholds(&cal_c, target, min_acted);
    FitReport {
        judge: judge.into(),
        test_auroc: auroc(test),
        test_ece_raw: ece(test, 10),
        test_ece_calibrated: ece(&test_c, 10),
        platt,
        fitted,
        test_fitted: selective(&test_c, fitted),
        test_current: current.map(|t| selective(test, t)),
    }
}

/// Mean of each judge's calibrated score, case by case. All inputs must cover the same cases in the same order.
pub fn mean_ensemble(parts: &[Vec<Scored>]) -> Vec<Scored> {
    let n = parts.iter().map(Vec::len).min().unwrap_or(0);
    (0..n)
        .map(|i| Scored {
            p: parts.iter().map(|v| v[i].p).sum::<f64>() / parts.len() as f64,
            stated: parts[0][i].stated,
        })
        .collect()
}
