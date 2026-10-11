//! The self-improvement loop, kept in the repo (no model training).
//! Every ruling is logged as an observation. A candidate habit (a rule such as "auto-accept this pattern at layer 1")
//! is adopted only after enough observations agree with no contradiction, and retired the moment one contradicts it.
//! Escalated Claude rulings are the teacher: each becomes a labelled pair for the local judge's eval set.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Candidate,
    Adopted,
    Retired,
}

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
pub struct Adoption {
    pub min_agree: u32,
}
impl Default for Adoption {
    fn default() -> Self {
        Adoption { min_agree: 20 }
    }
}

impl Habit {
    #[must_use]
    pub fn new(id: &str, rule: &str) -> Self {
        Habit {
            id: id.into(),
            rule: rule.into(),
            agree: 0,
            disagree: 0,
            state: State::Candidate,
            retired_because: None,
        }
    }

    /// Record whether the habit's prediction matched the authoritative ruling.
    pub fn observe(&mut self, matched: bool, note: &str, policy: Adoption) {
        if self.state == State::Retired {
            return;
        }
        if matched {
            self.agree += 1;
        } else {
            self.disagree += 1;
        }
        if !matched {
            self.state = State::Retired; // one counter-example is enough: bad habits are never kept
            self.retired_because = Some(note.into());
        } else if self.state == State::Candidate && self.agree >= policy.min_agree {
            self.state = State::Adopted;
        }
    }
}

/// The habits that may appear in a self-report or recommendation: adopted ones only. Candidates are unproven and
/// retired ones are bad habits; neither is ever recommended (owner decision 2026-10-11, docs/HABITS.md).
pub fn recommendable(habits: &[Habit]) -> impl Iterator<Item = &Habit> {
    habits.iter().filter(|h| h.state == State::Adopted)
}

/// A teacher pair from an escalation: the claim, the quote and Claude's ruling with its reason.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TeacherPair {
    pub claim: String,
    pub quote: String,
    pub ruling: String,
    pub reason: String,
}

/// Token usage of one paid model call, as the API reported it, plus wall-clock time. One line per call in the usage log.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Usage {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub ms: f64,
    /// What the call was for (e.g. "escalation"), so cost can be split by route.
    #[serde(default)]
    pub route: String,
}

impl Usage {
    /// Reads the API's `usage` object. None if it is missing.
    #[must_use]
    pub fn from_response(
        model: &str,
        route: &str,
        ms: f64,
        resp: &serde_json::Value,
    ) -> Option<Usage> {
        let u = resp.get("usage")?;
        let n = |k: &str| u[k].as_u64().unwrap_or(0);
        Some(Usage {
            model: resp["model"].as_str().unwrap_or(model).into(),
            input_tokens: n("input_tokens"),
            output_tokens: n("output_tokens"),
            cache_read_input_tokens: n("cache_read_input_tokens"),
            cache_creation_input_tokens: n("cache_creation_input_tokens"),
            ms,
            route: route.into(),
        })
    }
}

/// USD per million tokens for one model. Loaded from a price file that names its source and date; never hard-coded.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Price {
    pub input: f64,
    pub output: f64,
    #[serde(default)]
    pub cache_read: Option<f64>,
    #[serde(default)]
    pub cache_write_5m: Option<f64>,
}

/// Cost of one call in USD. None if the model has no price, or the call used a cache rate the price file lacks:
/// an unknown cost is reported as unknown, not as zero.
#[must_use]
pub fn cost(u: &Usage, prices: &std::collections::BTreeMap<String, Price>) -> Option<f64> {
    let p = prices.get(&u.model)?;
    let per = |n: u64, rate: Option<f64>| {
        if n == 0 {
            Some(0.0)
        } else {
            rate.map(|r| n as f64 * r / 1e6)
        }
    };
    Some(
        per(u.input_tokens, Some(p.input))?
            + per(u.output_tokens, Some(p.output))?
            + per(u.cache_read_input_tokens, p.cache_read)?
            + per(u.cache_creation_input_tokens, p.cache_write_5m)?,
    )
}

/// Appends any serialisable records to a JSON-lines file, creating it if needed.
///
/// # Errors
/// The file cannot be opened or written, or a record cannot be serialised.
pub fn append_jsonl<T: Serialize>(path: &std::path::Path, rows: &[T]) -> std::io::Result<()> {
    use std::io::Write;
    if rows.is_empty() {
        return Ok(());
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    for r in rows {
        writeln!(
            f,
            "{}",
            serde_json::to_string(r).map_err(std::io::Error::other)?
        )?;
    }
    Ok(())
}

/// Appends pairs to a JSON-lines file (one pair per line), creating it if needed. Never rewrites earlier lines.
///
/// # Errors
/// The file cannot be opened or written.
pub fn append_pairs(path: &std::path::Path, pairs: &[TeacherPair]) -> std::io::Result<()> {
    use std::io::Write;
    if pairs.is_empty() {
        return Ok(());
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    for p in pairs {
        writeln!(
            f,
            "{}",
            serde_json::to_string(p).map_err(std::io::Error::other)?
        )?;
    }
    Ok(())
}

/// Reads every pair from a JSON-lines file; a missing file is an empty set.
///
/// # Errors
/// The file exists but cannot be read, or a line is not a valid pair.
pub fn load_pairs(path: &std::path::Path) -> std::io::Result<Vec<TeacherPair>> {
    match std::fs::read_to_string(path) {
        Ok(s) => s
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).map_err(std::io::Error::other))
            .collect(),
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
        for _ in 0..3 {
            h.observe(true, "", Adoption { min_agree: 3 });
        }
        assert_eq!(h.state, State::Adopted);
    }
    #[test]
    fn stays_candidate_until_enough_agreement() {
        let mut h = Habit::new("h", "r");
        let p = Adoption { min_agree: 3 };
        h.observe(true, "", p);
        h.observe(true, "", p);
        assert_eq!((h.state, h.agree), (State::Candidate, 2));
        h.observe(true, "", p);
        assert_eq!((h.state, h.agree), (State::Adopted, 3));
    }

    #[test]
    fn every_token_kind_is_priced() {
        let u = Usage {
            model: "m".into(),
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
            cache_read_input_tokens: 1_000_000,
            cache_creation_input_tokens: 1_000_000,
            ms: 0.0,
            route: String::new(),
        };
        let mut prices = std::collections::BTreeMap::new();
        prices.insert(
            "m".to_string(),
            Price {
                input: 1.0,
                output: 2.0,
                cache_read: Some(3.0),
                cache_write_5m: Some(5.0),
            },
        );
        assert_eq!(cost(&u, &prices), Some(11.0));
    }

    #[test]
    fn jsonl_round_trip_and_read_errors() {
        let dir = std::env::temp_dir().join(format!("ej-jsonl-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("u.jsonl");
        let p = TeacherPair {
            claim: "c".into(),
            quote: "q".into(),
            ruling: "unsure".into(),
            reason: "r".into(),
        };
        append_jsonl(&path, &[p.clone(), p.clone()]).unwrap();
        assert_eq!(load_pairs(&path).unwrap(), vec![p.clone(), p]);
        // A directory is not a missing file: reading it is an error, not an empty set.
        assert!(load_pairs(&dir).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn only_adopted_habits_are_recommended() {
        let mut adopted = Habit::new("good", "fit thresholds");
        for _ in 0..3 {
            adopted.observe(true, "", Adoption { min_agree: 3 });
        }
        let candidate = Habit::new("maybe", "unproven");
        let mut retired = Habit::new("bad", "raw bar");
        retired.observe(false, "passed unsupported claims", Adoption::default());
        let all = [candidate, retired, adopted];
        let ids: Vec<&str> = recommendable(&all).map(|h| h.id.as_str()).collect();
        assert_eq!(ids, vec!["good"]);
        assert_eq!(recommendable(&all[..2]).count(), 0);
    }

    #[test]
    fn one_contradiction_retires() {
        let mut h = Habit::new("h2", "overlap >= 0.9 is support");
        h.observe(true, "", Adoption::default());
        h.observe(false, "negation flipped meaning", Adoption::default());
        assert_eq!((h.state, h.disagree), (State::Retired, 1));
        h.observe(true, "", Adoption::default());
        assert_eq!(h.agree, 1);
    }
    #[test]
    fn usage_cost_is_known_or_none() {
        let resp = serde_json::json!({ "model": "m", "usage": { "input_tokens": 2_000_000, "output_tokens": 100_000, "cache_read_input_tokens": 0 } });
        let u = Usage::from_response("fallback", "escalation", 12.0, &resp).unwrap();
        let mut prices = std::collections::BTreeMap::new();
        prices.insert(
            "m".to_string(),
            Price {
                input: 1.0,
                output: 10.0,
                cache_read: None,
                cache_write_5m: None,
            },
        );
        assert_eq!(cost(&u, &prices), Some(2.0 + 1.0));
        let cached = Usage {
            cache_read_input_tokens: 5,
            ..u.clone()
        };
        assert_eq!(cost(&cached, &prices), None); // a cache rate is needed but missing
        assert_eq!(
            cost(
                &Usage {
                    model: "other".into(),
                    ..u
                },
                &prices
            ),
            None
        );
    }

    #[test]
    fn teacher_pairs_append_and_reload() {
        let path = std::env::temp_dir().join(format!("ej-pairs-{}.jsonl", std::process::id()));
        let _ = std::fs::remove_file(&path);
        assert!(load_pairs(&path).unwrap().is_empty());
        let p = |r: &str| TeacherPair {
            claim: "c".into(),
            quote: "q".into(),
            ruling: r.into(),
            reason: "why".into(),
        };
        append_pairs(&path, &[p("supports")]).unwrap();
        append_pairs(&path, &[p("contradicts")]).unwrap();
        assert_eq!(
            load_pairs(&path).unwrap(),
            vec![p("supports"), p("contradicts")]
        );
        std::fs::remove_file(&path).unwrap();
    }
}
