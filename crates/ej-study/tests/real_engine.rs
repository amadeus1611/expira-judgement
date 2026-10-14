//! Optional real-production-binary smoke: transport parity, never a model-quality benchmark.

use ej_core::{Claim, Evidence, Finding, Verdict};
use ej_study::{check_study, Engine, LocalModel, Segmentation};
use std::process::Command;

#[test]
#[ignore = "build ej-cli and set EJ_STUDY_ENGINE to its absolute binary path"]
fn wrapper_findings_equal_direct_production_findings() {
    let mut engine = Engine {
        program: std::env::var_os("EJ_STUDY_ENGINE")
            .expect("set EJ_STUDY_ENGINE to the prebuilt ej binary")
            .into(),
        today: "2026-10-14".into(),
        max_age_days: None,
        local_model: None,
    };
    // Fresh synthetic training prose, not an exam reference or calibration fixture.
    let mut source = Evidence {
        source: "synthetic smoke document".into(),
        date: "2026-10-13".into(),
        quote: "A firewall filters network traffic. The exercise lasts 30 days.".into(),
    };
    let text = "A firewall filters network traffic. The exercise lasts 60 days. Encryption hides information.";
    let dir = tempfile::tempdir().unwrap();
    let claims_path = dir.path().join("direct-claims.json");
    let policy_path = dir.path().join("ineligible-policy.json");
    std::fs::write(&policy_path, r#"{"judges":{"nli":{"passed_v1":false}}}"#).unwrap();
    for case in 0..4 {
        match case {
            1 => engine.max_age_days = Some(0),
            2 => {
                engine.max_age_days = None;
                source.quote.clear();
            }
            3 => {
                source.quote =
                    "A firewall filters network traffic. The exercise lasts 30 days.".into();
                engine.local_model = Some(LocalModel {
                    directory: dir.path().join("missing-model-must-not-load"),
                    policy: policy_path.clone(),
                });
            }
            _ => {}
        }
        let report = check_study(text, &source, Segmentation::Sentences, &engine).unwrap();
        let claims: Vec<_> = report
            .claims
            .iter()
            .map(|checked| Claim {
                id: checked.candidate.id.clone(),
                text: checked.candidate.text.clone(),
                evidence: vec![source.clone()],
            })
            .collect();
        std::fs::write(&claims_path, serde_json::to_vec(&claims).unwrap()).unwrap();
        let mut command = Command::new(&engine.program);
        command
            .arg("check")
            .arg(&claims_path)
            .arg("--today")
            .arg(&engine.today);
        if let Some(age) = engine.max_age_days {
            command.arg("--max-age").arg(age.to_string());
        }
        if let Some(model) = &engine.local_model {
            command
                .arg("--nli")
                .arg(&model.directory)
                .arg("--policy")
                .arg(&model.policy);
        }
        let direct = command.output().unwrap();
        assert_eq!(direct.status.code(), Some(1));
        let findings: Vec<Finding> = serde_json::from_slice(&direct.stdout).unwrap();
        let actual: Vec<_> = report.claims.iter().map(|c| c.finding.clone()).collect();
        assert_eq!(actual, findings);
        assert_eq!(report.source, source);
        assert!(report.has_flags());
        let verdicts: Vec<_> = actual.iter().map(|f| f.verdict).collect();
        assert_eq!(
            verdicts,
            match case {
                1 => vec![Verdict::Stale; 3],
                2 => vec![Verdict::Unbacked; 3],
                _ => vec![Verdict::Backed, Verdict::Mismatch, Verdict::Unsure],
            }
        );
        if case == 3 {
            assert!(String::from_utf8(direct.stderr)
                .unwrap()
                .contains("nli has not passed"));
        }
    }
}
