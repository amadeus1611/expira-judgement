//! The self-improvement loop, kept in the repo (no model training).
//! Every ruling is logged as an observation. A candidate habit (a rule such as "auto-accept this pattern at layer 1")
//! is adopted only after enough observations agree with no contradiction, and retired the moment one contradicts it.
//! Escalated Claude rulings are the teacher: each becomes a labelled pair for the local judge's eval set.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum State { Candidate, Adopted, Retired }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Habit {
    pub id: String,
    pub rule: String,
    pub agree: u32,
    pub disagree: u32,
    pub state: State,
    #[serde(default)]
    pub retired_because: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct Adoption { pub min_agree: u32 }
impl Default for Adoption { fn default() -> Self { Adoption { min_agree: 20 } } }

impl Habit {
    pub fn new(id: &str, rule: &str) -> Self { Habit { id: id.into(), rule: rule.into(), agree: 0, disagree: 0, state: State::Candidate, retired_because: None } }

    /// Record whether the habit's prediction matched the authoritative ruling.
    pub fn observe(&mut self, matched: bool, note: &str, policy: Adoption) {
        if self.state == State::Retired { return; }
        if matched { self.agree += 1 } else { self.disagree += 1 }
        if !matched {
            self.state = State::Retired; // one counter-example is enough: bad habits are never kept
            self.retired_because = Some(note.into());
        } else if self.state == State::Candidate && self.agree >= policy.min_agree {
            self.state = State::Adopted;
        }
    }
}

/// A teacher pair from an escalation: the claim, the quote and Claude's ruling with its reason.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TeacherPair { pub claim: String, pub quote: String, pub ruling: String, pub reason: String }

/// Appends pairs to a JSON-lines file (one pair per line), creating it if needed. Never rewrites earlier lines.
pub fn append_pairs(path: &std::path::Path, pairs: &[TeacherPair]) -> std::io::Result<()> {
    use std::io::Write;
    if pairs.is_empty() { return Ok(()); }
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    for p in pairs { writeln!(f, "{}", serde_json::to_string(p).map_err(std::io::Error::other)?)?; }
    Ok(())
}

/// Reads every pair from a JSON-lines file; a missing file is an empty set.
pub fn load_pairs(path: &std::path::Path) -> std::io::Result<Vec<TeacherPair>> {
    match std::fs::read_to_string(path) {
        Ok(s) => s.lines().filter(|l| !l.trim().is_empty()).map(|l| serde_json::from_str(l).map_err(std::io::Error::other)).collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adopts_after_enough_agreement() {
        let mut h = Habit::new("h1", "exact quote with matching figures is backed");
        for _ in 0..3 { h.observe(true, "", Adoption { min_agree: 3 }); }
        assert_eq!(h.state, State::Adopted);
    }
    #[test]
    fn one_contradiction_retires() {
        let mut h = Habit::new("h2", "overlap >= 0.9 is support");
        h.observe(true, "", Adoption::default());
        h.observe(false, "negation flipped meaning", Adoption::default());
        assert_eq!(h.state, State::Retired);
        h.observe(true, "", Adoption::default());
        assert_eq!(h.agree, 1);
    }
    #[test]
    fn teacher_pairs_append_and_reload() {
        let path = std::env::temp_dir().join(format!("ej-pairs-{}.jsonl", std::process::id()));
        let _ = std::fs::remove_file(&path);
        assert!(load_pairs(&path).unwrap().is_empty());
        let p = |r: &str| TeacherPair { claim: "c".into(), quote: "q".into(), ruling: r.into(), reason: "why".into() };
        append_pairs(&path, &[p("supports")]).unwrap();
        append_pairs(&path, &[p("contradicts")]).unwrap();
        assert_eq!(load_pairs(&path).unwrap(), vec![p("supports"), p("contradicts")]);
        std::fs::remove_file(&path).unwrap();
    }
}
