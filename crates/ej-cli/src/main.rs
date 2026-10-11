//! ej check <claims.json> [--today YYYY-MM-DD] [--max-age DAYS] [--nli DIR --policy POLICY.json]
//!          [--claude [--model ID] [--pairs FILE] [--usage FILE] [--prices FILE]]
//! Reads an array of claims and runs the cascade: layer 1 (deterministic), then the NLI judge
//! under its fitted calibration (only if the policy says it passed the standard; otherwise it is skipped and its
//! cases escalate), then, with --claude, Claude for what is still unsure, appending each ruling to the teacher-pair
//! file and each call's token usage to the usage log; with --prices (a sourced, dated price file such as
//! config/prices.json) a cost summary goes to stderr. Prints findings as JSON. Exit 1 if any claim fails.

mod http;

use ej_core::Claim;
use ej_evidence::{check, Policy};
use ej_judge::claude::ClaudeJudge;
use ej_judge::{adjudicate, Bars, Calibrated, Calibration, Judge};

const USAGE: &str = "usage: ej check <claims.json> [--today YYYY-MM-DD] [--max-age DAYS] [--nli DIR --policy POLICY.json] [--claude [--model ID] [--pairs FILE] [--usage FILE] [--prices FILE]]";

/// Prints a plain message and exits 2 (bad input or setup), so no failure shows up as a panic.
fn die(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(2)
}

fn read_json<T: serde::de::DeserializeOwned>(path: &str, what: &str) -> T {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| die(&format!("{what} {path}: {e}")));
    serde_json::from_str(&text).unwrap_or_else(|e| die(&format!("{what} {path}: {e}")))
}

/// The escalation judge, if `--claude` is given: the model must be on the Claude-only roster.
fn claude_judge(
    args: &[String],
    opt: &dyn Fn(&str) -> Option<String>,
) -> Option<ClaudeJudge<http::Http>> {
    if !args.iter().any(|a| a == "--claude") {
        return None;
    }
    let model = opt("--model")
        .or_else(|| std::env::var("EJ_CLAUDE_MODEL").ok())
        .unwrap_or_else(|| "claude-opus-5-5".into());
    if !ej_delegate::ALLOWED_MODELS.contains(&model.as_str()) {
        die(&format!(
            "--claude: {model} is not an allowed model ({})",
            ej_delegate::ALLOWED_MODELS.join(", ")
        ));
    }
    let http = http::Http::from_env().unwrap_or_else(|e| die(&format!("--claude: {e}")));
    Some(ClaudeJudge::new(http, model))
}

/// The calibrated NLI judge, if `--nli` is given and the policy says it passed the standard.
fn nli_judge(opt: &dyn Fn(&str) -> Option<String>) -> Option<Calibrated<ej_nli::NliJudge>> {
    let dir = opt("--nli")?;
    let path = opt("--policy")
        .unwrap_or_else(|| die("--nli needs --policy (from ej-bench fit --write-policy)"));
    let doc: serde_json::Value = read_json(&path, "policy");
    let p = &doc["judges"]["nli"];
    if p["passed_v1"] != true {
        eprintln!("policy: nli has not passed the standard; it is not used and its cases escalate");
        return None;
    }
    let cal: Calibration = serde_json::from_value(p.clone())
        .unwrap_or_else(|e| die(&format!("policy {path}: nli calibration: {e}")));
    let inner = ej_nli::NliJudge::load(std::path::Path::new(&dir))
        .unwrap_or_else(|e| die(&format!("--nli {dir}: {e}")));
    Some(Calibrated { inner, cal })
}

/// Writes teacher pairs and usage, and prints the call count, tokens and (with --prices) the cost to stderr.
fn record_claude(c: &ClaudeJudge<http::Http>, opt: &dyn Fn(&str) -> Option<String>) {
    let pairs = opt("--pairs").unwrap_or_else(|| "teacher_pairs.jsonl".into());
    ej_habits::append_pairs(std::path::Path::new(&pairs), &c.take_pairs())
        .unwrap_or_else(|e| die(&format!("{pairs}: {e}")));
    let usage = c.take_usage();
    let log = opt("--usage").unwrap_or_else(|| "usage.jsonl".into());
    ej_habits::append_jsonl(std::path::Path::new(&log), &usage)
        .unwrap_or_else(|e| die(&format!("{log}: {e}")));
    let (inp, out) = usage
        .iter()
        .fold((0, 0), |a, u| (a.0 + u.input_tokens, a.1 + u.output_tokens));
    let priced = opt("--prices").map(|p| {
        let doc: serde_json::Value = read_json(&p, "prices");
        let prices: std::collections::BTreeMap<String, ej_habits::Price> =
            serde_json::from_value(doc["models"].clone())
                .unwrap_or_else(|e| die(&format!("prices {p}: {e}")));
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args[1] != "check" {
        die(USAGE);
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
    let claims: Vec<Claim> = read_json(&args[2], "claims");
    let claude = claude_judge(&args, &opt);
    let nli = nli_judge(&opt);
    // Only judges that passed the standard may decide. The overlap placeholder never did (on VitaminC it let 9 of 82
    // non-supported claims through), so it is not in the cascade; it remains available to ej-bench for comparison.
    let mut judges: Vec<&dyn Judge> = Vec::new();
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
        record_claude(c, &opt);
    }
    let out = serde_json::to_string_pretty(&findings)
        .unwrap_or_else(|e| die(&format!("serialise findings: {e}")));
    println!("{out}");
    if findings.iter().any(|f| !f.verdict.passes()) {
        std::process::exit(1);
    }
}
