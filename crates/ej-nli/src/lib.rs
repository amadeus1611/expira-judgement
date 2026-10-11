//! Layer 2 candidate: an NLI cross-encoder (e.g. DeBERTa-v3) run on CPU through ONNX Runtime.
//! Premise = the quotes, hypothesis = the claim. Entailment -> Supports, contradiction -> Contradicts,
//! neutral -> Unsure; confidence is the softmax probability of the winning class.
//! Expects a directory with `model.onnx`, `tokenizer.json` and `config.json` (for `id2label`).

use ej_core::{Claim, Layer};
use ej_judge::{Judge, Ruling};
use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;
use std::sync::Mutex;
use tokenizers::Tokenizer;

pub struct NliJudge {
    session: Mutex<Session>,
    tokenizer: Tokenizer,
    /// Ruling per output index, read from config.json.
    labels: Vec<Ruling>,
    inputs: Vec<String>,
}

fn ruling_for(label: &str) -> Option<Ruling> {
    match label.to_lowercase().as_str() {
        "entailment" => Some(Ruling::Supports),
        "contradiction" => Some(Ruling::Contradicts),
        "neutral" => Some(Ruling::Unsure),
        _ => None,
    }
}

/// Reads `id2label` into a ruling per index. Errors on unknown labels rather than guessing.
pub fn labels_from_config(config: &serde_json::Value) -> Result<Vec<Ruling>, String> {
    let map = config["id2label"].as_object().ok_or("config.json has no id2label")?;
    let mut out = vec![None; map.len()];
    for (k, v) in map {
        let i: usize = k.parse().map_err(|_| format!("bad id2label key {k}"))?;
        let l = v.as_str().ok_or("id2label value not a string")?;
        *out.get_mut(i).ok_or(format!("id2label index {i} out of range"))? = Some(ruling_for(l).ok_or(format!("unknown label {l}"))?);
    }
    out.into_iter().collect::<Option<Vec<_>>>().ok_or_else(|| "id2label has gaps".into())
}

pub fn softmax(x: &[f32]) -> Vec<f32> {
    let m = x.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let e: Vec<f32> = x.iter().map(|v| (v - m).exp()).collect();
    let s: f32 = e.iter().sum();
    e.into_iter().map(|v| v / s).collect()
}

impl NliJudge {
    pub fn load(dir: &Path) -> Result<Self, String> {
        let config: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("config.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let labels = labels_from_config(&config)?;
        let mut tokenizer = Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| e.to_string())?;
        tokenizer.with_truncation(Some(tokenizers::TruncationParams { max_length: 512, ..Default::default() })).map_err(|e| e.to_string())?;
        let session = Session::builder().and_then(|b| b.with_intra_threads(1)).and_then(|b| b.commit_from_file(dir.join("model.onnx"))).map_err(|e| e.to_string())?;
        let inputs = session.inputs.iter().map(|i| i.name.clone()).collect();
        Ok(NliJudge { session: Mutex::new(session), tokenizer, labels, inputs })
    }

    /// Class probabilities for (premise, hypothesis), in config order.
    pub fn probs(&self, premise: &str, hypothesis: &str) -> Result<Vec<f32>, String> {
        let enc = self.tokenizer.encode((premise, hypothesis), true).map_err(|e| e.to_string())?;
        let n = enc.get_ids().len();
        let col = |v: &[u32]| v.iter().map(|&x| x as i64).collect::<Vec<i64>>();
        let mut feed: Vec<(String, Tensor<i64>)> = Vec::new();
        for name in &self.inputs {
            let data = match name.as_str() {
                "input_ids" => col(enc.get_ids()),
                "attention_mask" => col(enc.get_attention_mask()),
                "token_type_ids" => col(enc.get_type_ids()),
                other => return Err(format!("unexpected model input {other}")),
            };
            feed.push((name.clone(), Tensor::from_array(([1usize, n], data)).map_err(|e| e.to_string())?));
        }
        let mut session = self.session.lock().unwrap();
        let out = session.run(feed).map_err(|e| e.to_string())?;
        let (_, logits) = out[0].try_extract_tensor::<f32>().map_err(|e| e.to_string())?;
        Ok(softmax(logits))
    }
}

impl Judge for NliJudge {
    fn layer(&self) -> Layer { Layer::LocalJudge }
    fn p_stated(&self, claim: &Claim) -> Option<f32> {
        let premise = claim.evidence.iter().map(|e| e.quote.as_str()).collect::<Vec<_>>().join(" ");
        let p = self.probs(&premise, &claim.text).ok()?;
        self.labels.iter().position(|r| *r == Ruling::Supports).map(|i| p[i])
    }
    fn rule(&self, claim: &Claim) -> (Ruling, f32, String) {
        let premise = claim.evidence.iter().map(|e| e.quote.as_str()).collect::<Vec<_>>().join(" ");
        match self.probs(&premise, &claim.text) {
            Err(e) => (Ruling::Unsure, 0.0, format!("nli failed: {e}")),
            Ok(p) => {
                let (i, &c) = p.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap();
                let r = self.labels[i];
                (r, c, format!("nli {r:?} p={c:.2}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_label_order_from_config() {
        let cfg = serde_json::json!({ "id2label": { "0": "contradiction", "1": "entailment", "2": "neutral" } });
        assert_eq!(labels_from_config(&cfg).unwrap(), vec![Ruling::Contradicts, Ruling::Supports, Ruling::Unsure]);
        assert!(labels_from_config(&serde_json::json!({ "id2label": { "0": "LABEL_0" } })).is_err());
    }

    #[test]
    fn softmax_sums_to_one() {
        let p = softmax(&[2.0, 1.0, 0.1]);
        assert!((p.iter().sum::<f32>() - 1.0).abs() < 1e-6 && p[0] > p[1] && p[1] > p[2]);
    }

    /// Runs only when the model is present (models/ is git-ignored): EJ_NLI_DIR=models/nli-deberta-v3-xsmall
    #[test]
    fn model_smoke() {
        let Ok(dir) = std::env::var("EJ_NLI_DIR") else { return };
        let j = NliJudge::load(Path::new(&dir)).unwrap();
        let c = |t: &str, q: &str| Claim { id: "c".into(), text: t.into(), evidence: vec![ej_core::Evidence { source: "s".into(), date: "2026-01-01".into(), quote: q.into() }] };
        assert_eq!(j.rule(&c("The store opens on Monday.", "The store is closed on Mondays.")).0, Ruling::Contradicts);
        assert_eq!(j.rule(&c("A fee applies to orders.", "A management fee applies to all orders.")).0, Ruling::Supports);
    }
}
