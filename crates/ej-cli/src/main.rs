//! ej check <claims.json> [--today YYYY-MM-DD] [--max-age DAYS]
//! Reads an array of claims, runs layer 1 then the local judge, prints findings as JSON. Exit 1 if any claim fails.

use ej_core::Claim;
use ej_evidence::{check, Policy};
use ej_judge::{adjudicate, Bars, OverlapJudge};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args[1] != "check" {
        eprintln!("usage: ej check <claims.json> [--today YYYY-MM-DD] [--max-age DAYS]");
        std::process::exit(2);
    }
    let opt = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let policy = Policy { today: opt("--today").unwrap_or_default(), max_age_days: opt("--max-age").and_then(|s| s.parse().ok()) };
    let claims: Vec<Claim> = serde_json::from_str(&std::fs::read_to_string(&args[2]).expect("read claims")).expect("parse claims");
    let findings: Vec<_> = claims.iter().map(|c| check(c, &policy).unwrap_or_else(|| adjudicate(c, &[&OverlapJudge], Bars::default()))).collect();
    println!("{}", serde_json::to_string_pretty(&findings).unwrap());
    if findings.iter().any(|f| !f.verdict.passes()) { std::process::exit(1); }
}
