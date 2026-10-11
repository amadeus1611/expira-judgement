//! ej-bench <cases.jsonl> [--bar 0.8] [--overlap] [--nli MODEL_DIR] [--cmd PROGRAM ARGS...]
//! Scores each requested judge on the same cases and prints one JSON report per judge.
//! `--cmd` must come last: everything after it is the judge command (see command.rs for the protocol).

use ej_bench::{command::CommandJudge, load_cases, run};
use ej_judge::{Bars, Judge, OverlapJudge};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let Some(path) = args.get(1).filter(|a| !a.starts_with("--")) else {
        eprintln!("usage: ej-bench <cases.jsonl> [--bar 0.8] [--overlap] [--nli MODEL_DIR] [--cmd PROGRAM ARGS...]");
        std::process::exit(2);
    };
    let cmd_at = args.iter().position(|a| a == "--cmd");
    let head = &args[..cmd_at.unwrap_or(args.len())];
    let opt = |k: &str| head.iter().position(|a| a == k).and_then(|i| head.get(i + 1)).cloned();
    let die = |e: String| -> ! { eprintln!("{e}"); std::process::exit(2) };

    let cases = load_cases(&std::fs::read_to_string(path).unwrap_or_else(|e| die(format!("{path}: {e}")))).unwrap_or_else(|e| die(format!("{path}: {e}")));
    let bars = Bars { act: opt("--bar").map(|s| s.parse().unwrap_or_else(|_| die("bad --bar".into()))).unwrap_or(Bars::default().act) };

    let mut judges: Vec<(String, Box<dyn Judge>)> = Vec::new();
    if head.iter().any(|a| a == "--overlap") { judges.push(("overlap".into(), Box::new(OverlapJudge))); }
    if let Some(dir) = opt("--nli") {
        let j = ej_nli::NliJudge::load(std::path::Path::new(&dir)).unwrap_or_else(|e| die(format!("--nli: {e}")));
        judges.push((format!("nli:{dir}"), Box::new(j)));
    }
    if let Some(i) = cmd_at {
        let argv = &args[i + 1..];
        judges.push((format!("cmd:{}", argv.join(" ")), Box::new(CommandJudge::spawn(argv).unwrap_or_else(|e| die(format!("--cmd: {e}"))))));
    }
    if judges.is_empty() { die("no judge selected (--overlap, --nli, --cmd)".into()); }
    for (name, j) in &judges {
        println!("{}", serde_json::to_string(&run(name, j.as_ref(), &cases, bars)).unwrap());
    }
}
