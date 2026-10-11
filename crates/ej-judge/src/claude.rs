//! Layer 3: Claude as the escalation judge, via the Messages API.
//! The HTTP call sits behind `Transport` so the judge is testable offline. Every ruling Claude gives with a reason
//! is kept as a teacher pair (ej-habits) for the local judge's eval set; a ruling without a reason counts as Unsure.

use crate::{Judge, Ruling};
use ej_core::{Claim, Layer};
use ej_habits::TeacherPair;
use serde_json::{json, Value};
use std::sync::Mutex;

/// Sends one Messages API request body and returns the response body.
pub trait Transport {
    fn post(&self, body: &Value) -> Result<Value, String>;
}

pub struct ClaudeJudge<T: Transport> {
    pub transport: T,
    pub model: String,
    pub max_tokens: u32,
    pairs: Mutex<Vec<TeacherPair>>,
}

const SYSTEM: &str = "You check whether quoted evidence supports a claim. Use only the quotes given; do not use outside knowledge. \
Answer with one JSON object and nothing else: {\"ruling\": \"supports\" | \"contradicts\" | \"unsure\", \"confidence\": 0.0-1.0, \"reason\": \"one sentence citing the quote\"}. \
Say unsure if the quotes neither state nor directly imply the claim.";

impl<T: Transport> ClaudeJudge<T> {
    pub fn new(transport: T, model: impl Into<String>) -> Self {
        ClaudeJudge { transport, model: model.into(), max_tokens: 300, pairs: Mutex::new(Vec::new()) }
    }

    pub fn request(&self, claim: &Claim) -> Value {
        let quotes = claim.evidence.iter().enumerate().map(|(i, e)| format!("[{}] ({}, {}) \"{}\"", i + 1, e.source, e.date, e.quote)).collect::<Vec<_>>().join("\n");
        json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "system": SYSTEM,
            "messages": [{ "role": "user", "content": format!("Claim: {}\n\nQuotes:\n{}", claim.text, quotes) }],
        })
    }

    /// Teacher pairs gathered so far, drained.
    pub fn take_pairs(&self) -> Vec<TeacherPair> {
        std::mem::take(&mut *self.pairs.lock().unwrap())
    }
}

/// Reads the first `{...}` object in the response text into a ruling. None if it is missing or malformed.
pub fn parse_ruling(response: &Value) -> Option<(Ruling, f32, String)> {
    let text: String = response["content"].as_array()?.iter().filter_map(|b| b["text"].as_str()).collect();
    let (start, end) = (text.find('{')?, text.rfind('}')?);
    let v: Value = serde_json::from_str(text.get(start..=end)?).ok()?;
    let ruling = match v["ruling"].as_str()? { "supports" => Ruling::Supports, "contradicts" => Ruling::Contradicts, "unsure" => Ruling::Unsure, _ => return None };
    let conf = v["confidence"].as_f64()?.clamp(0.0, 1.0) as f32;
    let reason = v["reason"].as_str().unwrap_or("").trim().to_string();
    Some((ruling, conf, reason))
}

impl<T: Transport> Judge for ClaudeJudge<T> {
    fn layer(&self) -> Layer { Layer::Claude }
    fn rule(&self, claim: &Claim) -> (Ruling, f32, String) {
        let resp = match self.transport.post(&self.request(claim)) { Ok(r) => r, Err(e) => return (Ruling::Unsure, 0.0, format!("claude call failed: {e}")) };
        let Some((ruling, conf, reason)) = parse_ruling(&resp) else { return (Ruling::Unsure, 0.0, "claude reply not parseable".into()) };
        if reason.is_empty() { return (Ruling::Unsure, 0.0, "claude gave no reason".into()); }
        let label = match ruling { Ruling::Supports => "supports", Ruling::Contradicts => "contradicts", Ruling::Unsure => "unsure" };
        let quote = claim.evidence.iter().map(|e| e.quote.as_str()).collect::<Vec<_>>().join("\n");
        self.pairs.lock().unwrap().push(TeacherPair { claim: claim.text.clone(), quote, ruling: label.into(), reason: reason.clone() });
        (ruling, conf, reason)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{adjudicate, Bars, OverlapJudge};
    use ej_core::{Evidence, Verdict};

    struct Canned(Result<Value, String>);
    impl Transport for Canned { fn post(&self, _: &Value) -> Result<Value, String> { self.0.clone() } }
    fn reply(text: &str) -> Canned { Canned(Ok(json!({ "content": [{ "type": "text", "text": text }] }))) }
    fn claim() -> Claim { Claim { id: "c".into(), text: "orders carry the fee".into(), evidence: vec![Evidence { source: "s".into(), date: "2026-01-01".into(), quote: "a management fee applies to all orders".into() }] } }

    #[test]
    fn request_carries_claim_quotes_and_model() {
        let j = ClaudeJudge::new(reply(""), "m-test");
        let r = j.request(&claim());
        assert_eq!(r["model"], "m-test");
        let msg = r["messages"][0]["content"].as_str().unwrap();
        assert!(msg.contains("orders carry the fee") && msg.contains("a management fee applies to all orders"));
    }

    #[test]
    fn ruling_settles_escalation_and_is_kept_as_teacher_pair() {
        let j = ClaudeJudge::new(reply("```json\n{\"ruling\":\"supports\",\"confidence\":0.92,\"reason\":\"quote says the fee applies to all orders\"}\n```"), "m");
        let f = adjudicate(&claim(), &[&OverlapJudge, &j], Bars::default());
        assert_eq!((f.verdict, f.layer), (Verdict::Supported, Layer::Claude));
        let pairs = j.take_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].ruling, "supports");
        assert!(j.take_pairs().is_empty());
    }

    #[test]
    fn no_reason_or_bad_reply_or_error_is_unsure_and_not_kept() {
        for t in [reply("{\"ruling\":\"supports\",\"confidence\":0.99,\"reason\":\"\"}"), reply("sure, it supports it"), Canned(Err("timeout".into()))] {
            let j = ClaudeJudge::new(t, "m");
            assert_eq!(j.rule(&claim()).0, Ruling::Unsure);
            assert!(j.take_pairs().is_empty());
        }
    }
}
