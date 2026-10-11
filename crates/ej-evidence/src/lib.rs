//! Layer 1: deterministic checks. No model, no network. A claim settled here is settled with certainty.
//!
//! Rules: every number in the claim (with its unit) must appear in some quote; the quote must appear
//! in nothing but its own words (we cannot open the source here, so the quote is the contract); the
//! evidence must carry a source and a parseable date, and be no older than `max_age_days`.

use ej_core::{Claim, Finding, Layer, Verdict};

#[derive(Debug, Clone, PartialEq)]
pub struct Quantity {
    pub value: f64,
    pub unit: String,
}

/// Pull numbers out of text: "PHP 1,250,000", "35%", "2,400 mm", "1.2M". Units are the adjacent token, lower-cased.
pub fn quantities(text: &str) -> Vec<Quantity> {
    let toks: Vec<&str> = text.split_whitespace().collect();
    let mut out = Vec::new();
    for (i, raw) in toks.iter().enumerate() {
        let t = raw.trim_matches(|c: char| c == '(' || c == ')' || c == ',' || c == ';' || c == ':');
        let t = t.trim_end_matches('.');
        let (num, suffix) = split_number(t);
        let Some(mut v) = num else { continue };
        let mut unit = suffix.to_lowercase();
        match unit.as_str() {
            "k" => { v *= 1e3; unit.clear(); }
            "m" if !is_length_context(&toks, i) => { v *= 1e6; unit.clear(); }
            "b" => { v *= 1e9; unit.clear(); }
            _ => {}
        }
        if unit.is_empty() {
            if let Some(next) = toks.get(i + 1) {
                let n = next.trim_matches(|c: char| !c.is_alphanumeric() && c != '%').to_lowercase();
                if is_unit(&n) { unit = n; }
            }
        }
        if unit.is_empty() && i > 0 {
            let p = toks[i - 1].trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
            if is_currency(&p) { unit = p; }
        }
        out.push(Quantity { value: v, unit });
    }
    out
}

fn is_length_context(toks: &[&str], i: usize) -> bool {
    toks.get(i + 1).map_or(false, |n| n.eq_ignore_ascii_case("long") || n.eq_ignore_ascii_case("wide"))
}

fn split_number(t: &str) -> (Option<f64>, &str) {
    let t = t.strip_prefix('₱').unwrap_or(t);
    let end = t.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ',')).unwrap_or(t.len());
    let (n, rest) = t.split_at(end);
    if !n.chars().any(|c| c.is_ascii_digit()) { return (None, ""); }
    (n.replace(',', "").parse().ok(), rest)
}

fn is_currency(s: &str) -> bool { matches!(s, "php" | "usd" | "eur" | "p") }
fn is_unit(s: &str) -> bool {
    is_currency(s) || matches!(s, "%" | "mm" | "cm" | "m" | "sqm" | "m2" | "kg" | "days" | "day" | "pcs" | "units" | "percent")
}

fn same(a: &Quantity, b: &Quantity) -> bool {
    let unit_ok = a.unit == b.unit || a.unit.is_empty() || b.unit.is_empty() || (a.unit == "%" && b.unit == "percent") || (a.unit == "percent" && b.unit == "%");
    unit_ok && (a.value - b.value).abs() <= 1e-9 * a.value.abs().max(1.0)
}

/// Days since 0000-03-01 for a YYYY-MM-DD string (civil calendar), or None.
pub fn day_number(date: &str) -> Option<i64> {
    let p: Vec<i64> = date.get(..10)?.split('-').map(|x| x.parse().ok()).collect::<Option<_>>()?;
    let [y, m, d] = p[..] else { return None };
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let dim = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if !(1..=12).contains(&m) || d < 1 || d > dim[(m - 1) as usize] { return None; }
    let (y, m) = if m <= 2 { (y - 1, m + 9) } else { (y, m - 3) };
    Some(365 * y + y / 4 - y / 100 + y / 400 + (153 * m + 2) / 5 + d - 1)
}

pub struct Policy {
    pub today: String,
    pub max_age_days: Option<i64>,
}

/// Returns Some(finding) when layer 1 can decide; None means "grey zone, ask the judge".
pub fn check(claim: &Claim, policy: &Policy) -> Option<Finding> {
    let f = |verdict, confidence, reason: String| Some(Finding { claim_id: claim.id.clone(), verdict, layer: Layer::Deterministic, confidence, reason });
    let now = day_number(&policy.today);
    // A date after `today` is a typo or a fabrication, never fresh evidence.
    let dated = |e: &&ej_core::Evidence| day_number(&e.date).is_some_and(|d| now.map_or(true, |n| d <= n));
    let usable: Vec<_> = claim.evidence.iter().filter(|e| !e.source.trim().is_empty() && !e.quote.trim().is_empty()).filter(dated).collect();
    if usable.is_empty() {
        return f(Verdict::Unbacked, 1.0, "no evidence row with source, valid past date and quote".into());
    }
    if let (Some(max), Some(now)) = (policy.max_age_days, now) {
        if usable.iter().all(|e| now - day_number(&e.date).unwrap() > max) {
            return f(Verdict::Stale, 1.0, format!("all evidence older than {max} days"));
        }
    }
    let want = quantities(&claim.text);
    let have: Vec<Quantity> = usable.iter().flat_map(|e| quantities(&e.quote)).collect();
    let missing: Vec<&Quantity> = want.iter().filter(|q| !have.iter().any(|h| same(q, h))).collect();
    if !missing.is_empty() {
        let list = missing.iter().map(|q| format!("{} {}", q.value, q.unit).trim().to_string()).collect::<Vec<_>>().join(", ");
        return f(Verdict::Mismatch, 1.0, format!("figures not in any quote: {list}"));
    }
    let norm = |s: &str| s.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ");
    if usable.iter().any(|e| norm(&e.quote).contains(&norm(&claim.text))) {
        return f(Verdict::Backed, 1.0, "claim appears word for word in a quote".into());
    }
    None
}

/// Checks that the listed parts add up to the stated total (tolerance for rounding to one unit).
pub fn sum_matches(parts: &[f64], total: f64) -> bool {
    (parts.iter().sum::<f64>() - total).abs() < 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use ej_core::Evidence;

    fn claim(text: &str, quote: &str, date: &str) -> Claim {
        Claim { id: "c1".into(), text: text.into(), evidence: vec![Evidence { source: "https://example.test/p".into(), date: date.into(), quote: quote.into() }] }
    }
    fn pol() -> Policy { Policy { today: "2026-10-11".into(), max_age_days: Some(365) } }

    #[test]
    fn extracts_quantities() {
        let q = quantities("Fit-out was PHP 1,250,000 at a 35% fee for 2,400 mm.");
        assert_eq!(q, vec![
            Quantity { value: 1_250_000.0, unit: "php".into() },
            Quantity { value: 35.0, unit: "%".into() },
            Quantity { value: 2400.0, unit: "mm".into() },
        ]);
        assert_eq!(quantities("about 1.2M")[0].value, 1_200_000.0);
    }

    #[test]
    fn unbacked_without_evidence() {
        let c = Claim { id: "c".into(), text: "Rent is PHP 900".into(), evidence: vec![] };
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Unbacked);
    }

    #[test]
    fn mismatch_on_wrong_figure() {
        let c = claim("The fee is 40%", "a management fee of 35% applies", "2026-09-01");
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Mismatch);
    }

    #[test]
    fn backed_on_exact_quote() {
        let c = claim("A management fee of 35% applies", "Terms: a management fee of 35% applies to all orders.", "2026-09-01");
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Backed);
    }

    #[test]
    fn grey_zone_goes_to_judge() {
        let c = claim("Orders carry a 35% fee", "a management fee of 35% applies", "2026-09-01");
        assert!(check(&c, &pol()).is_none());
    }

    #[test]
    fn stale_evidence() {
        let c = claim("The fee is 35%", "fee is 35%", "2020-01-01");
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Stale);
    }

    #[test]
    fn bad_date_is_unusable() {
        let c = claim("The fee is 35%", "fee is 35%", "last year");
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Unbacked);
    }

    #[test]
    fn impossible_or_future_dates_are_unusable() {
        assert!(day_number("2026-02-31").is_none() && day_number("2025-02-29").is_none() && day_number("2024-02-29").is_some());
        for d in ["2026-02-31", "2026-12-01"] {
            assert_eq!(check(&claim("The fee is 35%", "fee is 35%", d), &pol()).unwrap().verdict, Verdict::Unbacked, "{d}");
        }
    }

    #[test]
    fn sums() {
        assert!(sum_matches(&[100.0, 250.0], 350.0));
        assert!(!sum_matches(&[100.0, 250.0], 360.0));
    }
}
