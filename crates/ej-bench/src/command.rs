//! Adapter for judges that live outside Rust (e.g. LAYA's Python service): a long-running child process that
//! reads one JSON request per line on stdin, `{"claim": "...", "quotes": ["..."]}`, and answers one line on stdout,
//! `{"ruling": "supports"|"contradicts"|"unsure", "confidence": 0..1, "reason": "..."}`.
//! The process stays up between cases so latency is per ruling, not per process start.

use ej_core::{Claim, Layer};
use ej_judge::{Judge, Ruling};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Mutex;

/// A [`Judge`] that talks to a long-running child process over the JSON-lines protocol above.
pub struct CommandJudge {
    io: Mutex<(Child, ChildStdin, BufReader<ChildStdout>)>,
}

fn ruling_of(v: &Value) -> Result<Ruling, String> {
    match v["ruling"].as_str() {
        Some("supports") => Ok(Ruling::Supports),
        Some("contradicts") => Ok(Ruling::Contradicts),
        Some("unsure") => Ok(Ruling::Unsure),
        _ => Err("reply has no valid ruling".into()),
    }
}

impl CommandJudge {
    /// `argv[0]` is the program, the rest its arguments.
    ///
    /// # Errors
    /// An empty command, or the program cannot be started.
    pub fn spawn(argv: &[String]) -> Result<Self, String> {
        let (prog, rest) = argv.split_first().ok_or("empty command")?;
        let mut child = Command::new(prog)
            .args(rest)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| format!("{prog}: {e}"))?;
        let stdin = child.stdin.take().ok_or("child has no stdin")?;
        let stdout = BufReader::new(child.stdout.take().ok_or("child has no stdout")?);
        Ok(CommandJudge {
            io: Mutex::new((child, stdin, stdout)),
        })
    }

    fn ask(&self, claim: &Claim) -> Result<Value, String> {
        let mut io = self.io.lock().unwrap();
        let req = json!({ "claim": claim.text, "quotes": claim.evidence.iter().map(|e| &e.quote).collect::<Vec<_>>() });
        writeln!(io.1, "{req}")
            .and_then(|()| io.1.flush())
            .map_err(|e| e.to_string())?;
        let mut line = String::new();
        if io.2.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
            return Err("judge process closed stdout".into());
        }
        serde_json::from_str(&line).map_err(|e| format!("bad reply: {e}"))
    }
}

impl Drop for CommandJudge {
    fn drop(&mut self) {
        if let Ok(mut io) = self.io.lock() {
            let _ = io.0.kill();
            let _ = io.0.wait();
        }
    }
}

impl Judge for CommandJudge {
    fn layer(&self) -> Layer {
        Layer::LocalJudge
    }
    fn rule(&self, claim: &Claim) -> (Ruling, f32, String) {
        let v = match self.ask(claim) {
            Ok(v) => v,
            Err(e) => return (Ruling::Unsure, 0.0, format!("command judge: {e}")),
        };
        match ruling_of(&v) {
            Ok(r) => (
                r,
                v["confidence"].as_f64().unwrap_or(0.0).clamp(0.0, 1.0) as f32,
                v["reason"].as_str().unwrap_or("").into(),
            ),
            Err(e) => (Ruling::Unsure, 0.0, format!("command judge: {e}")),
        }
    }
    /// Optional `"p"` in the reply: the judge's raw probability that the text states the claim.
    fn p_stated(&self, claim: &Claim) -> Option<f32> {
        self.ask(claim).ok()?["p"]
            .as_f64()
            .map(|p| p.clamp(0.0, 1.0) as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ej_core::Evidence;

    #[test]
    fn talks_line_protocol_with_a_child() {
        // A shell stand-in: answers "supports" while it reads lines, so two cases share one process.
        let script = "while read l; do echo '{\"ruling\":\"supports\",\"confidence\":0.9,\"reason\":\"stub\",\"p\":0.7}'; done";
        let j = CommandJudge::spawn(&["sh".into(), "-c".into(), script.into()]).unwrap();
        let c = Claim {
            id: "c".into(),
            text: "t".into(),
            evidence: vec![Evidence {
                source: "s".into(),
                date: "2026-01-01".into(),
                quote: "q".into(),
            }],
        };
        assert_eq!(j.rule(&c), (Ruling::Supports, 0.9, "stub".into()));
        assert_eq!(j.rule(&c).0, Ruling::Supports);
        assert_eq!(j.p_stated(&c), Some(0.7));
    }

    #[test]
    fn every_ruling_word_is_read() {
        for (w, r) in [
            ("supports", Ruling::Supports),
            ("contradicts", Ruling::Contradicts),
            ("unsure", Ruling::Unsure),
        ] {
            assert_eq!(ruling_of(&json!({ "ruling": w })), Ok(r));
        }
        assert!(ruling_of(&json!({ "ruling": "maybe" })).is_err());
        assert!(ruling_of(&json!({})).is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn drop_stops_the_child() {
        let j = CommandJudge::spawn(&["sleep".into(), "30".into()]).unwrap();
        let pid = j.io.lock().unwrap().0.id();
        let proc_dir = std::path::PathBuf::from(format!("/proc/{pid}"));
        assert!(proc_dir.exists());
        drop(j);
        // Killed and reaped: the process entry is gone at once.
        assert!(!proc_dir.exists());
    }

    #[test]
    fn dead_child_is_unsure() {
        let j = CommandJudge::spawn(&["true".into()]).unwrap();
        let c = Claim {
            id: "c".into(),
            text: "t".into(),
            evidence: vec![],
        };
        assert_eq!(j.rule(&c).0, Ruling::Unsure);
    }
}
