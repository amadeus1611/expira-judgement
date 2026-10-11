//! Messages API transport over HTTPS. Key from ANTHROPIC_API_KEY; proxy from the usual env vars; system CA roots.

use ej_judge::claude::Transport;
use serde_json::Value;

pub const URL: &str = "https://api.anthropic.com/v1/messages";
pub const VERSION: &str = "2023-06-01";

pub struct Http { agent: ureq::Agent, key: String }

impl Http {
    pub fn from_env() -> Result<Self, String> {
        let key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| "ANTHROPIC_API_KEY not set".to_string())?;
        if key.trim().is_empty() { return Err("ANTHROPIC_API_KEY is empty".into()); }
        let tls = ureq::tls::TlsConfig::builder().root_certs(ureq::tls::RootCerts::PlatformVerifier).build();
        let agent = ureq::Agent::config_builder().tls_config(tls).timeout_global(Some(std::time::Duration::from_secs(60))).http_status_as_error(false).build().into();
        Ok(Http { agent, key })
    }
}

impl Transport for Http {
    fn post(&self, body: &Value) -> Result<Value, String> {
        let mut resp = self.agent.post(URL).header("x-api-key", &self.key).header("anthropic-version", VERSION).header("anthropic-beta", ej_judge::claude::FALLBACK_BETA).send_json(body).map_err(|e| e.to_string())?;
        let status = resp.status().as_u16();
        let v: Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;
        if status != 200 { return Err(format!("HTTP {status}: {}", v["error"]["message"].as_str().unwrap_or("unknown error"))); }
        Ok(v)
    }
}
