//! ej check <claims.json> [--today YYYY-MM-DD] [--max-age DAYS] [--nli DIR --policy POLICY.json]
//!          [--claude [--model ID] [--pairs FILE] [--usage FILE] [--prices FILE]]
//! Reads an array of claims and runs the cascade: layer 1 (deterministic), then the overlap judge, then the NLI judge
//! under its fitted calibration (only if the policy says it passed the standard; otherwise it is skipped and its
//! cases escalate), then, with --claude, Claude for what is still unsure, appending each ruling to the teacher-pair
//! file and each call's token usage to the usage log; with --prices (a sourced, dated price file such as
//! config/prices.json) a cost summary goes to stderr. Prints findings as JSON. Exit 1 if any claim fails.

mod http;

use ej_core::Claim;
use ej_evidence::{check, Policy};
use ej_judge::claude::ClaudeJudge;
use ej_judge::{adjudicate, Bars, Calibrated, Calibration, Judge, OverlapJudge};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args[1] != "check" {
        eprintln!("usage: ej check <claims.json> [--today YYYY-MM-DD] [--max-age DAYS] [--claude [--model ID] [--pairs FILE]]");
        std::process::exit(2);
    }
    let opt = |k: &str| {
        args.iter()
            .position(|a| a == k)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let policy = Policy {
        today: opt("--today").unwrap_or_default(),
        max_age_days: opt("--max-age").and_then(|s| s.parse().ok()),
    };
    let claims: Vec<Claim> =
        serde_json::from_str(&std::fs::read_to_string(&args[2]).expect("read claims"))
            .expect("parse claims");

    let claude = args.iter().any(|a| a == "--claude").then(|| {
        let model = opt("--model")
            .or_else(|| std::env::var("EJ_CLAUDE_MODEL").ok())
            .unwrap_or_else(|| "claude-opus-5-5".into());
        if !ej_delegate::ALLOWED_MODELS.contains(&model.as_str()) {
            eprintln!(
                "--claude: {model} is not an allowed model ({})",
                ej_delegate::ALLOWED_MODELS.join(", ")
            );
            std::process::exit(2);
        }
        ClaudeJudge::new(
            http::Http::from_env().unwrap_or_else(|e| {
                eprintln!("--claude: {e}");
                std::process::exit(2)
            }),
            model,
        )
    });
    let nli = opt("--nli").and_then(|dir| {
        let path = opt("--policy").unwrap_or_else(|| {
            eprintln!("--nli needs --policy (from ej-bench fit --write-policy)");
            std::process::exit(2)
        });
        let doc: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("read policy"))
                .expect("parse policy");
        let p = &doc["judges"]["nli"];
        if p["passed_v1"] != true {
            eprintln!(
                "policy: nli has not passed the standard; it is not used and its cases escalate"
            );
            return None;
        }
        let cal: Calibration = serde_json::from_value(p.clone()).expect("policy: nli calibration");
        Some(Calibrated {
            inner: ej_nli::NliJudge::load(std::path::Path::new(&dir)).expect("load nli"),
            cal,
        })
    });
    let mut judges: Vec<&dyn Judge> = vec![&OverlapJudge];
    if let Some(n) = &nli {
        judges.push(n);
    }
    if let Some(c) = &claude {
        judges.push(c);
    }

    let findings: Vec<_> = claims
        .iter()
        .map(|c| check(c, &policy).unwrap_or_else(|| adjudicate(c, &judges, Bars::default())))
        .collect();
    if let Some(c) = &claude {
        let path = opt("--pairs").unwrap_or_else(|| "teacher_pairs.jsonl".into());
        ej_habits::append_pairs(std::path::Path::new(&path), &c.take_pairs())
            .expect("write teacher pairs");
        let usage = c.take_usage();
        ej_habits::append_jsonl(
            std::path::Path::new(&opt("--usage").unwrap_or_else(|| "usage.jsonl".into())),
            &usage,
        )
        .expect("write usage");
        let (inp, out): (u64, u64) = usage
            .iter()
            .fold((0, 0), |a, u| (a.0 + u.input_tokens, a.1 + u.output_tokens));
        let priced = opt("--prices").map(|p| {
            let doc: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&p).expect("read prices"))
                    .expect("parse prices");
            let prices: std::collections::BTreeMap<String, ej_habits::Price> =
                serde_json::from_value(doc["models"].clone()).expect("prices.models");
            usage
                .iter()
                .map(|u| ej_habits::cost(u, &prices))
                .sum::<Option<f64>>()
        });
        let cost = match priced {
            Some(Some(c)) => format!(", ${c:.4}"),
            Some(None) => ", cost unknown (model or cache rate not in the price file)".into(),
            None => String::new(),
        };
        eprintln!(
            "claude: {} calls, {inp} input / {out} output tokens{cost}",
            usage.len()
        );
    }
    println!("{}", serde_json::to_string_pretty(&findings).unwrap());
    if findings.iter().any(|f| !f.verdict.passes()) {
        std::process::exit(1);
    }
}
