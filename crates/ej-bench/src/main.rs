//! ej-bench <cases.jsonl> [--bar 0.8] [--overlap] [--nli `MODEL_DIR`] [--cmd PROGRAM ARGS...]
//!     Scores each requested judge on the same cases and prints one JSON report per judge.
//! ej-bench score <cases.jsonl> --out SCORES.jsonl [--nli `MODEL_DIR`] [--name NAME] [--cmd PROGRAM ARGS...]
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

fn die(e: &str) -> ! {
    eprintln!("{e}");
    std::process::exit(2)
}
fn usage() -> ! {
    die("usage: ej-bench <cases.jsonl> [--bar 0.8] [--overlap] [--nli DIR] [--cmd ...] | score <cases.jsonl> --out FILE ... | fit --cal FILE --test FILE | words <cases.jsonl>")
}

fn judges(
    head: &[String],
    tail: Option<&[String]>,
    opt: &dyn Fn(&str) -> Option<String>,
) -> Vec<(String, Box<dyn Judge>)> {
    let mut judges: Vec<(String, Box<dyn Judge>)> = Vec::new();
    if head.iter().any(|a| a == "--overlap") {
        judges.push(("overlap".into(), Box::new(OverlapJudge)));
    }
    if let Some(dir) = opt("--nli") {
        let j = ej_nli::NliJudge::load(std::path::Path::new(&dir))
            .unwrap_or_else(|e| die(&format!("--nli: {e}")));
        judges.push(("nli".into(), Box::new(j)));
    }
    if let Some(argv) = tail {
        let name = opt("--name").unwrap_or_else(|| "cmd".into());
        judges.push((
            name,
            Box::new(CommandJudge::spawn(argv).unwrap_or_else(|e| die(&format!("--cmd: {e}")))),
        ));
    }
    judges
}

/// Per judge: scores in case order, and per-case latency in ms when the file carries it.
fn read_scores(path: &str) -> BTreeMap<String, (Vec<Scored>, Option<Vec<f64>>)> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| die(&format!("{path}: {e}")));
    let mut rows: BTreeMap<String, Vec<(u64, Scored, Option<f64>)>> = BTreeMap::new();
    for l in text.lines().filter(|l| !l.trim().is_empty()) {
        let v: serde_json::Value =
            serde_json::from_str(l).unwrap_or_else(|e| die(&format!("{path}: {e}")));
        let (Some(j), Some(i), Some(st), Some(p)) = (
            v["judge"].as_str(),
            v["i"].as_u64(),
            v["stated"].as_bool(),
            v["p"].as_f64(),
        ) else {
            die(&format!("{path}: bad row {l}"))
        };
        rows.entry(j.into())
            .or_default()
            .push((i, Scored { p, stated: st }, v["ms"].as_f64()));
    }
    rows.into_iter()
        .map(|(k, mut v)| {
            v.sort_by_key(|r| r.0);
            let ms = v.iter().map(|r| r.2).collect::<Option<Vec<f64>>>();
            (k, (v.iter().map(|r| r.1).collect(), ms))
        })
        .collect()
}

fn p95(ms: &[f64]) -> f64 {
    let mut v = ms.to_vec();
    v.sort_by(f64::total_cmp);
    v[ej_bench::percentile_index(v.len(), 0.95)]
}

/// The command line split at `--cmd`: options before it, the judge command after it.
struct Cli<'a> {
    head: &'a [String],
    tail: Option<&'a [String]>,
}

impl Cli<'_> {
    fn opt(&self, k: &str) -> Option<String> {
        self.head
            .iter()
            .position(|a| a == k)
            .and_then(|i| self.head.get(i + 1))
            .cloned()
    }
    fn parsed<T: std::str::FromStr>(&self, k: &str, default: T) -> T {
        self.opt(k).map_or(default, |s| {
            s.parse().unwrap_or_else(|_| die(&format!("bad {k}")))
        })
    }
}

fn read_cases(path: &str) -> Vec<ej_bench::Case> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| die(&format!("{path}: {e}")));
    load_cases(&text).unwrap_or_else(|e| die(&format!("{path}: {e}")))
}

fn bench_claim(i: usize, c: &ej_bench::Case) -> Claim {
    Claim {
        id: i.to_string(),
        text: c.claim.clone(),
        evidence: vec![Evidence {
            source: "bench".into(),
            date: "2000-01-01".into(),
            quote: c.quote.clone(),
        }],
    }
}

/// `score`: raw P(stated) and latency per case and judge, one JSON line each.
fn cmd_score(cli: &Cli, path: &str) {
    let out = cli.opt("--out").unwrap_or_else(|| usage());
    let cases = read_cases(path);
    let js = judges(cli.head, cli.tail, &|k| cli.opt(k));
    let mut f = std::fs::File::create(&out).unwrap_or_else(|e| die(&format!("{out}: {e}")));
    for (name, j) in &js {
        for (i, c) in cases.iter().enumerate() {
            let t = std::time::Instant::now();
            let Some(p) = j.p_stated(&bench_claim(i, c)) else {
                die(&format!("judge {name} gave no score on case {i}"))
            };
            let ms = t.elapsed().as_secs_f64() * 1e3;
            let row = serde_json::json!({ "judge": name, "i": i, "stated": c.label == "supports", "p": p, "ms": ms });
            writeln!(f, "{row}").unwrap_or_else(|e| die(&format!("{out}: {e}")));
        }
    }
}

/// `--current JUDGE=YES,NO` options: the raw thresholds a judge uses today, for comparison.
fn current_bars(cli: &Cli) -> BTreeMap<String, Thresholds> {
    let mut current = BTreeMap::new();
    for (i, a) in cli.head.iter().enumerate() {
        if a != "--current" {
            continue;
        }
        let spec = cli.head.get(i + 1).unwrap_or_else(|| usage());
        let (j, t) = spec.split_once('=').unwrap_or_else(|| usage());
        let (y, n) = t.split_once(',').unwrap_or_else(|| usage());
        let bar = |x: &str| x.parse().unwrap_or_else(|_| usage());
        current.insert(
            j.to_string(),
            Thresholds {
                yes: bar(y),
                no: bar(n),
            },
        );
    }
    current
}

fn calibrated(f: &Platt, xs: &[Scored]) -> Vec<Scored> {
    xs.iter()
        .map(|s| Scored {
            p: f.apply(s.p),
            stated: s.stated,
        })
        .collect()
}

/// Calibrated scores of every judge, kept to build the ensemble after the per-judge fits.
#[derive(Default)]
struct Pool {
    cal: Vec<Vec<Scored>>,
    test: Vec<Vec<Scored>>,
    stress: Vec<Vec<Scored>>,
    p95_sum: Option<f64>,
}

/// `fit`: per-judge calibration and thresholds on --cal, verdicts on --test and --stress, the ensemble, the policy file.
fn cmd_fit(cli: &Cli) {
    let cal = read_scores(&cli.opt("--cal").unwrap_or_else(|| usage()));
    let test = read_scores(&cli.opt("--test").unwrap_or_else(|| usage()));
    let stress = cli.opt("--stress").map(|p| read_scores(&p));
    let target: f64 = cli.parsed("--target", V1.pass_lower_bound);
    let min_acted: usize = cli.parsed("--min-acted", 20);
    let current = current_bars(cli);
    let mut pool = Pool {
        p95_sum: Some(0.0),
        ..Pool::default()
    };
    let (mut policy, mut any_passed) = (serde_json::Map::new(), false);
    for (name, (c, _)) in &cal {
        let Some((t, ms)) = test.get(name) else {
            continue;
        };
        let r = fit_report(
            name,
            c,
            t,
            target,
            min_acted,
            current.get(name).copied(),
            true,
        );
        let f = r.platt.unwrap_or_else(|| die("calibration missing"));
        let st = stress
            .as_ref()
            .and_then(|s| s.get(name))
            .map(|(x, _)| calibrated(&f, x));
        let st_sel = st.as_ref().map(|x| selective(x, r.fitted));
        let lat = ms.as_ref().map(|m| p95(m));
        let v = check(
            name,
            &V1,
            &Measured {
                test: &r.test_fitted,
                test_ece: r.test_ece_calibrated,
                stress: st_sel.as_ref(),
                p95_ms: lat,
            },
        );
        any_passed |= v.passed;
        println!(
            "{}",
            serde_json::json!({ "report": r, "stress": st_sel, "test_p95_ms": lat, "standard_v1": v })
        );
        // A side that never acts has an infinite threshold; JSON has no infinity, so it is written out of range (2 / -1).
        policy.insert(name.clone(), serde_json::json!({ "a": f.a, "b": f.b, "yes": r.fitted.yes.min(2.0), "no": r.fitted.no.max(-1.0), "passed_v1": v.passed }));
        pool.cal.push(calibrated(&f, c));
        pool.test.push(calibrated(&f, t));
        if let Some(x) = st {
            pool.stress.push(x);
        }
        pool.p95_sum = pool.p95_sum.zip(lat).map(|(a, b)| a + b);
    }
    if pool.cal.len() > 1 {
        any_passed |= fit_ensemble(&pool, target, min_acted);
    }
    if let Some(path) = cli.opt("--write-policy") {
        let doc = serde_json::json!({ "standard": "v1", "target": target, "judges": policy });
        let text =
            serde_json::to_string_pretty(&doc).unwrap_or_else(|e| die(&format!("{path}: {e}")));
        std::fs::write(&path, text).unwrap_or_else(|e| die(&format!("{path}: {e}")));
    }
    if !any_passed {
        std::process::exit(1);
    }
}

/// The mean of all judges' calibrated scores, fitted and checked like a single judge. Returns whether it passed.
fn fit_ensemble(pool: &Pool, target: f64, min_acted: usize) -> bool {
    let r = fit_report(
        "mean_of_calibrated",
        &mean_ensemble(&pool.cal),
        &mean_ensemble(&pool.test),
        target,
        min_acted,
        None,
        false,
    );
    let st_sel = (pool.stress.len() == pool.cal.len())
        .then(|| selective(&mean_ensemble(&pool.stress), r.fitted));
    // Members run one after another, so the ensemble's latency is bounded by the sum of their p95s.
    let v = check(
        "mean_of_calibrated",
        &V1,
        &Measured {
            test: &r.test_fitted,
            test_ece: r.test_ece_calibrated,
            stress: st_sel.as_ref(),
            p95_ms: pool.p95_sum,
        },
    );
    println!(
        "{}",
        serde_json::json!({ "report": r, "stress": st_sel, "test_p95_ms_sum": pool.p95_sum, "standard_v1": v })
    );
    v.passed
}

/// Default mode: act/escalate report per judge at a fixed bar.
fn cmd_run(cli: &Cli, path: &str) {
    let cases = read_cases(path);
    let bars = Bars {
        act: cli.parsed("--bar", Bars::default().act),
    };
    let js = judges(cli.head, cli.tail, &|k| cli.opt(k));
    if js.is_empty() {
        die("no judge selected (--overlap, --nli, --cmd)");
    }
    for (name, j) in &js {
        let report = serde_json::to_string(&run(name, j.as_ref(), &cases, bars))
            .unwrap_or_else(|e| die(&format!("{e}")));
        println!("{report}");
    }
}

/// One line per case: how many claim words no quote has, and whether every name is in a quote (the wording gate a
/// local "stated" must pass in `adjudicate`), so offline cascade measurements apply exactly the production rule.
fn cmd_words(path: &str) {
    for (i, c) in read_cases(path).iter().enumerate() {
        let quotes = [c.quote.as_str()];
        println!(
            "{}",
            serde_json::json!({
                "i": i,
                "unstated": ej_evidence::unstated_words(&c.claim, &quotes),
                "names_ok": ej_evidence::names_in_quotes(&c.claim, &quotes),
                "figures_ok": ej_evidence::figures_backed(&c.claim, &quotes),
            })
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd_at = args.iter().position(|a| a == "--cmd");
    let cli = Cli {
        head: &args[..cmd_at.unwrap_or(args.len())],
        tail: cmd_at.map(|i| &args[i + 1..]),
    };
    match args.get(1).map(String::as_str) {
        Some("score") => cmd_score(&cli, args.get(2).unwrap_or_else(|| usage())),
        Some("fit") => cmd_fit(&cli),
        Some("words") => cmd_words(args.get(2).unwrap_or_else(|| usage())),
        Some(path) if !path.starts_with("--") => cmd_run(&cli, path),
        _ => usage(),
    }
}
