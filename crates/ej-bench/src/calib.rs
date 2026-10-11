//! Calibration and selective-prediction maths for the binary question layer 2 answers: does the quote state the claim?
//! Fit on one labelled split, report on another. Nothing here looks at test data while fitting.

use serde::Serialize;

/// One scored case: the judge's raw P(stated) and whether the quote really states the claim.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scored {
    pub p: f64,
    pub stated: bool,
}

/// Area under the ROC curve (probability a stated case outscores an unstated one; ties count half).
pub fn auroc(xs: &[Scored]) -> Option<f64> {
    let (pos, neg): (Vec<&Scored>, Vec<&Scored>) = xs.iter().partition(|s| s.stated);
    if pos.is_empty() || neg.is_empty() {
        return None;
    }
    let mut wins = 0.0;
    for a in &pos {
        for b in &neg {
            wins += if a.p > b.p {
                1.0
            } else if a.p == b.p {
                0.5
            } else {
                0.0
            };
        }
    }
    Some(wins / (pos.len() * neg.len()) as f64)
}

/// Expected calibration error over equal-width bins: how far stated-rates sit from the probabilities claimed.
pub fn ece(xs: &[Scored], bins: usize) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut sum = vec![(0.0, 0.0, 0usize); bins];
    for s in xs {
        let b = ((s.p * bins as f64) as usize).min(bins - 1);
        sum[b].0 += s.p;
        sum[b].1 += s.stated as u8 as f64;
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
    pub a: f64,
    pub b: f64,
}
impl Platt {
    pub fn fit(xs: &[Scored]) -> Platt {
        let (mut a, mut b) = (1.0, 0.0);
        for _ in 0..100 {
            let (mut ga, mut gb, mut haa, mut hab, mut hbb) = (0.0, 0.0, 1e-9, 0.0, 1e-9);
            for s in xs {
                let x = logit(s.p);
                let q = sigmoid(a * x + b);
                let r = q - s.stated as u8 as f64;
                let w = q * (1.0 - q);
                ga += r * x;
                gb += r;
                haa += w * x * x;
                hab += w * x;
                hbb += w;
            }
            let det = haa * hbb - hab * hab;
            if det.abs() < 1e-12 {
                break;
            }
            let (da, db) = ((hbb * ga - hab * gb) / det, (haa * gb - hab * ga) / det);
            // Halve the Newton step until the loss goes down: full steps overshoot on very over-confident scores.
            let (base, mut step) = (Self::loss(xs, a, b), 1.0);
            while step > 1e-6 && Self::loss(xs, a - step * da, b - step * db) > base {
                step /= 2.0;
            }
            a -= step * da;
            b -= step * db;
            if (step * da).abs() + (step * db).abs() < 1e-10 {
                break;
            }
        }
        Platt { a, b }
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
    pub fn apply(&self, p: f64) -> f64 {
        sigmoid(self.a * logit(p) + self.b)
    }
}

/// Act "stated" when p >= yes, "not stated" when p <= no, otherwise escalate.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Thresholds {
    pub yes: f64,
    pub no: f64,
}

/// Wilson score lower bound on a success rate k/n at normal quantile `z` (1.96 for a two-sided 95% interval).
/// Used so a threshold is never chosen on a small lucky sample: the bound, not the raw rate, must clear the target.
pub fn wilson_lower(k: usize, n: usize, z: f64) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let (n, p) = (n as f64, k as f64 / n as f64);
    let d = 1.0 + z * z / n;
    ((p + z * z / (2.0 * n)) - z * ((p * (1.0 - p) / n) + z * z / (4.0 * n * n)).sqrt()) / d
}

pub const Z95: f64 = 1.96;

/// Widest thresholds whose acted cases on `xs` are right at a rate whose 95% Wilson lower bound is at least `target`,
/// on each side separately. `min_acted` is a floor on sample size as well. A side that cannot reach it never acts.
pub fn fit_thresholds(xs: &[Scored], target: f64, min_acted: usize) -> Thresholds {
    let mut ps: Vec<f64> = xs.iter().map(|s| s.p).collect();
    ps.sort_by(|a, b| a.total_cmp(b));
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
    let no = ps
        .iter()
        .rev()
        .copied()
        .find(|&t| ok(xs.iter().filter(|s| s.p <= t).collect(), false))
        .unwrap_or(f64::NEG_INFINITY);
    Thresholds {
        yes,
        no: if no >= yes { yes - f64::EPSILON } else { no },
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Selective {
    pub cases: usize,
    pub coverage: f64,
    pub acted: usize,
    pub right: usize,
    pub accuracy_when_acting: Option<f64>,
    /// "Stated" calls and how many were right; a wrong one passes an unbacked claim (the dangerous error).
    pub yes_acted: usize,
    pub yes_right: usize,
    /// "Not stated" calls and how many were right; a wrong one only costs an escalation or a needless block.
    pub no_acted: usize,
    pub no_right: usize,
    pub wrong_yes: usize,
    pub wrong_no: usize,
}

/// Coverage and accuracy when acting under fixed thresholds. `wrong_yes` (unstated claim passed) is the dangerous error.
pub fn selective(xs: &[Scored], t: Thresholds) -> Selective {
    let (mut ya, mut yr, mut na, mut nr) = (0, 0, 0, 0);
    for s in xs {
        if s.p >= t.yes {
            ya += 1;
            yr += s.stated as usize;
        } else if s.p <= t.no {
            na += 1;
            nr += !s.stated as usize;
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
    fn wilson_bound_is_conservative() {
        assert!((wilson_lower(10, 10, Z95) - 0.7225).abs() < 1e-3); // 10/10 right only proves ~72%
        assert!(wilson_lower(1000, 1000, Z95) > 0.996);
        assert_eq!(wilson_lower(0, 0, Z95), 0.0);
    }

    #[test]
    fn thresholds_meet_target_and_escalate_the_middle() {
        // 60 clean stated cases high, 60 clean unstated low, a mixed band in the middle.
        let mut xs = Vec::new();
        for i in 0..60 {
            xs.push(s(0.9 + i as f64 * 1e-3, true));
            xs.push(s(0.05 + i as f64 * 1e-3, false));
        }
        for i in 0..20 {
            xs.push(s(0.4 + i as f64 * 1e-2, i % 2 == 0));
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
        assert_eq!(t.yes, f64::INFINITY);
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
    pub judge: String,
    pub test_auroc: Option<f64>,
    pub test_ece_raw: f64,
    pub test_ece_calibrated: f64,
    pub platt: Option<Platt>,
    /// Thresholds on the calibrated score, fitted on the calibration split for `target`.
    pub fitted: Thresholds,
    pub test_fitted: Selective,
    /// Fixed raw thresholds in use today (if given), scored on the same test split for comparison.
    pub test_current: Option<Selective>,
}

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
