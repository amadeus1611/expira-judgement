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
}
