//! Layer 3: Claude as the escalation judge, via the Messages API.
//! The HTTP call sits behind `Transport` so the judge is testable offline. Every ruling Claude gives with a reason
//! is kept as a teacher pair (ej-habits) for the local judge's eval set; a ruling without a reason counts as Unsure.
//!
//! The reply is constrained by a JSON schema (structured outputs), and must carry `evidence_span`: words copied from a
//! quote. Code checks that span against the quotes, so a Supports or Contradicts ruling that cites text not in the
//! evidence is downgraded to Unsure: the escalation layer is held to the same "quote or it didn't happen" rule as layer 1.
//! A refusal (`stop_reason: "refusal"`) is Unsure. Server-side fallbacks are requested so a declined request is retried
//! on Anthropic's recommended fallback model instead of failing.

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
    /// `output_config.effort`. Claim checking is a short classification, so the default is "low"; raise it per route
    /// only if a measured eval shows it pays.
    pub effort: String,
    pairs: Mutex<Vec<TeacherPair>>,
}

const SYSTEM: &str = "You check whether quoted evidence supports a claim. Use only the quotes given; do not use outside knowledge. \
supports: the quotes state the claim or directly imply it, with the same figures, units, dates and polarity. \
contradicts: the quotes state something incompatible with the claim. unsure: anything else, including partial support. \
evidence_span: copy the exact words from one quote that decide your ruling (empty only when unsure). \
reason: one sentence. confidence: your probability that the ruling is right.";

/// Beta header that enables `fallbacks: "default"` (server-side retry of a declined request on the recommended model).
pub const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";

fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "ruling": { "type": "string", "enum": ["supports", "contradicts", "unsure"] },
            "confidence": { "type": "number" },
            "evidence_span": { "type": "string" },
            "reason": { "type": "string" }
        },
        "required": ["ruling", "confidence", "evidence_span", "reason"],
        "additionalProperties": false
    })
}

impl<T: Transport> ClaudeJudge<T> {
    pub fn new(transport: T, model: impl Into<String>) -> Self {
        ClaudeJudge { transport, model: model.into(), max_tokens: 2000, effort: "low".into(), pairs: Mutex::new(Vec::new()) }
    }

    pub fn request(&self, claim: &Claim) -> Value {
        let quotes = claim.evidence.iter().enumerate().map(|(i, e)| format!("[{}] ({}, {}) \"{}\"", i + 1, e.source, e.date, e.quote)).collect::<Vec<_>>().join("\n");
        json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "system": SYSTEM,
            "output_config": { "effort": self.effort, "format": { "type": "json_schema", "schema": schema() } },
            "fallbacks": "default",
            "messages": [{ "role": "user", "content": format!("Claim: {}\n\nQuotes:\n{}", claim.text, quotes) }],
        })
    }

    /// Teacher pairs gathered so far, drained.
    pub fn take_pairs(&self) -> Vec<TeacherPair> {
        std::mem::take(&mut *self.pairs.lock().unwrap())
    }
}

/// A parsed reply: ruling, confidence, reason, and the span Claude says it relied on.
pub struct Reply { pub ruling: Ruling, pub confidence: f32, pub reason: String, pub span: String }

/// Reads the reply's JSON object into a ruling. None if it is missing, malformed, or a refusal.
pub fn parse_ruling(response: &Value) -> Option<Reply> {
    if response["stop_reason"] == "refusal" { return None; }
    let text: String = response["content"].as_array()?.iter().filter_map(|b| b["text"].as_str()).collect();
    let (start, end) = (text.find('{')?, text.rfind('}')?);
    let v: Value = serde_json::from_str(text.get(start..=end)?).ok()?;
    let ruling = match v["ruling"].as_str()? { "supports" => Ruling::Supports, "contradicts" => Ruling::Contradicts, "unsure" => Ruling::Unsure, _ => return None };
    let conf = v["confidence"].as_f64()?.clamp(0.0, 1.0) as f32;
    let reason = v["reason"].as_str().unwrap_or("").trim().to_string();
    let span = v["evidence_span"].as_str().unwrap_or("").trim().to_string();
    Some(Reply { ruling, confidence: conf, reason, span })
}

/// The span appears in some quote, ignoring case and runs of whitespace.
pub fn span_in_quotes(span: &str, claim: &Claim) -> bool {
    let norm = |s: &str| s.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ");
    let sp = norm(span.trim_matches(|c: char| c == '"' || c == '\'' || c == '…' || c == '.'));
    !sp.is_empty() && claim.evidence.iter().any(|e| norm(&e.quote).contains(&sp))
}

impl<T: Transport> Judge for ClaudeJudge<T> {
    fn layer(&self) -> Layer { Layer::Claude }
    fn rule(&self, claim: &Claim) -> (Ruling, f32, String) {
        let resp = match self.transport.post(&self.request(claim)) { Ok(r) => r, Err(e) => return (Ruling::Unsure, 0.0, format!("claude call failed: {e}")) };
        let Some(Reply { ruling, confidence: conf, reason, span }) = parse_ruling(&resp) else { return (Ruling::Unsure, 0.0, "claude reply not parseable or refused".into()) };
        if reason.is_empty() { return (Ruling::Unsure, 0.0, "claude gave no reason".into()); }
        if ruling != Ruling::Unsure && !span_in_quotes(&span, claim) {
            return (Ruling::Unsure, 0.0, format!("claude cited text not in the quotes: {span:?}"));
        }
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
        assert_eq!(r["output_config"]["format"]["type"], "json_schema");
        assert_eq!(r["fallbacks"], "default");
        let msg = r["messages"][0]["content"].as_str().unwrap();
        assert!(msg.contains("orders carry the fee") && msg.contains("a management fee applies to all orders"));
    }

    #[test]
    fn ruling_settles_escalation_and_is_kept_as_teacher_pair() {
        let j = ClaudeJudge::new(reply("{\"ruling\":\"supports\",\"confidence\":0.92,\"evidence_span\":\"applies to all  Orders\",\"reason\":\"quote says the fee applies to all orders\"}"), "m");
        let f = adjudicate(&claim(), &[&OverlapJudge, &j], Bars::default());
        assert_eq!((f.verdict, f.layer), (Verdict::Supported, Layer::Claude));
        let pairs = j.take_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].ruling, "supports");
        assert!(j.take_pairs().is_empty());
    }

    #[test]
    fn no_reason_or_bad_reply_or_error_is_unsure_and_not_kept() {
        let refusal = Canned(Ok(json!({ "stop_reason": "refusal", "content": [] })));
        let invented = reply("{\"ruling\":\"supports\",\"confidence\":0.99,\"evidence_span\":\"orders carry the fee\",\"reason\":\"it says so\"}");
        for t in [reply("{\"ruling\":\"supports\",\"confidence\":0.99,\"evidence_span\":\"applies\",\"reason\":\"\"}"), reply("sure, it supports it"), Canned(Err("timeout".into())), refusal, invented] {
            let j = ClaudeJudge::new(t, "m");
            assert_eq!(j.rule(&claim()).0, Ruling::Unsure);
            assert!(j.take_pairs().is_empty());
        }
    }
}
