//! ej-bench <cases.jsonl> [--bar 0.8] [--overlap] [--nli MODEL_DIR] [--cmd PROGRAM ARGS...]
//!     Scores each requested judge on the same cases and prints one JSON report per judge.
//! ej-bench score <cases.jsonl> --out SCORES.jsonl [--nli MODEL_DIR] [--name NAME] [--cmd PROGRAM ARGS...]
//!     Writes each judge's raw P(stated) per case (one line: judge, i, stated, p), so slow judges run once.
//!     `--name` names the `--cmd` judge (default "cmd").
//! ej-bench fit --cal SCORES.jsonl --test SCORES.jsonl [--stress SCORES.jsonl] [--target 0.99] [--min-acted 20]
//!              [--current JUDGE=YES,NO]... [--write-policy POLICY.json]
//!     Fits Platt scaling and act thresholds per judge on --cal, reports on --test, scores --stress with the fitted
//!     policy, and checks each judge (and the mean ensemble) against the standard (standard.rs). Exit 1 if none passes.
//! `--cmd` must come last: everything after it is the judge command (see command.rs for the protocol).

use ej_bench::calib::{fit_report, mean_ensemble, selective, Platt, Scored, Thresholds};
use ej_bench::standard::{check, Evidence as Measured, V1};
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

/// Per judge: scores in case order, and per-case latency in ms when the file carries it.
fn read_scores(path: &str) -> BTreeMap<String, (Vec<Scored>, Option<Vec<f64>>)> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| die(format!("{path}: {e}")));
    let mut rows: BTreeMap<String, Vec<(u64, Scored, Option<f64>)>> = BTreeMap::new();
    for l in text.lines().filter(|l| !l.trim().is_empty()) {
        let v: serde_json::Value = serde_json::from_str(l).unwrap_or_else(|e| die(format!("{path}: {e}")));
        let (Some(j), Some(i), Some(st), Some(p)) = (v["judge"].as_str(), v["i"].as_u64(), v["stated"].as_bool(), v["p"].as_f64()) else { die(format!("{path}: bad row {l}")) };
        rows.entry(j.into()).or_default().push((i, Scored { p, stated: st }, v["ms"].as_f64()));
    }
    rows.into_iter().map(|(k, mut v)| {
        v.sort_by_key(|r| r.0);
        let ms = v.iter().map(|r| r.2).collect::<Option<Vec<f64>>>();
        (k, (v.iter().map(|r| r.1).collect(), ms))
    }).collect()
}

fn p95(ms: &[f64]) -> f64 {
    let mut v = ms.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    v[((v.len() - 1) as f64 * 0.95).round() as usize]
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
                    let t = std::time::Instant::now();
                    let Some(p) = j.p_stated(&claim) else { die(format!("judge {name} gave no score on case {i}")) };
                    let ms = t.elapsed().as_secs_f64() * 1e3;
                    writeln!(f, "{}", serde_json::json!({ "judge": name, "i": i, "stated": c.label == "supports", "p": p, "ms": ms })).unwrap();
                }
            }
        }
        Some("fit") => {
            let (cal, test) = (read_scores(&opt("--cal").unwrap_or_else(|| usage())), read_scores(&opt("--test").unwrap_or_else(|| usage())));
            let stress = opt("--stress").map(|p| read_scores(&p));
            let target: f64 = opt("--target").map_or(V1.pass_lower_bound, |s| s.parse().unwrap_or_else(|_| die("bad --target".into())));
            let min_acted: usize = opt("--min-acted").map_or(20, |s| s.parse().unwrap_or_else(|_| die("bad --min-acted".into())));
            let mut current = BTreeMap::new();
            for (i, a) in head.iter().enumerate() {
                if a != "--current" { continue; }
                let spec = head.get(i + 1).unwrap_or_else(|| usage());
                let (j, t) = spec.split_once('=').unwrap_or_else(|| usage());
                let (y, n) = t.split_once(',').unwrap_or_else(|| usage());
                current.insert(j.to_string(), Thresholds { yes: y.parse().unwrap_or_else(|_| usage()), no: n.parse().unwrap_or_else(|_| usage()) });
            }
            let calibrated = |f: &Platt, xs: &[Scored]| xs.iter().map(|s| Scored { p: f.apply(s.p), stated: s.stated }).collect::<Vec<_>>();
            let (mut cal_all, mut test_all, mut stress_all, mut p95_sum) = (Vec::new(), Vec::new(), Vec::new(), Some(0.0));
            let (mut policy, mut any_passed) = (serde_json::Map::new(), false);
            for (name, (c, _)) in &cal {
                let Some((t, ms)) = test.get(name) else { continue };
                let r = fit_report(name, c, t, target, min_acted, current.get(name).copied(), true);
                let f = r.platt.unwrap();
                let st = stress.as_ref().and_then(|s| s.get(name)).map(|(x, _)| calibrated(&f, x));
                let st_sel = st.as_ref().map(|x| selective(x, r.fitted));
                let lat = ms.as_ref().map(|m| p95(m));
                let v = check(name, &V1, &Measured { test: &r.test_fitted, test_ece: r.test_ece_calibrated, stress: st_sel.as_ref(), p95_ms: lat });
                any_passed |= v.passed;
                println!("{}", serde_json::json!({ "report": r, "stress": st_sel, "test_p95_ms": lat, "standard_v1": v }));
                policy.insert(name.clone(), serde_json::json!({ "a": f.a, "b": f.b, "yes": r.fitted.yes, "no": r.fitted.no, "passed_v1": v.passed }));
                cal_all.push(calibrated(&f, c));
                test_all.push(calibrated(&f, t));
                if let Some(x) = st { stress_all.push(x); }
                p95_sum = p95_sum.zip(lat).map(|(a, b)| a + b);
            }
            if cal_all.len() > 1 {
                let r = fit_report("mean_of_calibrated", &mean_ensemble(&cal_all), &mean_ensemble(&test_all), target, min_acted, None, false);
                let st_sel = (stress_all.len() == cal_all.len()).then(|| selective(&mean_ensemble(&stress_all), r.fitted));
                // Members run one after another, so the ensemble's latency is bounded by the sum of their p95s.
                let v = check("mean_of_calibrated", &V1, &Measured { test: &r.test_fitted, test_ece: r.test_ece_calibrated, stress: st_sel.as_ref(), p95_ms: p95_sum });
                any_passed |= v.passed;
                println!("{}", serde_json::json!({ "report": r, "stress": st_sel, "test_p95_ms_sum": p95_sum, "standard_v1": v }));
            }
            if let Some(path) = opt("--write-policy") {
                let doc = serde_json::json!({ "standard": "v1", "target": target, "judges": policy });
                std::fs::write(&path, serde_json::to_string_pretty(&doc).unwrap()).unwrap_or_else(|e| die(format!("{path}: {e}")));
            }
            if !any_passed { std::process::exit(1); }
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
