//! Shared types for EXPIRA Judgement: a claim, the evidence offered for it, and the verdict.

use serde::{Deserialize, Serialize};

/// One evidence row: where a fact came from, when, and the exact words relied on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    pub source: String,
    /// ISO date (YYYY-MM-DD) the source was opened.
    pub date: String,
    /// Word-for-word quote from the source.
    pub quote: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claim {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
}

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
    #[must_use]
    pub fn passes(self) -> bool {
        matches!(self, Verdict::Backed | Verdict::Supported)
    }
}

/// Which layer reached the verdict; cheaper layers run first.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    Deterministic,
    LocalJudge,
    Claude,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Finding {
    pub claim_id: String,
    pub verdict: Verdict,
    pub layer: Layer,
    pub confidence: f32,
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
