//! CLI for checking study notes against a supplied source using the existing `ej check` binary.

use ej_core::Evidence;
use ej_study::{check_study, Engine, LocalModel, Segmentation};
use std::collections::BTreeMap;
use std::path::PathBuf;

const USAGE: &str = "usage: ej-study --text NOTES.txt --source SOURCE.txt --source-date YYYY-MM-DD --today YYYY-MM-DD [--source-id URL_OR_NAME] [--lines] [--ej PATH] [--max-age DAYS] [--nli DIR --policy POLICY.json]\nPrints a JSON report. Exit 0: all pass; 1: review flagged claims; 2: input/setup/protocol error.\n--source-date is the actual date the source was opened. --help prints this message.";

fn run(args: &[String]) -> Result<bool, String> {
    let mut opts = BTreeMap::new();
    let mut mode = Segmentation::Sentences;
    let mut lines = false;
    let mut iter = args.iter();
    while let Some(key) = iter.next() {
        if key == "--lines" && !lines {
            mode = Segmentation::Lines;
            lines = true;
            continue;
        }
        if !matches!(
            key.as_str(),
            "--text"
                | "--source"
                | "--source-date"
                | "--today"
                | "--source-id"
                | "--ej"
                | "--max-age"
                | "--nli"
                | "--policy"
        ) {
            return Err(format!("unknown or repeated option: {key}"));
        }
        let value = iter
            .next()
            .filter(|v| !v.starts_with("--"))
            .ok_or_else(|| format!("{key} needs a value"))?;
        if opts.insert(key.as_str(), value.as_str()).is_some() {
            return Err(format!("repeated option: {key}"));
        }
    }
    let required = |key| {
        opts.get(key)
            .copied()
            .ok_or_else(|| format!("missing {key}"))
    };
    let notes_path = required("--text")?;
    let source_path = required("--source")?;
    let source = Evidence {
        source: opts
            .get("--source-id")
            .copied()
            .unwrap_or(source_path)
            .into(),
        date: required("--source-date")?.into(),
        quote: std::fs::read_to_string(source_path)
            .map_err(|e| format!("source {source_path}: {e}"))?,
    };
    let local_model = match (opts.get("--nli"), opts.get("--policy")) {
        (Some(dir), Some(policy)) => Some(LocalModel {
            directory: PathBuf::from(dir),
            policy: PathBuf::from(policy),
        }),
        (None, None) => None,
        _ => return Err("--nli and --policy must be supplied together".into()),
    };
    let engine = Engine {
        program: opts.get("--ej").copied().unwrap_or("ej").into(),
        today: required("--today")?.into(),
        max_age_days: opts
            .get("--max-age")
            .map(|v| v.parse::<u32>().map_err(|e| format!("--max-age: {e}")))
            .transpose()?,
        local_model,
    };
    let notes =
        std::fs::read_to_string(notes_path).map_err(|e| format!("notes {notes_path}: {e}"))?;
    let report = check_study(&notes, &source, mode, &engine)?;
    let json = serde_json::to_string_pretty(&report).map_err(|e| format!("report: {e}"))?;
    println!("{json}");
    Ok(report.has_flags())
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--help"] {
        println!("{USAGE}");
        return;
    }
    match run(&args) {
        Ok(flagged) => std::process::exit(i32::from(flagged)),
        Err(e) => {
            eprintln!("{e}\n{USAGE}");
            std::process::exit(2);
        }
    }
}
