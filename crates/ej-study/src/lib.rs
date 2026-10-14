//! Study-note integration around the existing `ej check` binary.
//!
//! Segmentation creates verbatim candidates, not semantic or atomic fact extraction.
//! Judgement, model eligibility and thresholds belong entirely to the existing engine.

use ej_core::{Claim, Evidence, Finding};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// How to divide supplied study text into candidates for the engine.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Segmentation {
    /// Split at newlines and `.`, `?`, `!` followed by whitespace or end of text.
    Sentences,
    /// Check each nonempty line intact; useful for caller-reviewed atomic claims.
    Lines,
}

/// An unchanged slice of the study text, with a unique positional identifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Candidate {
    /// Positional ID (`claim-1`, `claim-2`, ...); repeated text keeps distinct IDs.
    pub id: String,
    /// UTF-8 byte offset of the first included character in the input.
    pub start: usize,
    /// Exclusive UTF-8 byte offset of the last included character in the input.
    pub end: usize,
    /// Exact input slice after trimming surrounding whitespace.
    pub text: String,
}

/// Split input without rewriting, classifying or silently dropping any non-whitespace text.
#[must_use]
pub fn candidates(text: &str, mode: Segmentation) -> Vec<Candidate> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        let end = i + ch.len_utf8();
        let terminal = mode == Segmentation::Sentences
            && matches!(ch, '.' | '?' | '!')
            && chars.peek().is_none_or(|(_, next)| next.is_whitespace());
        if ch == '\n' || terminal {
            add_candidate(text, start, end, &mut out);
            start = end;
        }
    }
    add_candidate(text, start, text.len(), &mut out);
    out
}

fn add_candidate(text: &str, start: usize, end: usize, out: &mut Vec<Candidate>) {
    let slice = &text[start..end];
    let trimmed = slice.trim();
    if !trimmed.is_empty() {
        let start = start + slice.len() - slice.trim_start().len();
        out.push(Candidate {
            id: format!("claim-{}", out.len() + 1),
            start,
            end: start + trimmed.len(),
            text: trimmed.into(),
        });
    }
}

/// An existing local model directory and its existing fitted policy, forwarded unchanged.
#[derive(Debug, Clone)]
pub struct LocalModel {
    /// ONNX directory accepted by `ej check --nli`.
    pub directory: PathBuf,
    /// Policy produced by `ej-bench fit --write-policy`; eligibility is enforced by `ej`.
    pub policy: PathBuf,
}

/// Invocation settings for the existing production binary; no judgement settings are fitted here.
#[derive(Debug, Clone)]
pub struct Engine {
    /// Path to the `ej` binary (or `ej` to resolve it through PATH).
    pub program: PathBuf,
    /// Explicit ISO date used by the engine's existing evidence policy.
    pub today: String,
    /// Optional evidence age limit forwarded to `ej check --max-age`.
    pub max_age_days: Option<u32>,
    /// Optional existing local model and policy; no model is enabled by default.
    pub local_model: Option<LocalModel>,
}

/// A candidate and the engine's original finding, with the existing pass rule exposed for review.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckedClaim {
    /// Location and unchanged text in the study material.
    pub candidate: Candidate,
    /// EXPIRA's finding, preserved without changing its verdict, confidence, layer or reason.
    pub finding: Finding,
    /// True when the existing `Verdict::passes()` rule requires the claim to be reviewed.
    pub flagged: bool,
}

/// Results for all candidates, retaining the exact source snapshot supplied to the engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StudyReport {
    /// Complete supplied document, source identifier and actual date opened.
    pub source: Evidence,
    /// Date used by the engine to evaluate evidence age.
    pub today: String,
    /// Segmentation mode used to create candidates.
    pub segmentation: Segmentation,
    /// Results in study-text order, including repeated candidates and unresolved claims.
    pub claims: Vec<CheckedClaim>,
}

impl StudyReport {
    /// True if at least one original engine verdict does not pass.
    #[must_use]
    pub fn has_flags(&self) -> bool {
        self.claims.iter().any(|c| c.flagged)
    }
}

/// Check every candidate against one source snapshot by invoking the existing `ej check` once.
///
/// All source text is attached as one verbatim quote per candidate. Engine stderr is inherited,
/// so skipped-model diagnostics remain visible. A private temporary file is removed on return.
///
/// # Errors
/// Empty notes or source identifier, invalid dates, process/setup failure, malformed findings,
/// missing/duplicate/unknown claim IDs, invalid confidences, or exit status inconsistent with findings.
pub fn check_study(
    text: &str,
    source: &Evidence,
    mode: Segmentation,
    engine: &Engine,
) -> Result<StudyReport, String> {
    for (name, date) in [("source date", &source.date), ("today", &engine.today)] {
        let iso_shape = date.len() == 10
            && date.bytes().enumerate().all(|(i, ch)| {
                if i == 4 || i == 7 {
                    ch == b'-'
                } else {
                    ch.is_ascii_digit()
                }
            });
        if !iso_shape || ej_evidence::day_number(date).is_none() {
            return Err(format!("{name}: expected a valid YYYY-MM-DD date"));
        }
    }
    if source.source.trim().is_empty() {
        return Err("source identifier must not be empty".into());
    }
    let candidates = candidates(text, mode);
    if candidates.is_empty() {
        return Err("study text contains no candidates".into());
    }
    let claims: Vec<_> = candidates
        .iter()
        .map(|c| Claim {
            id: c.id.clone(),
            text: c.text.clone(),
            evidence: vec![source.clone()],
        })
        .collect();
    let file = tempfile::NamedTempFile::new().map_err(|e| format!("temporary claims file: {e}"))?;
    serde_json::to_writer(file.as_file(), &claims).map_err(|e| format!("write claims: {e}"))?;
    let mut command = Command::new(&engine.program);
    command
        .arg("check")
        .arg(file.path())
        .arg("--today")
        .arg(&engine.today)
        .stdin(Stdio::null())
        .stderr(Stdio::inherit());
    if let Some(days) = engine.max_age_days {
        command.arg("--max-age").arg(days.to_string());
    }
    if let Some(model) = &engine.local_model {
        command
            .arg("--nli")
            .arg(&model.directory)
            .arg("--policy")
            .arg(&model.policy);
    }
    let output = command.output().map_err(|e| format!("start ej: {e}"))?;
    let code = output.status.code();
    if !matches!(code, Some(0 | 1)) {
        return Err(format!("ej failed with {}", output.status));
    }
    let checked = join_findings(candidates, &output.stdout)?;
    let report = StudyReport {
        source: source.clone(),
        today: engine.today.clone(),
        segmentation: mode,
        claims: checked,
    };
    if code != Some(i32::from(report.has_flags())) {
        return Err("ej exit status disagrees with its findings".into());
    }
    Ok(report)
}

fn join_findings(candidates: Vec<Candidate>, bytes: &[u8]) -> Result<Vec<CheckedClaim>, String> {
    let findings: Vec<Finding> =
        serde_json::from_slice(bytes).map_err(|e| format!("ej findings: {e}"))?;
    if findings.len() != candidates.len() {
        return Err("ej returned a different number of findings than candidates".into());
    }
    let mut by_id = BTreeMap::new();
    for finding in findings {
        if !finding.confidence.is_finite() || !(0.0..=1.0).contains(&finding.confidence) {
            return Err("ej returned an invalid confidence".into());
        }
        if by_id.insert(finding.claim_id.clone(), finding).is_some() {
            return Err("ej returned a duplicate claim ID".into());
        }
    }
    candidates
        .into_iter()
        .map(|candidate| {
            let finding = by_id
                .remove(&candidate.id)
                .ok_or_else(|| format!("ej omitted {} or returned an unknown ID", candidate.id))?;
            Ok(CheckedClaim {
                flagged: !finding.verdict.passes(),
                candidate,
                finding,
            })
        })
        .collect()
}
