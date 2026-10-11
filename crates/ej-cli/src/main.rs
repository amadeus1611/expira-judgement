//! ej check <claims.json> [--today YYYY-MM-DD] [--max-age DAYS] [--claude [--model ID] [--pairs FILE]]
//! Reads an array of claims, runs layer 1 then the local judge, and with --claude escalates what is still unsure to
//! Claude, appending each ruling to the teacher-pair file. Prints findings as JSON. Exit 1 if any claim fails.

mod http;

use ej_core::Claim;
use ej_evidence::{check, Policy};
use ej_judge::claude::ClaudeJudge;
use ej_judge::{adjudicate, Bars, Judge, OverlapJudge};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args[1] != "check" {
        eprintln!("usage: ej check <claims.json> [--today YYYY-MM-DD] [--max-age DAYS] [--claude [--model ID] [--pairs FILE]]");
        std::process::exit(2);
    }
    let opt = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let policy = Policy { today: opt("--today").unwrap_or_default(), max_age_days: opt("--max-age").and_then(|s| s.parse().ok()) };
    let claims: Vec<Claim> = serde_json::from_str(&std::fs::read_to_string(&args[2]).expect("read claims")).expect("parse claims");

    let claude = args.iter().any(|a| a == "--claude").then(|| {
        let model = opt("--model").or_else(|| std::env::var("EJ_CLAUDE_MODEL").ok()).unwrap_or_else(|| "claude-opus-5-5".into());
        ClaudeJudge::new(http::Http::from_env().unwrap_or_else(|e| { eprintln!("--claude: {e}"); std::process::exit(2) }), model)
    });
    let mut judges: Vec<&dyn Judge> = vec![&OverlapJudge];
    if let Some(c) = &claude { judges.push(c); }

    let findings: Vec<_> = claims.iter().map(|c| check(c, &policy).unwrap_or_else(|| adjudicate(c, &judges, Bars::default()))).collect();
    if let Some(c) = &claude {
        let path = opt("--pairs").unwrap_or_else(|| "teacher_pairs.jsonl".into());
        ej_habits::append_pairs(std::path::Path::new(&path), &c.take_pairs()).expect("write teacher pairs");
    }
    println!("{}", serde_json::to_string_pretty(&findings).unwrap());
    if findings.iter().any(|f| !f.verdict.passes()) { std::process::exit(1); }
}
