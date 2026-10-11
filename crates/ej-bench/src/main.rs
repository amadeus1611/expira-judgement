//! ej-bench <cases.jsonl> [--bar 0.8] [--overlap] [--nli MODEL_DIR] [--cmd PROGRAM ARGS...]
//!     Scores each requested judge on the same cases and prints one JSON report per judge.
//! ej-bench score <cases.jsonl> --out SCORES.jsonl [--nli MODEL_DIR] [--name NAME] [--cmd PROGRAM ARGS...]
//!     Writes each judge's raw P(stated) per case (one line: judge, i, stated, p), so slow judges run once.
//!     `--name` names the `--cmd` judge (default "cmd").
//! ej-bench fit --cal SCORES.jsonl --test SCORES.jsonl [--target 0.95] [--min-acted 20] [--current JUDGE=YES,NO]...
//!     Fits Platt scaling and act thresholds per judge on --cal, reports on --test, plus the mean ensemble of all judges.
//! `--cmd` must come last: everything after it is the judge command (see command.rs for the protocol).

use ej_bench::calib::{fit_report, mean_ensemble, Platt, Scored, Thresholds};
use ej_bench::{command::CommandJudge, load_cases, run};
use ej_core::{Claim, Evidence};
use ej_judge::{Bars, Judge, OverlapJudge};
use std::collections::BTreeMap;
use std::io::Write;

fn die(e: String) -> ! { eprintln!("{e}"); std::process::exit(2) }
fn usage() -> ! { die("usage: ej-bench <cases.jsonl> [--bar 0.8] [--overlap] [--nli DIR] [--cmd ...] | score <cases.jsonl> --out FILE ... | fit --cal FILE --test FILE".into()) }

fn judges(head: &[String], tail: Option<&[String]>, opt: &dyn Fn(&str) -> Option<String>) -> Vec<(String, Box<dyn Judge>)> {
    let mut judges: Vec<(String, Box<dyn Judge>)> = Vec::new();
    if head.iter().any(|a| a == "--overlap") { judges.push(("overlap".into(), Box::new(OverlapJudge))); }
    if let Some(dir) = opt("--nli") {
        let j = ej_nli::NliJudge::load(std::path::Path::new(&dir)).unwrap_or_else(|e| die(format!("--nli: {e}")));
        judges.push(("nli".into(), Box::new(j)));
    }
    if let Some(argv) = tail {
        let name = opt("--name").unwrap_or_else(|| "cmd".into());
        judges.push((name, Box::new(CommandJudge::spawn(argv).unwrap_or_else(|e| die(format!("--cmd: {e}"))))));
    }
    judges
}

fn read_scores(path: &str) -> BTreeMap<String, Vec<Scored>> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| die(format!("{path}: {e}")));
    let mut rows: BTreeMap<String, Vec<(u64, Scored)>> = BTreeMap::new();
    for l in text.lines().filter(|l| !l.trim().is_empty()) {
        let v: serde_json::Value = serde_json::from_str(l).unwrap_or_else(|e| die(format!("{path}: {e}")));
        let (Some(j), Some(i), Some(st), Some(p)) = (v["judge"].as_str(), v["i"].as_u64(), v["stated"].as_bool(), v["p"].as_f64()) else { die(format!("{path}: bad row {l}")) };
        rows.entry(j.into()).or_default().push((i, Scored { p, stated: st }));
    }
    rows.into_iter().map(|(k, mut v)| { v.sort_by_key(|r| r.0); (k, v.into_iter().map(|r| r.1).collect()) }).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd_at = args.iter().position(|a| a == "--cmd");
    let head = &args[..cmd_at.unwrap_or(args.len())];
    let tail = cmd_at.map(|i| &args[i + 1..]);
    let opt = |k: &str| head.iter().position(|a| a == k).and_then(|i| head.get(i + 1)).cloned();

    match args.get(1).map(String::as_str) {
        Some("score") => {
            let path = args.get(2).unwrap_or_else(|| usage());
            let out = opt("--out").unwrap_or_else(|| usage());
            let cases = load_cases(&std::fs::read_to_string(path).unwrap_or_else(|e| die(format!("{path}: {e}")))).unwrap_or_else(|e| die(format!("{path}: {e}")));
            let js = judges(head, tail, &opt);
            let mut f = std::fs::File::create(&out).unwrap_or_else(|e| die(format!("{out}: {e}")));
            for (name, j) in &js {
                for (i, c) in cases.iter().enumerate() {
                    let claim = Claim { id: i.to_string(), text: c.claim.clone(), evidence: vec![Evidence { source: "bench".into(), date: "2000-01-01".into(), quote: c.quote.clone() }] };
                    let Some(p) = j.p_stated(&claim) else { die(format!("judge {name} gave no score on case {i}")) };
                    writeln!(f, "{}", serde_json::json!({ "judge": name, "i": i, "stated": c.label == "supports", "p": p })).unwrap();
                }
            }
        }
        Some("fit") => {
            let (cal, test) = (read_scores(&opt("--cal").unwrap_or_else(|| usage())), read_scores(&opt("--test").unwrap_or_else(|| usage())));
            let target: f64 = opt("--target").map_or(0.95, |s| s.parse().unwrap_or_else(|_| die("bad --target".into())));
            let min_acted: usize = opt("--min-acted").map_or(20, |s| s.parse().unwrap_or_else(|_| die("bad --min-acted".into())));
            let mut current = BTreeMap::new();
            for (i, a) in head.iter().enumerate() {
                if a != "--current" { continue; }
                let spec = head.get(i + 1).unwrap_or_else(|| usage());
                let (j, t) = spec.split_once('=').unwrap_or_else(|| usage());
                let (y, n) = t.split_once(',').unwrap_or_else(|| usage());
                current.insert(j.to_string(), Thresholds { yes: y.parse().unwrap_or_else(|_| usage()), no: n.parse().unwrap_or_else(|_| usage()) });
            }
            let mut cal_cal = Vec::new();
            let mut test_cal = Vec::new();
            for (name, c) in &cal {
                let Some(t) = test.get(name) else { continue };
                let r = fit_report(name, c, t, target, min_acted, current.get(name).copied(), true);
                println!("{}", serde_json::to_string(&r).unwrap());
                let f = Platt::fit(c);
                cal_cal.push(c.iter().map(|s| Scored { p: f.apply(s.p), stated: s.stated }).collect::<Vec<_>>());
                test_cal.push(t.iter().map(|s| Scored { p: f.apply(s.p), stated: s.stated }).collect::<Vec<_>>());
            }
            if cal_cal.len() > 1 {
                let r = fit_report("mean_of_calibrated", &mean_ensemble(&cal_cal), &mean_ensemble(&test_cal), target, min_acted, None, false);
                println!("{}", serde_json::to_string(&r).unwrap());
            }
        }
        Some(path) if !path.starts_with("--") => {
            let cases = load_cases(&std::fs::read_to_string(path).unwrap_or_else(|e| die(format!("{path}: {e}")))).unwrap_or_else(|e| die(format!("{path}: {e}")));
            let bars = Bars { act: opt("--bar").map(|s| s.parse().unwrap_or_else(|_| die("bad --bar".into()))).unwrap_or(Bars::default().act) };
            let js = judges(head, tail, &opt);
            if js.is_empty() { die("no judge selected (--overlap, --nli, --cmd)".into()); }
            for (name, j) in &js { println!("{}", serde_json::to_string(&run(name, j.as_ref(), &cases, bars)).unwrap()); }
        }
        _ => usage(),
    }
}
