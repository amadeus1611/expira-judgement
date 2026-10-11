//! Agent delegation: route each task to the cheapest tier that can pass its gate, and escalate one tier at a time.
//! Mirrors the roster: scout/worker (cheap model) -> builder (mid) -> reviewer (top). Model IDs are config, not code.

pub mod cost;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Scout,
    Worker,
    Builder,
    Reviewer,
}

impl Tier {
    pub fn next(self) -> Option<Tier> {
        match self {
            Tier::Scout => Some(Tier::Worker),
            Tier::Worker => Some(Tier::Builder),
            Tier::Builder => Some(Tier::Reviewer),
            Tier::Reviewer => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub kind: String,
    /// Money or risk at stake in the task's currency; above `high_stakes` it starts at Reviewer.
    pub stakes: f64,
    pub token_budget: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roster {
    /// Model ID per tier, e.g. loaded from roster.json.
    pub models: std::collections::BTreeMap<String, String>,
    pub high_stakes: f64,
}

/// The only models any tier may use (owner decision 2026-10-11: Claude only, current generation).
pub const ALLOWED_MODELS: [&str; 3] = ["claude-haiku-5-5", "claude-sonnet-5-5", "claude-opus-5-5"];

impl Roster {
    /// Loads a roster and refuses any model outside `ALLOWED_MODELS` or any tier without a model.
    pub fn load(json: &str) -> Result<Roster, String> {
        let r: Roster = serde_json::from_str(json).map_err(|e| e.to_string())?;
        for tier in ["scout", "worker", "builder", "reviewer"] {
            let m = r
                .models
                .get(tier)
                .ok_or(format!("roster has no model for {tier}"))?;
            if !ALLOWED_MODELS.contains(&m.as_str()) {
                return Err(format!(
                    "{tier}: {m} is not an allowed model ({})",
                    ALLOWED_MODELS.join(", ")
                ));
            }
        }
        if let Some(extra) = r
            .models
            .keys()
            .find(|k| !["scout", "worker", "builder", "reviewer"].contains(&k.as_str()))
        {
            return Err(format!("unknown tier {extra}"));
        }
        Ok(r)
    }
}

pub fn start_tier(task: &Task, roster: &Roster) -> Tier {
    if task.stakes > roster.high_stakes {
        return Tier::Reviewer;
    }
    match task.kind.as_str() {
        "lookup" | "extract" | "gate_run" => Tier::Scout,
        "ledger_rows" | "normalise" | "small_edit" => Tier::Worker,
        "synthesis" | "draft" | "build" => Tier::Builder,
        _ => Tier::Builder,
    }
}

/// A launch record, one per attempt (the tier log).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Launch {
    pub task_id: String,
    pub tier: Tier,
    pub gate_passed: bool,
}

/// Run attempts, escalating only when the gate fails. `attempt` performs the work at a tier and reports gate pass.
pub fn run(task: &Task, roster: &Roster, mut attempt: impl FnMut(Tier) -> bool) -> Vec<Launch> {
    let mut log = Vec::new();
    let mut tier = Some(start_tier(task, roster));
    while let Some(t) = tier {
        let ok = attempt(t);
        log.push(Launch {
            task_id: task.id.clone(),
            tier: t,
            gate_passed: ok,
        });
        if ok {
            break;
        }
        tier = t.next();
    }
    log
}

#[cfg(test)]
mod tests {
    use super::*;
    fn roster() -> Roster {
        Roster {
            models: Default::default(),
            high_stakes: 500_000.0,
        }
    }
    fn task(kind: &str, stakes: f64) -> Task {
        Task {
            id: "t".into(),
            kind: kind.into(),
            stakes,
            token_budget: 10_000,
        }
    }

    #[test]
    fn cheap_first_then_escalate_one_tier() {
        let log = run(&task("lookup", 0.0), &roster(), |t| t == Tier::Builder);
        assert_eq!(
            log.iter().map(|l| l.tier).collect::<Vec<_>>(),
            vec![Tier::Scout, Tier::Worker, Tier::Builder]
        );
        assert!(log.last().unwrap().gate_passed);
    }
    #[test]
    fn roster_accepts_only_claude_models() {
        let ok = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../config/roster.json"
        ))
        .unwrap();
        assert!(Roster::load(&ok).is_ok());
        let gpt = ok.replace("claude-haiku-5-5", "gpt-6.1");
        assert!(Roster::load(&gpt)
            .unwrap_err()
            .contains("not an allowed model"));
        assert!(
            Roster::load(r#"{"models":{"scout":"claude-haiku-5-5"},"high_stakes":1}"#).is_err()
        );
    }

    #[test]
    fn high_stakes_start_at_reviewer() {
        assert_eq!(
            start_tier(&task("lookup", 600_000.0), &roster()),
            Tier::Reviewer
        );
    }
    #[test]
    fn stops_after_top_tier() {
        assert_eq!(run(&task("draft", 0.0), &roster(), |_| false).len(), 2);
    }
}
