//! Property tests for layer 1: invariants checked on thousands of generated inputs, not a handful of examples.
//! Layer 1 is the only layer allowed to call a claim proven, so its rules must hold for every input.

use ej_core::{Claim, Evidence, Verdict};
use ej_evidence::{check, day_number, quantities, Policy};
use proptest::prelude::*;

const TODAY: &str = "2026-10-11";

fn policy() -> Policy {
    Policy {
        today: TODAY.into(),
        max_age_days: Some(365),
    }
}

fn claim(text: &str, quote: &str, date: &str) -> Claim {
    Claim {
        id: "p".into(),
        text: text.into(),
        evidence: vec![Evidence {
            source: "https://example.test".into(),
            date: date.into(),
            quote: quote.into(),
        }],
    }
}

/// Plain lowercase words with no figures, negators or month names, so a test controls every quantity and polarity.
fn word() -> impl Strategy<Value = String> {
    "[a-z]{4,9}".prop_filter("not a negator, month or unit word", |w| {
        !ej_core::is_negator(w)
            && ![
                "janu", "febr", "marc", "apri", "june", "july", "augu", "sept", "octo", "nove",
                "dece", "peso", "doll", "euro", "perc", "unit", "days",
            ]
            .iter()
            .any(|m| w.starts_with(m))
    })
}

/// The next calendar day, counted the slow, obvious way (independent of `day_number`'s formula).
fn next_day(y: i64, m: i64, d: i64) -> (i64, i64, i64) {
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let len = match m {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if d < len {
        (y, m, d + 1)
    } else if m < 12 {
        (y, m + 1, 1)
    } else {
        (y + 1, 1, 1)
    }
}

fn iso(d: (i64, i64, i64)) -> String {
    format!("{:04}-{:02}-{:02}", d.0, d.1, d.2)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    /// No text, quote or date, however malformed, makes layer 1 panic.
    #[test]
    fn never_panics(text in "\\PC{0,80}", quote in "\\PC{0,120}", date in "\\PC{0,12}") {
        let _ = quantities(&text);
        let _ = check(&claim(&text, &quote, &date), &policy());
    }

    /// Consecutive calendar days are consecutive day numbers, across months, leap years and centuries.
    #[test]
    fn day_numbers_follow_the_calendar(y in 1600i64..2400, m in 1i64..=12, d in 1i64..=28) {
        let mut cur = (y, m, d);
        for _ in 0..40 {
            let n = next_day(cur.0, cur.1, cur.2);
            let a = day_number(&iso(cur)).unwrap();
            let b = day_number(&iso(n)).unwrap();
            prop_assert_eq!(b, a + 1, "{:?} -> {:?}", cur, n);
            cur = n;
        }
    }

    /// Day 0, month 13 and day 32 are never valid dates.
    #[test]
    fn impossible_dates_rejected(y in 1000i64..3000, d in 1i64..=28) {
        let bad = [format!("{y:04}-13-{d:02}"), format!("{y:04}-01-00"), format!("{y:04}-01-32")];
        for date in &bad {
            prop_assert!(day_number(date).is_none(), "{}", date);
        }
    }

    /// A claim copied word for word from a fresh quote, with matching figures, is backed.
    #[test]
    fn verbatim_span_is_backed(words in prop::collection::vec(word(), 3..20), start in 0usize..20, len in 1usize..6, amount in 1u32..10_000_000) {
        let start = start % words.len();
        let end = (start + len).min(words.len());
        let text = format!("{} PHP {amount}", words[start..end].join(" "));
        let quote = format!("{} {text} {}", words[..start].join(" "), words[end..].join(" "));
        let f = check(&claim(&text, &quote, "2026-09-01"), &policy()).unwrap();
        prop_assert_eq!(f.verdict, Verdict::Backed, "{} | {}", text, quote);
    }

    /// The same claim with a negator just before it in the quote is never called backed.
    #[test]
    fn negated_span_is_never_backed(words in prop::collection::vec(word(), 3..12), neg in prop::sample::select(vec!["not", "no", "never", "without", "doesn't", "isn’t"])) {
        let text = words.join(" ");
        let quote = format!("it is said {neg} {text}");
        let verdict = check(&claim(&text, &quote, "2026-09-01"), &policy()).map(|f| f.verdict);
        prop_assert_ne!(verdict, Some(Verdict::Backed), "{}", quote);
    }

    /// A figure the quote does not carry is always a mismatch, whatever the words around it.
    #[test]
    fn changed_figure_is_mismatch(words in prop::collection::vec(word(), 2..10), amount in 1u32..10_000_000, delta in 1u32..1000) {
        let w = words.join(" ");
        let quote = format!("{w} PHP {amount}");
        let text = format!("{w} PHP {}", amount + delta);
        let f = check(&claim(&text, &quote, "2026-09-01"), &policy()).unwrap();
        prop_assert_eq!(f.verdict, Verdict::Mismatch);
    }

    /// Evidence dated after today is never usable, so the claim is unbacked however well it matches.
    #[test]
    fn future_evidence_is_unusable(words in prop::collection::vec(word(), 2..8), ahead in 1i64..5000) {
        let today = day_number(TODAY).unwrap();
        let mut d = (2026, 10, 11);
        for _ in 0..ahead {
            d = next_day(d.0, d.1, d.2);
        }
        prop_assert_eq!(day_number(&iso(d)).unwrap(), today + ahead);
        let text = words.join(" ");
        let f = check(&claim(&text, &text, &iso(d)), &policy()).unwrap();
        prop_assert_eq!(f.verdict, Verdict::Unbacked);
    }
}
