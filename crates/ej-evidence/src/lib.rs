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
/// Dates ("2026-09-01", "1 Sept 2026", "Sept 1, 2026") are one quantity, unit "date", value YYYYMMDD;
/// a month alone ("September 2026", "2026-09") is unit "month", value YYYYMM.
pub fn quantities(text: &str) -> Vec<Quantity> {
    let toks = range_tokens(text);
    let toks: Vec<&str> = toks.iter().map(String::as_str).collect();
    let (mut out, used) = dates(&toks);
    for (i, raw) in toks.iter().enumerate() {
        if used[i] {
            continue;
        }
        let t = raw.trim_matches(|c: char| c == '(' || c == ')' || c == ';' || c == ':');
        let t = t.trim_end_matches('.');
        let (cur, t) = strip_currency_prefix(t);
        let (num, suffix) = split_number(t);
        let Some(mut v) = num else { continue };
        let mut unit = suffix.to_lowercase();
        // A decade: "1970s", "1970's" -> 1970 with no unit.
        if matches!(unit.as_str(), "s" | "'s" | "’s") && (1000.0..=2999.0).contains(&v) {
            unit.clear();
        }
        match unit.as_str() {
            "k" => {
                v *= 1e3;
                unit.clear();
            }
            "m" if !is_length_context(&toks, i) => {
                v *= 1e6;
                unit.clear();
            }
            "b" => {
                v *= 1e9;
                unit.clear();
            }
            _ => {}
        }
        if let Some(c) = cur {
            unit = c.into();
        }
        if unit.is_empty() {
            if let Some(next) = toks.get(i + 1) {
                let n = next
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != '%')
                    .to_lowercase();
                if is_unit(&n) {
                    unit = n;
                }
            }
        }
        if unit.is_empty() && i > 0 {
            if let Some(c) = currency(
                toks[i - 1].trim_matches(|c: char| !c.is_alphanumeric() && !"$€₱".contains(c)),
            ) {
                unit = c.into();
            }
        }
        out.push((
            i,
            Quantity {
                value: v,
                unit: canonical_unit(&unit),
            },
        ));
    }
    for k in 1..out.len() {
        let (i, j) = (out[k - 1].0, out[k].0);
        let link = j == i + 2
            && (matches!(toks[i + 1], "-" | "–" | "to")
                || (toks[i + 1].eq_ignore_ascii_case("and")
                    && i > 0
                    && toks[i - 1].eq_ignore_ascii_case("between")));
        if !link {
            continue;
        }
        let (a, b) = (out[k - 1].1.unit.clone(), out[k].1.unit.clone());
        if a.is_empty() {
            out[k - 1].1.unit = b;
        } else if b.is_empty() {
            out[k].1.unit = a;
        }
    }
    out.sort_by_key(|(i, _)| *i);
    out.into_iter().map(|(_, q)| q).collect()
}

fn month(t: &str) -> Option<i64> {
    const NAMES: [&str; 12] = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    let t = t.trim_matches(|c: char| !c.is_alphabetic()).to_lowercase();
    if t.len() < 3 {
        return None;
    }
    NAMES
        .iter()
        .position(|n| n.starts_with(&t)) // "sept" is a prefix of "september" like any other abbreviation
        .and_then(|i| i64::try_from(i + 1).ok())
}

fn int(t: &str, digits: std::ops::RangeInclusive<usize>) -> Option<i64> {
    let t = t.trim_matches(|c: char| !c.is_alphanumeric());
    let t = ["st", "nd", "rd", "th"]
        .iter()
        .find_map(|o| t.strip_suffix(o))
        .unwrap_or(t); // "6th"
    let t = t.trim_matches(|c: char| !c.is_ascii_digit());
    if !digits.contains(&t.len()) {
        return None;
    }
    t.parse().ok()
}

/// Finds dates and months in the tokens; returns them with their first token index, and which tokens they used.
fn dates(toks: &[&str]) -> (Vec<(usize, Quantity)>, Vec<bool>) {
    let mut out = Vec::new();
    let mut used = vec![false; toks.len()];
    let valid = |y: i64, m: i64, d: i64| day_number(&format!("{y:04}-{m:02}-{d:02}")).is_some();
    let date = |y: i64, m: i64, d: i64| Quantity {
        value: (y * 10000 + m * 100 + d) as f64,
        unit: "date".into(),
    };
    let mut i = 0;
    // Each pass consumes at least one token, so the loop needs at most one pass per token. The explicit bound keeps a
    // broken step (one that stops advancing) from looping forever and exhausting memory.
    for _ in 0..toks.len() {
        if toks.get(i).is_none() {
            break;
        }
        let t = |k: usize| toks.get(i + k).copied().unwrap_or("");
        let iso: Vec<&str> = t(0)
            .trim_matches(|c: char| !c.is_ascii_digit())
            .split('-')
            .collect();
        let found: Option<(Quantity, usize)> =
            if iso.len() == 3 && iso[0].len() == 4 && iso[1].len() == 2 && iso[2].len() == 2 {
                match (iso[0].parse(), iso[1].parse(), iso[2].parse()) {
                    (Ok(y), Ok(m), Ok(d)) if valid(y, m, d) => Some((date(y, m, d), 1)),
                    _ => None,
                }
            } else if iso.len() == 2
                && iso[0].len() == 4
                && iso[1].len() == 2
                && matches!(iso[1].parse::<i64>(), Ok(1..=12))
            {
                iso[0]
                    .parse::<i64>()
                    .ok()
                    .zip(iso[1].parse::<i64>().ok())
                    .map(|(y, m)| {
                        (
                            Quantity {
                                value: (y * 100 + m) as f64,
                                unit: "month".into(),
                            },
                            1,
                        )
                    })
            } else if let (Some(d), Some(m), Some(y)) =
                (int(t(0), 1..=2), month(t(1)), int(t(2), 4..=4))
            {
                valid(y, m, d).then(|| (date(y, m, d), 3))
            } else if let (Some(m), Some(d), Some(y)) =
                (month(t(0)), int(t(1), 1..=2), int(t(2), 4..=4))
            {
                valid(y, m, d).then(|| (date(y, m, d), 3))
            } else if let (Some(m), Some(y)) = (month(t(0)), int(t(1), 4..=4)) {
                Some((
                    Quantity {
                        value: (y * 100 + m) as f64,
                        unit: "month".into(),
                    },
                    2,
                ))
            } else {
                None
            };
        match found {
            Some((q, n)) => {
                out.push((i, q));
                used[i..i + n].iter_mut().for_each(|u| *u = true);
                i += n;
            }
            None => i += 1,
        }
    }
    (out, used)
}

/// Whitespace tokens, with a single-dash range "A-B" / "A–B" split into "A", "-", "B" when both sides start with a figure.
fn range_tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    // A comma or semicolon typed as its own word ("June 24 , 2007") carries nothing and breaks date patterns.
    for t in text
        .split_whitespace()
        .filter(|t| !t.chars().all(|c| c == ',' || c == ';'))
    {
        let parts: Vec<&str> = t.split(['-', '–']).collect();
        let figure = |p: &str| {
            let (_, rest) = strip_currency_prefix(p);
            rest.starts_with(|c: char| c.is_ascii_digit())
        };
        let iso_month = parts.len() == 2
            && parts[0].len() == 4
            && parts[1].len() == 2
            && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
            && matches!(parts[1].parse::<u8>(), Ok(1..=12));
        if parts.len() == 2 && figure(parts[0]) && figure(parts[1]) && !iso_month {
            out.extend([parts[0].to_string(), "-".to_string(), parts[1].to_string()]);
        } else {
            out.push(t.to_string());
        }
    }
    out
}

fn is_length_context(toks: &[&str], i: usize) -> bool {
    toks.get(i + 1)
        .is_some_and(|n| n.eq_ignore_ascii_case("long") || n.eq_ignore_ascii_case("wide"))
}

/// Splits a currency marker glued to the number ("₱900", "$5", "US$5", "PHP1,000", "P900").
fn strip_currency_prefix(t: &str) -> (Option<&'static str>, &str) {
    let digit = t.find(|c: char| c.is_ascii_digit()).unwrap_or(0);
    if digit == 0 {
        return (None, t);
    }
    match currency(&t[..digit]) {
        Some(c) => (Some(c), &t[digit..]),
        None => (None, t),
    }
}

fn split_number(t: &str) -> (Option<f64>, &str) {
    let end = t
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ','))
        .unwrap_or(t.len());
    let (n, rest) = t.split_at(end);
    if !n.starts_with(|c: char| c.is_ascii_digit()) {
        return (None, "");
    }
    (
        n.trim_end_matches([',', '.']).replace(',', "").parse().ok(),
        rest,
    )
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
    if let Some(c) = currency(u) {
        return c.into();
    }
    match u {
        "percent" => "%".into(),
        "day" => "days".into(),
        _ => u.into(),
    }
}

fn is_unit(s: &str) -> bool {
    currency(s).is_some()
        || matches!(
            s,
            "%" | "mm"
                | "cm"
                | "m"
                | "sqm"
                | "m2"
                | "kg"
                | "days"
                | "day"
                | "pcs"
                | "units"
                | "percent"
        )
}

/// Does quote quantity `have` back claim quantity `want`?
fn same(want: &Quantity, have: &Quantity) -> bool {
    // Dates (YYYYMMDD) and months (YYYYMM) are whole numbers held in f64; compare them as integers.
    #[allow(clippy::cast_possible_truncation)]
    let whole = |x: f64| x.round() as i64;
    match (want.unit.as_str(), have.unit.as_str()) {
        // A claimed month is backed by any date in that month.
        ("month", "date") => return whole(have.value) / 100 == whole(want.value),
        // Otherwise dates and months only match their own kind and value; in particular a claimed day is never
        // backed by a month alone, since the claim would be more precise than its evidence.
        ("date" | "month", _) | (_, "date" | "month") => {
            return want.unit == have.unit && whole(want.value) == whole(have.value)
        }
        _ => {}
    }
    let unit_ok = want.unit == have.unit || want.unit.is_empty() || have.unit.is_empty();
    // Relative tolerance only absorbs float noise ("1.2M" vs "1,200,000"); 1,000,000,001 is not 1,000,000,000.
    unit_ok && (want.value - have.value).abs() <= 1e-12 * want.value.abs().max(1.0)
}

/// Days since 0000-03-01 for a YYYY-MM-DD string (civil calendar), or None.
#[must_use]
pub fn day_number(date: &str) -> Option<i64> {
    let p: Vec<i64> = date
        .get(..10)?
        .split('-')
        .map(|x| x.parse().ok())
        .collect::<Option<_>>()?;
    let [y, m, d] = p[..] else { return None };
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let dim = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if !(1..=12).contains(&m) || d < 1 || d > dim[usize::try_from(m - 1).ok()?] {
        return None;
    }
    let (y, m) = if m <= 2 { (y - 1, m + 9) } else { (y, m - 3) };
    Some(365 * y + y / 4 - y / 100 + y / 400 + (153 * m + 2) / 5 + d - 1)
}

pub struct Policy {
    pub today: String,
    pub max_age_days: Option<i64>,
}

/// Returns Some(finding) when layer 1 can decide; None means "grey zone, ask the judge".
#[must_use]
pub fn check(claim: &Claim, policy: &Policy) -> Option<Finding> {
    let f = |verdict, confidence, reason: String| {
        Some(Finding {
            claim_id: claim.id.clone(),
            verdict,
            layer: Layer::Deterministic,
            confidence,
            reason,
        })
    };
    let now = day_number(&policy.today);
    // A date after `today` is a typo or a fabrication, never fresh evidence.
    let dated =
        |e: &&ej_core::Evidence| day_number(&e.date).is_some_and(|d| now.is_none_or(|n| d <= n));
    let usable: Vec<_> = claim
        .evidence
        .iter()
        .filter(|e| !e.source.trim().is_empty() && !e.quote.trim().is_empty())
        .filter(dated)
        .collect();
    if usable.is_empty() {
        return f(
            Verdict::Unbacked,
            1.0,
            "no evidence row with source, valid past date and quote".into(),
        );
    }
    if let (Some(max), Some(now)) = (policy.max_age_days, now) {
        if usable
            .iter()
            .all(|e| day_number(&e.date).is_some_and(|d| now - d > max))
        {
            return f(
                Verdict::Stale,
                1.0,
                format!("all evidence older than {max} days"),
            );
        }
    }
    let want = quantities(&claim.text);
    let have: Vec<Quantity> = usable.iter().flat_map(|e| quantities(&e.quote)).collect();
    let missing: Vec<&Quantity> = want
        .iter()
        .filter(|q| !have.iter().any(|h| same(q, h)))
        .collect();
    if !missing.is_empty() {
        let list = missing
            .iter()
            .map(|q| format!("{} {}", q.value, q.unit).trim().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return f(
            Verdict::Mismatch,
            1.0,
            format!("figures not in any quote: {list}"),
        );
    }
    if usable
        .iter()
        .any(|e| contained_unnegated(&e.quote, &claim.text))
    {
        return f(
            Verdict::Backed,
            1.0,
            "claim appears word for word in a quote".into(),
        );
    }
    None
}

/// The claim appears word for word in the quote at least once with no negator in the four words before it, within
/// the same clause ("no fee applies" does not back "fee applies"). A negated match is left to the judge.
fn contained_unnegated(quote: &str, claim: &str) -> bool {
    // (word, ends a clause)
    let words = |s: &str| {
        s.to_lowercase()
            .split_whitespace()
            .map(|w| {
                (
                    w.trim_matches(|c: char| {
                        !c.is_alphanumeric() && c != '%' && c != '\'' && c != '’'
                    })
                    .to_string(),
                    w.ends_with(['.', ':', ';', '!', '?']),
                )
            })
            .filter(|(w, _)| !w.is_empty())
            .collect::<Vec<_>>()
    };
    let (q, c) = (words(quote), words(claim));
    if c.is_empty() || c.len() > q.len() {
        return false;
    }
    let negated = ej_core::is_negator;
    let negated_before = |i: usize| {
        q[i.saturating_sub(4)..i]
            .iter()
            .rev()
            .take_while(|(_, end)| !end)
            .any(|(w, _)| negated(w))
    };
    (0..=q.len() - c.len()).any(|i| {
        q[i..i + c.len()]
            .iter()
            .map(|(w, _)| w)
            .eq(c.iter().map(|(w, _)| w))
            && !negated_before(i)
    })
}

/// Checks that the listed parts add up to the stated total (tolerance for rounding to one unit).
#[must_use]
pub fn sum_matches(parts: &[f64], total: f64) -> bool {
    (parts.iter().sum::<f64>() - total).abs() < 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ymd(y: i32, m: i32, d: i32) -> f64 {
        f64::from(y * 10_000 + m * 100 + d)
    }
    fn ym(y: i32, m: i32) -> f64 {
        f64::from(y * 100 + m)
    }
    use ej_core::Evidence;

    fn claim(text: &str, quote: &str, date: &str) -> Claim {
        Claim {
            id: "c1".into(),
            text: text.into(),
            evidence: vec![Evidence {
                source: "https://example.test/p".into(),
                date: date.into(),
                quote: quote.into(),
            }],
        }
    }
    fn pol() -> Policy {
        Policy {
            today: "2026-10-11".into(),
            max_age_days: Some(365),
        }
    }

    #[test]
    fn extracts_quantities() {
        let q = quantities("Fit-out was PHP 1,250,000 at a 35% fee for 2,400 mm.");
        assert_eq!(
            q,
            vec![
                Quantity {
                    value: 1_250_000.0,
                    unit: "php".into()
                },
                Quantity {
                    value: 35.0,
                    unit: "%".into()
                },
                Quantity {
                    value: 2400.0,
                    unit: "mm".into()
                },
            ]
        );
        assert!((quantities("about 1.2M")[0].value - 1_200_000.0).abs() < 1e-6);
    }

    #[test]
    fn currencies_are_canonical_whatever_the_marker() {
        let q = |t: &str| {
            quantities(t)
                .into_iter()
                .map(|q| (q.value, q.unit))
                .collect::<Vec<_>>()
        };
        for t in [
            "PHP 1,500,000",
            "₱1.5M",
            "PHP1,500,000",
            "P1,500,000",
            "1,500,000 pesos",
            "Php 1.5M",
        ] {
            assert_eq!(q(t), vec![(1_500_000.0, "php".to_string())], "{t}");
        }
        for t in ["$500", "US$ 500", "US$500", "USD 500", "500 dollars"] {
            assert_eq!(q(t), vec![(500.0, "usd".to_string())], "{t}");
        }
        assert_eq!(q("€20"), vec![(20.0, "eur".to_string())]);
        assert_eq!(q("35 percent"), vec![(35.0, "%".to_string())]);
    }

    #[test]
    fn ranges_give_both_ends_with_shared_unit() {
        let q = |t: &str| {
            quantities(t)
                .into_iter()
                .map(|q| (q.value, q.unit))
                .collect::<Vec<_>>()
        };
        let php = vec![(1000.0, "php".to_string()), (2000.0, "php".to_string())];
        for t in [
            "PHP 1,000-2,000",
            "PHP 1,000–2,000",
            "PHP 1,000 to 2,000",
            "1,000-2,000 pesos",
            "₱1,000-₱2,000",
        ] {
            assert_eq!(q(t), php, "{t}");
        }
        assert_eq!(
            q("between 10 and 20 days"),
            vec![(10.0, "days".to_string()), (20.0, "days".to_string())]
        );
        assert_eq!(
            q("10 units and 20 kg"),
            vec![(10.0, "units".to_string()), (20.0, "kg".to_string())]
        );
    }

    #[test]
    fn range_claim_needs_both_ends_in_evidence() {
        let c = claim(
            "Rates run PHP 1,000-2,500",
            "rates from PHP 1,000 to 2,000 per day",
            "2026-09-01",
        );
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Mismatch);
        let c = claim(
            "Rates run PHP 1,000-2,000",
            "rates from PHP 1,000 to 2,000 per day",
            "2026-09-01",
        );
        assert!(check(&c, &pol()).is_none());
    }

    #[test]
    fn dates_in_text_are_single_values() {
        let q = |t: &str| {
            quantities(t)
                .into_iter()
                .map(|q| (q.value, q.unit))
                .collect::<Vec<_>>()
        };
        let d = vec![(ymd(2026, 9, 1), "date".to_string())];
        for t in [
            "on 2026-09-01.",
            "on 1 Sept 2026",
            "on 1 September 2026,",
            "on Sept 1, 2026",
            "on September 1 2026",
        ] {
            assert_eq!(q(t), d, "{t}");
        }
        assert_eq!(
            q("in September 2026"),
            vec![(ym(2026, 9), "month".to_string())]
        );
        assert_eq!(q("in 2026-09"), vec![(ym(2026, 9), "month".to_string())]);
        // Two dates, the first at the very start: each is found once, in order.
        assert_eq!(
            q("2026-09-01 then 1 Oct 2026"),
            vec![
                (ymd(2026, 9, 1), "date".to_string()),
                (ymd(2026, 10, 1), "date".to_string())
            ]
        );
        assert_eq!(
            q("opened 1 Sept 2026 at PHP 900"),
            vec![
                (ymd(2026, 9, 1), "date".to_string()),
                (900.0, "php".to_string())
            ]
        );
        assert_eq!(q("31 Feb 2026").len(), 2); // not a date: stays two plain numbers
        assert_eq!(
            q("( May 21 , 1967 -- June 24 , 2007 )"),
            vec![
                (ymd(1967, 5, 21), "date".to_string()),
                (ymd(2007, 6, 24), "date".to_string())
            ]
        );
        assert_eq!(
            q("born January 6th, 1924"),
            vec![(ymd(1924, 1, 6), "date".to_string())]
        );
        assert_eq!(q("the 1970's"), q("the early 1970s"));
    }

    #[test]
    fn date_claims_check_against_quote_dates() {
        let c = claim(
            "The branch opened on 1 Sept 2026",
            "The branch opened on 2026-09-01.",
            "2026-09-02",
        );
        assert!(check(&c, &pol()).is_none());
        let c = claim(
            "The branch opened in September 2026",
            "The branch opened on 2026-09-01.",
            "2026-09-02",
        );
        assert!(check(&c, &pol()).is_none());
        let c = claim(
            "The branch opened on 2 Sept 2026",
            "The branch opened on 2026-09-01.",
            "2026-09-02",
        );
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Mismatch);
        let c = claim(
            "The branch opened on 1 Sept 2026",
            "Opened 2026, phase 9, lot 1.",
            "2026-09-02",
        );
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Mismatch);
    }

    #[test]
    fn different_currency_same_number_is_mismatch() {
        let c = claim(
            "The deposit is USD 5,000",
            "a deposit of PHP 5,000 is required",
            "2026-09-01",
        );
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Mismatch);
        let c = claim(
            "The deposit is ₱5,000",
            "a deposit of PHP 5,000 is required",
            "2026-09-01",
        );
        assert!(check(&c, &pol()).is_none());
    }

    #[test]
    fn unbacked_without_evidence() {
        let c = Claim {
            id: "c".into(),
            text: "Rent is PHP 900".into(),
            evidence: vec![],
        };
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Unbacked);
    }

    #[test]
    fn mismatch_on_wrong_figure() {
        let c = claim(
            "The fee is 40%",
            "a management fee of 35% applies",
            "2026-09-01",
        );
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Mismatch);
    }

    #[test]
    fn backed_on_exact_quote() {
        let c = claim(
            "A management fee of 35% applies",
            "Terms: a management fee of 35% applies to all orders.",
            "2026-09-01",
        );
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Backed);
    }

    #[test]
    fn negated_quote_does_not_back() {
        for q in [
            "No management fee of 35% applies to these orders.",
            "It is not true that a management fee of 35% applies.",
            "We don't think a management fee of 35% applies",
            "We don’t think a management fee of 35% applies",
        ] {
            assert!(
                check(
                    &claim("A management fee of 35% applies", q, "2026-09-01"),
                    &pol()
                )
                .is_none(),
                "{q}"
            );
        }
        let c = claim(
            "A management fee of 35% applies",
            "Not for retail: a management fee of 35% applies to bulk orders. Retail has none.",
            "2026-09-01",
        );
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Backed);
    }

    #[test]
    fn grey_zone_goes_to_judge() {
        let c = claim(
            "Orders carry a 35% fee",
            "a management fee of 35% applies",
            "2026-09-01",
        );
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
        assert!(
            day_number("2026-02-31").is_none()
                && day_number("2025-02-29").is_none()
                && day_number("2024-02-29").is_some()
        );
        for d in ["2026-02-31", "2026-12-01"] {
            assert_eq!(
                check(&claim("The fee is 35%", "fee is 35%", d), &pol())
                    .unwrap()
                    .verdict,
                Verdict::Unbacked,
                "{d}"
            );
        }
    }

    #[test]
    fn trims_brackets_and_clause_marks() {
        for t in ["(900)", "900;", "900:", "(PHP 900);"] {
            let q = quantities(t);
            assert_eq!(q.len(), 1, "{t}");
            assert!(
                (q[0].value - 900.0).abs() < 1e-9 && (q[0].unit.is_empty() || q[0].unit == "php"),
                "{t}: {q:?}"
            );
        }
    }

    #[test]
    fn multipliers_and_length_context() {
        let v = |t: &str| {
            quantities(t)
                .into_iter()
                .map(|q| (q.value, q.unit))
                .collect::<Vec<_>>()
        };
        assert_eq!(v("5k"), vec![(5_000.0, String::new())]);
        assert_eq!(v("2b"), vec![(2e9, String::new())]);
        assert_eq!(v("3m"), vec![(3e6, String::new())]);
        assert_eq!(v("3m long"), vec![(3.0, "m".to_string())]);
        assert_eq!(v("2m wide"), vec![(2.0, "m".to_string())]);
        assert_eq!(v("3m tall"), vec![(3e6, String::new())]);
        // The decade rule only applies to "s" after a year-sized number.
        assert_eq!(v("1500kg"), vec![(1500.0, "kg".to_string())]);
        assert_eq!(v("10s"), vec![(10.0, "s".to_string())]);
        assert_eq!(v("1 day"), v("1 days"));
    }

    #[test]
    fn only_real_range_words_link_units() {
        let units = |t: &str| {
            quantities(t)
                .into_iter()
                .map(|q| q.unit)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            units("10 or 20 days"),
            vec![String::new(), "days".to_string()]
        );
        assert_eq!(
            units("10 and 20 days"),
            vec![String::new(), "days".to_string()]
        );
        assert_eq!(
            units("between 10 and 20 days"),
            vec!["days".to_string(), "days".to_string()]
        );
    }

    #[test]
    fn evidence_needs_both_a_source_and_a_quote() {
        let mut c = claim("fee is 35%", "fee is 35%", "2026-09-01");
        c.evidence[0].source = " ".into();
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Unbacked);
        let mut c = claim("fee is 35%", " ", "2026-09-01");
        c.evidence[0].source = "https://example.test/p".into();
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Unbacked);
    }

    #[test]
    fn invalid_named_dates_are_not_dates() {
        assert!(quantities("on 2026-02-31").iter().all(|q| q.unit != "date"));
        assert!(quantities("Feb 30, 2026").iter().all(|q| q.unit != "date"));
        assert!(quantities("30 Feb 2026").iter().all(|q| q.unit != "date"));
    }

    #[test]
    fn date_precision_and_kind_are_respected() {
        // A claimed day needs the day in the quote; a month in the quote is not enough.
        let c = claim(
            "Opened on 1 Sept 2026",
            "Opened in September 2026.",
            "2026-09-02",
        );
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Mismatch);
        // A claimed month is backed by a dated quote in that month (left to the judge, not a mismatch).
        let c = claim(
            "Opened in September 2026",
            "Opened on 2026-09-01.",
            "2026-09-02",
        );
        assert!(check(&c, &pol()).is_none());
        // A date is not the same thing as a bare number with the same digits.
        let c = claim(
            "Code 2026-09-01 applies",
            "Code 20260901 applies",
            "2026-09-02",
        );
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Mismatch);
    }

    #[test]
    fn figures_compare_exactly_but_tolerate_float_noise() {
        let c = claim(
            "Budget PHP 1,000,000,001",
            "budget of PHP 1,000,000,000",
            "2026-09-01",
        );
        assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Mismatch);
        let c = claim("Budget PHP 1.2M", "budget of PHP 1,200,000", "2026-09-01");
        assert!(check(&c, &pol()).is_none());
        // 1.005 x 1000 is 1004.9999999999999 in floating point: still the same figure as 1,005.
        let c = claim("Fee PHP 1.005k", "fee of PHP 1,005", "2026-09-01");
        assert!(check(&c, &pol()).is_none());
        // A unit-less figure in the quote does not contradict the claim's unit.
        let c = claim("The fee PHP 900", "the fee 900 per order", "2026-09-01");
        assert!(check(&c, &pol()).is_none());
    }

    #[test]
    fn staleness_boundary_is_exact() {
        // 2026-10-11 minus 365 days is 2025-10-11: exactly max age is still fresh, one day more is stale.
        let fresh = claim("fee is 35%", "fee is 35%", "2025-10-11");
        assert_eq!(check(&fresh, &pol()).unwrap().verdict, Verdict::Backed);
        let stale = claim("fee is 35%", "fee is 35%", "2025-10-10");
        assert_eq!(check(&stale, &pol()).unwrap().verdict, Verdict::Stale);
    }

    #[test]
    fn punctuation_in_the_quote_does_not_block_a_verbatim_match() {
        // Punctuation around words is ignored when matching, on both sides.
        for q in [
            "Note: the owner's fee applies, to orders.",
            "Note: the owner's fee applies to orders.",
        ] {
            let c = claim("the owner's fee applies to orders", q, "2026-09-01");
            assert_eq!(check(&c, &pol()).unwrap().verdict, Verdict::Backed, "{q}");
        }
    }

    #[test]
    fn calendar_matches_an_independent_count_every_day_1600_to_2400() {
        let mut ymd = (1600i64, 1i64, 1i64);
        let mut expected = day_number("1600-01-01").unwrap();
        while ymd.0 < 2400 {
            let s = format!("{:04}-{:02}-{:02}", ymd.0, ymd.1, ymd.2);
            assert_eq!(day_number(&s), Some(expected), "{s}");
            let leap = (ymd.0 % 4 == 0 && ymd.0 % 100 != 0) || ymd.0 % 400 == 0;
            let len = match ymd.1 {
                2 if leap => 29,
                2 => 28,
                4 | 6 | 9 | 11 => 30,
                _ => 31,
            };
            assert_eq!(
                day_number(&format!("{:04}-{:02}-{:02}", ymd.0, ymd.1, len + 1)),
                None
            );
            ymd = if ymd.2 < len {
                (ymd.0, ymd.1, ymd.2 + 1)
            } else if ymd.1 < 12 {
                (ymd.0, ymd.1 + 1, 1)
            } else {
                (ymd.0 + 1, 1, 1)
            };
            expected += 1;
        }
        // Anchors: the epoch of the formula, and 1970-01-01 to 2000-01-01 is 10,957 days.
        assert_eq!(day_number("0000-03-01"), Some(0));
        assert_eq!(
            day_number("2000-01-01").unwrap() - day_number("1970-01-01").unwrap(),
            10_957
        );
    }

    #[test]
    fn sums() {
        assert!(sum_matches(&[100.0, 250.0], 350.0));
        assert!(sum_matches(&[1.0], 1.49));
        assert!(!sum_matches(&[1.0], 1.5));
        assert!(!sum_matches(&[100.0, 250.0], 360.0));
    }
}
