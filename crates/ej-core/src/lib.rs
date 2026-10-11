//! Shared types for EXPIRA Judgement: a claim, the evidence offered for it, and the verdict.

use serde::{Deserialize, Serialize};

/// One evidence row: where a fact came from, when, and the exact words relied on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    /// Where the fact came from (URL, document name or ledger reference).
    pub source: String,
    /// ISO date (YYYY-MM-DD) the source was opened.
    pub date: String,
    /// Word-for-word quote from the source.
    pub quote: String,
}

/// A statement to check, with the evidence offered for it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claim {
    /// Caller-chosen identifier, echoed in the [`Finding`].
    pub id: String,
    /// The statement as written in the deliverable.
    pub text: String,
    /// Evidence rows offered for the claim (empty when none).
    #[serde(default)]
    pub evidence: Vec<Evidence>,
}

/// The outcome of checking one claim.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Proven by a deterministic check.
    Backed,
    /// A judge found the evidence supports the claim (paraphrase).
    Supported,
    /// No usable evidence.
    Unbacked,
    /// Evidence exists but states different figures or units.
    Mismatch,
    /// Evidence older than the allowed age.
    Stale,
    /// Nobody could decide: escalate or leave the claim out.
    Unsure,
}

impl Verdict {
    /// True only for verdicts that let a claim be stated: [`Verdict::Backed`] and [`Verdict::Supported`].
    #[must_use]
    pub fn passes(self) -> bool {
        matches!(self, Verdict::Backed | Verdict::Supported)
    }
}

/// Which layer reached the verdict; cheaper layers run first.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    /// Layer 1: deterministic checks on the quoted evidence.
    Deterministic,
    /// Layer 2: a local model judge.
    LocalJudge,
    /// Layer 3: escalation to Claude.
    Claude,
}

/// The result for one claim: verdict, the layer that reached it, and why.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Finding {
    /// The [`Claim::id`] this finding is for.
    pub claim_id: String,
    /// The verdict reached.
    pub verdict: Verdict,
    /// The layer that reached it.
    pub layer: Layer,
    /// Confidence in the verdict, 0.0 to 1.0.
    pub confidence: f32,
    /// Human-readable explanation of the verdict.
    pub reason: String,
}

/// Words that flip a statement's polarity. Shared so every layer reads negation the same way.
pub const NEGATORS: [&str; 8] = [
    "no", "not", "never", "without", "none", "neither", "nor", "cannot",
];

/// True if a lower-cased, punctuation-trimmed word negates ("not", "don't", "isn’t", ...).
#[must_use]
pub fn is_negator(w: &str) -> bool {
    NEGATORS.contains(&w) || w.ends_with("n't") || w.ends_with("n’t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_backed_and_supported_pass() {
        for v in [Verdict::Backed, Verdict::Supported] {
            assert!(v.passes(), "{v:?}");
        }
        for v in [
            Verdict::Unbacked,
            Verdict::Mismatch,
            Verdict::Stale,
            Verdict::Unsure,
        ] {
            assert!(!v.passes(), "{v:?}");
        }
    }

    #[test]
    fn negators() {
        for w in [
            "not", "no", "never", "without", "none", "neither", "nor", "cannot", "isn't", "don't",
            "wasn’t",
        ] {
            assert!(is_negator(w), "{w}");
        }
        for w in ["note", "know", "nothing-like", "on", "isn", "tn't"] {
            assert_eq!(is_negator(w), w.ends_with("n't"), "{w}");
        }
        assert!(!is_negator("notice"));
    }
}
