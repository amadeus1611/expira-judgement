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

/// Pull numbers out of text: "PHP 1,250,000", "₱1.5M", "$500", "35%", "2,400 mm", "1.2M".
/// Units are canonical: currencies become "php" / "usd" / "eur", "percent" becomes "%", other units lower-cased.
/// Ranges ("1,000-2,000", "1,000 to 2,000", "between 10 and 20 days") give both ends, sharing the unit stated on either.
pub fn quantities(text: &str) -> Vec<Quantity> {
    let toks = range_tokens(text);
    let toks: Vec<&str> = toks.iter().map(String::as_str).collect();
    let mut out: Vec<(usize, Quantity)> = Vec::new();
    for (i, raw) in toks.iter().enumerate() {
        let t = raw.trim_matches(|c: char| c == '(' || c == ')' || c == ',' || c == ';' || c == ':');
        let t = t.trim_end_matches('.');
        let (cur, t) = strip_currency_prefix(t);
        let (num, suffix) = split_number(t);
        let Some(mut v) = num else { continue };
        let mut unit = suffix.to_lowercase();
        match unit.as_str() {
            "k" => { v *= 1e3; unit.clear(); }
            "m" if !is_length_context(&toks, i) => { v *= 1e6; unit.clear(); }
            "b" => { v *= 1e9; unit.clear(); }
            _ => {}
        }
        if let Some(c) = cur { unit = c.into(); }
        if unit.is_empty() {
            if let Some(next) = toks.get(i + 1) {
                let n = next.trim_matches(|c: char| !c.is_alphanumeric() && c != '%').to_lowercase();
                if is_unit(&n) { unit = n; }
            }
        }
        if unit.is_empty() && i > 0 {
            if let Some(c) = currency(toks[i - 1].trim_matches(|c: char| !c.is_alphanumeric() && !"$€₱".contains(c))) { unit = c.into(); }
        }
        out.push((i, Quantity { value: v, unit: canonical_unit(&unit) }));
    }
    for k in 1..out.len() {
        let (i, j) = (out[k - 1].0, out[k].0);
        let link = j == i + 2 && (matches!(toks[i + 1], "-" | "–" | "to") || (toks[i + 1].eq_ignore_ascii_case("and") && i > 0 && toks[i - 1].eq_ignore_ascii_case("between")));
        if !link { continue; }
        let (a, b) = (out[k - 1].1.unit.clone(), out[k].1.unit.clone());
        if a.is_empty() { out[k - 1].1.unit = b; } else if b.is_empty() { out[k].1.unit = a; }
    }
    out.into_iter().map(|(_, q)| q).collect()
}

/// Whitespace tokens, with a single-dash range "A-B" / "A–B" split into "A", "-", "B" when both sides start with a figure.
fn range_tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for t in text.split_whitespace() {
        let parts: Vec<&str> = t.split(['-', '–']).collect();
        let figure = |p: &str| { let (_, rest) = strip_currency_prefix(p); rest.starts_with(|c: char| c.is_ascii_digit()) };
        if parts.len() == 2 && figure(parts[0]) && figure(parts[1]) {
            out.extend([parts[0].to_string(), "-".to_string(), parts[1].to_string()]);
        } else {
            out.push(t.to_string());
        }
    }
    out
}

fn is_length_context(toks: &[&str], i: usize) -> bool {
    toks.get(i + 1).map_or(false, |n| n.eq_ignore_ascii_case("long") || n.eq_ignore_ascii_case("wide"))
}

/// Splits a currency marker glued to the number ("₱900", "$5", "US$5", "PHP1,000", "P900").
fn strip_currency_prefix(t: &str) -> (Option<&'static str>, &str) {
    let digit = t.find(|c: char| c.is_ascii_digit()).unwrap_or(0);
    if digit == 0 { return (None, t); }
    match currency(&t[..digit]) { Some(c) => (Some(c), &t[digit..]), None => (None, t) }
}

fn split_number(t: &str) -> (Option<f64>, &str) {
    let end = t.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ',')).unwrap_or(t.len());
    let (n, rest) = t.split_at(end);
    if !n.starts_with(|c: char| c.is_ascii_digit()) { return (None, ""); }
    (n.trim_end_matches([',', '.']).replace(',', "").parse().ok(), rest)
}

/// Canonical currency code for a marker or word, if it is one.
fn currency(s: &str) -> Option<&'static str> {
    match s.to_lowercase().as_str() {
        "php" | "p" | "₱" | "peso" | "pesos" => Some("php"),
        "usd" | "$" | "us$" | "dollar" | "dollars" => Some("usd"),
        "eur" | "€" | "euro" | "euros" => Some("eur"),
        _ => None,
    }
}

fn canonical_unit(u: &str) -> String {
    if let Some(c) = currency(u) { return c.into(); }
    match u { "percent" => "%".into(), "day" => "days".into(), _ => u.into() }
}

fn is_unit(s: &str) -> bool {
    currency(s).is_some() || matches!(s, "%" | "mm" | "cm" | "m" | "sqm" | "m2" | "kg" | "days" | "day" | "pcs" | "units" | "percent")
}

fn same(a: &Quantity, b: &Quantity) -> bool {
    let unit_ok = a.unit == b.unit || a.unit.is_empty() || b.unit.is_empty();
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
    fn currencies_are_canonical_whatever_the_marker() {
        let q = |t: &str| quantities(t).into_iter().map(|q| (q.value, q.unit)).collect::<Vec<_>>();
        for t in ["PHP 1,500,000", "₱1.5M", "PHP1,500,000", "P1,500,000", "1,500,000 pesos", "Php 1.5M"] {
            assert_eq!(q(t), vec![(1_500_000.0, "php".to_string())], "{t}");
        }
        for t in ["$500", "US$ 500", "US$500", "USD 500", "500 dollars"] { assert_eq!(q(t), vec![(500.0, "usd".to_string())], "{t}"); }
        assert_eq!(q("€20"), vec![(20.0, "eur".to_string())]);
        assert_eq!(q("35 percent"), vec![(35.0, "%".to_string())]);
    }

    #[test]
    fn ranges_give_both_ends_with_shared_unit() {
        let q = |t: &str| quantities(t).into_iter().map(|q| (q.value, q.unit)).collect::<Vec<_>>();
        let php = vec![(1000.0, "php".to_string()), (2000.0, "php".to_string())];
        for t in ["PHP 1,000-2,000", "PHP 1,000–2,000", "PHP 1,000 to 2,000", "1,000-2,000 pesos", "₱1,000-₱2,000"] { assert_eq!(q(t), php, "{t}"); }
        assert_eq!(q("between 10 and 20 days"), vec![(10.0, "days".to_string()), (20.0, "days".to_string())]);
        assert_eq!(q("10 units and 20 kg"), vec![(10.0, "units".to_string()), (20.0, "kg".to_string())]);
    }

    #[test]
    fn range_claim_needs_both_ends_in_evidence() {
        let c = claim("Rates run PHP 1,000-2,500", "rates from PHP 1,000 to 2,000 per day", "2026-09-01");
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Mismatch);
        let c = claim("Rates run PHP 1,000-2,000", "rates from PHP 1,000 to 2,000 per day", "2026-09-01");
        assert!(check(&c, &pol()).is_none());
    }

    #[test]
    fn different_currency_same_number_is_mismatch() {
        let c = claim("The deposit is USD 5,000", "a deposit of PHP 5,000 is required", "2026-09-01");
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Mismatch);
        let c = claim("The deposit is ₱5,000", "a deposit of PHP 5,000 is required", "2026-09-01");
        assert!(check(&c, &pol()).is_none());
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
