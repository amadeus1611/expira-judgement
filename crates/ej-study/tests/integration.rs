//! Integration-only tests: candidate coverage and the production binary's transport contract.

use ej_core::{Evidence, Finding, Layer, Verdict};
use ej_study::{candidates, check_study, Engine, LocalModel, Segmentation, StudyReport};

#[test]
fn segmentation_retains_unicode_decimals_duplicates_and_trailing_text() {
    let text = "  Café uses 1.5 units. Why? Yes!\r\n\nCafé uses 1.5 units.\n trailing text  ";
    let got = candidates(text, Segmentation::Sentences);
    assert_eq!(
        got.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(),
        [
            "Café uses 1.5 units.",
            "Why?",
            "Yes!",
            "Café uses 1.5 units.",
            "trailing text"
        ]
    );
    for (i, c) in got.iter().enumerate() {
        assert_eq!(c.id, format!("claim-{}", i + 1));
        assert_eq!(&text[c.start..c.end], c.text);
    }
    for (i, ch) in text.char_indices().filter(|(_, ch)| !ch.is_whitespace()) {
        assert!(got.iter().any(|c| (c.start..c.end).contains(&i)), "{ch}");
    }
    assert!(candidates(" \n\t", Segmentation::Sentences).is_empty());
    assert_eq!(candidates("end.", Segmentation::Sentences)[0].end, 4);
}

#[test]
fn lines_mode_keeps_abbreviations_and_compound_lines_intact() {
    let text = " Dr. Example uses e.g. notes. Second sentence.\r\n\nAnother line! ";
    let got = candidates(text, Segmentation::Lines);
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].text, "Dr. Example uses e.g. notes. Second sentence.");
    assert_eq!(got[1].text, "Another line!");
    for c in got {
        assert_eq!(&text[c.start..c.end], c.text);
    }
}

#[cfg(unix)]
mod processes {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::process::Command;

    struct Fixture {
        dir: tempfile::TempDir,
        engine: Engine,
        source: Evidence,
    }

    impl Fixture {
        fn new(stdout: &str, status: i32) -> Self {
            let dir = tempfile::tempdir().unwrap();
            let program = dir.path().join("fake ej with spaces");
            let script = format!(
                "#!/usr/bin/env python3\nimport json, pathlib, sys\n\
                 assert sys.argv[1] == 'check'\n\
                 path = pathlib.Path(sys.argv[2])\n\
                 claims = json.loads(path.read_text())\n\
                 pathlib.Path(__file__).with_suffix('.request').write_text(json.dumps({{'claims': claims, 'args': sys.argv[3:], 'file': str(path)}}))\n\
                 sys.stdout.write({stdout})\nsys.exit({status})\n",
                stdout = serde_json::to_string(stdout).unwrap(),
            );
            std::fs::write(&program, script).unwrap();
            std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
            Self {
                engine: Engine {
                    program,
                    today: "2026-10-14".into(),
                    max_age_days: None,
                    local_model: None,
                },
                source: Evidence {
                    source: "https://example.test/source".into(),
                    date: "2026-10-13".into(),
                    quote: "Verbatim source.\nCafé's data: $(not-a-command) `literal`.".into(),
                },
                dir,
            }
        }

        fn request(&self) -> serde_json::Value {
            let path = self.engine.program.with_extension("request");
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
        }

        fn check(&self, text: &str) -> Result<StudyReport, String> {
            check_study(text, &self.source, Segmentation::Sentences, &self.engine)
        }

        fn cli(&self, extra: &[&str]) -> std::process::Output {
            let notes = self.dir.path().join("notes.txt");
            let source = self.dir.path().join("source.txt");
            std::fs::write(&notes, "One.").unwrap();
            std::fs::write(&source, &self.source.quote).unwrap();
            Command::new(env!("CARGO_BIN_EXE_ej-study"))
                .args(["--text", notes.to_str().unwrap()])
                .args(["--source", source.to_str().unwrap()])
                .args(["--source-date", &self.source.date])
                .args(["--today", &self.engine.today])
                .arg("--ej")
                .arg(&self.engine.program)
                .args(extra)
                .output()
                .unwrap()
        }
    }

    fn finding(id: &str, verdict: Verdict) -> Finding {
        Finding {
            claim_id: id.into(),
            verdict,
            layer: Layer::LocalJudge,
            confidence: 0.875,
            reason: "original engine reason".into(),
        }
    }

    fn reply(findings: &[Finding]) -> String {
        serde_json::to_string(findings).unwrap()
    }

    #[test]
    fn forwards_complete_source_and_existing_policy_and_cleans_up_file() {
        let expected = finding("claim-1", Verdict::Supported);
        let mut fixture = Fixture::new(&reply(std::slice::from_ref(&expected)), 0);
        fixture.engine.max_age_days = Some(30);
        fixture.engine.local_model = Some(LocalModel {
            directory: "model with spaces".into(),
            policy: "existing policy.json".into(),
        });
        let report = fixture.check("One.").unwrap();
        assert_eq!(report.source, fixture.source);
        assert_eq!(report.today, "2026-10-14");
        assert_eq!(report.segmentation, Segmentation::Sentences);
        assert_eq!(report.claims[0].finding, expected);
        assert!(!report.has_flags());
        let request = fixture.request();
        let claims: Vec<ej_core::Claim> =
            serde_json::from_value(request["claims"].clone()).unwrap();
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].id, "claim-1");
        assert_eq!(claims[0].text, "One.");
        assert_eq!(claims[0].evidence, [fixture.source]);
        assert_eq!(
            request["args"],
            serde_json::json!([
                "--today",
                "2026-10-14",
                "--max-age",
                "30",
                "--nli",
                "model with spaces",
                "--policy",
                "existing policy.json"
            ])
        );
        assert!(!PathBuf::from(request["file"].as_str().unwrap()).exists());
    }

    #[test]
    fn preserves_every_verdict_and_rejoins_out_of_order_findings_by_id() {
        let verdicts = [
            Verdict::Backed,
            Verdict::Supported,
            Verdict::Unbacked,
            Verdict::Mismatch,
            Verdict::Stale,
            Verdict::Unsure,
        ];
        let expected: Vec<_> = verdicts
            .iter()
            .enumerate()
            .map(|(i, v)| finding(&format!("claim-{}", i + 1), *v))
            .collect();
        let reversed: Vec<_> = expected.iter().rev().cloned().collect();
        let fixture = Fixture::new(&reply(&reversed), 1);
        let report = fixture.check("One. Two. Three. Four. Five. Six.").unwrap();
        assert!(report.has_flags());
        for (i, (checked, expected)) in report.claims.iter().zip(expected).enumerate() {
            assert_eq!(checked.finding, expected);
            assert_eq!(checked.flagged, i >= 2);
            assert_eq!(checked.candidate.id, checked.finding.claim_id);
        }
    }

    #[test]
    fn rejects_missing_unknown_duplicate_and_malformed_findings() {
        let one = finding("claim-1", Verdict::Backed);
        let two = finding("claim-2", Verdict::Backed);
        let mut bad_confidence = two.clone();
        bad_confidence.confidence = 1.01;
        let mut negative_confidence = two.clone();
        negative_confidence.confidence = -0.01;
        for stdout in [
            "not JSON".into(),
            "{}".into(),
            "[{\"claim_id\":\"claim-1\"}]".into(),
            reply(&[]),
            reply(std::slice::from_ref(&one)),
            reply(&[
                one.clone(),
                two.clone(),
                finding("claim-3", Verdict::Backed),
            ]),
            reply(&[one.clone(), one.clone()]),
            reply(&[one.clone(), finding("unknown", Verdict::Backed)]),
            reply(&[one.clone(), bad_confidence]),
            reply(&[one.clone(), negative_confidence]),
            reply(&[one.clone(), two]).replace("\"backed\"", "\"invented_verdict\""),
        ] {
            let fixture = Fixture::new(&stdout, 0);
            assert!(fixture.check("One. Two.").is_err(), "accepted {stdout}");
            assert!(!PathBuf::from(fixture.request()["file"].as_str().unwrap()).exists());
        }
    }

    #[test]
    fn rejects_process_failure_and_status_disagreement_even_with_valid_json() {
        for (verdict, status) in [
            (Verdict::Backed, 1),
            (Verdict::Unsure, 0),
            (Verdict::Backed, 2),
            (Verdict::Backed, 7),
        ] {
            let fixture = Fixture::new(&reply(&[finding("claim-1", verdict)]), status);
            assert!(fixture.check("One.").is_err());
        }
        let mut fixture = Fixture::new("[]", 0);
        fixture.engine.program = fixture.dir.path().join("missing executable");
        assert!(fixture.check("One.").unwrap_err().contains("start ej"));
    }

    #[test]
    fn rejects_invalid_input_before_starting_the_engine() {
        let mut fixture = Fixture::new("[]", 0);
        assert!(fixture.check(" \n ").is_err());
        for date in [
            "",
            "yesterday",
            "2026-02-30",
            "2026-10-14 trailing",
            "2026-1-014",
        ] {
            fixture.source.date = date.into();
            assert!(fixture.check("One.").is_err());
        }
        fixture.source.date = "2026-10-13".into();
        fixture.engine.today = "2026-13-01".into();
        assert!(fixture.check("One.").is_err());
        fixture.engine.today = "2026-10-14".into();
        fixture.source.source.clear();
        assert!(fixture.check("One.").is_err());
        assert!(!fixture.engine.program.with_extension("request").exists());
    }

    #[test]
    fn an_empty_document_is_forwarded_for_the_engine_to_judge() {
        let mut fixture = Fixture::new(&reply(&[finding("claim-1", Verdict::Unbacked)]), 1);
        fixture.source.quote.clear();
        let report = fixture.check("One.").unwrap();
        assert!(report.has_flags());
        assert_eq!(fixture.request()["claims"][0]["evidence"][0]["quote"], "");
    }

    #[test]
    fn cli_reports_passes_flags_and_protocol_errors_with_distinct_exit_codes() {
        for (verdict, status) in [(Verdict::Backed, 0), (Verdict::Unsure, 1)] {
            let fixture = Fixture::new(&reply(&[finding("claim-1", verdict)]), status);
            let output = fixture.cli(&["--source-id", "Study source", "--lines"]);
            assert_eq!(output.status.code(), Some(status));
            let report: StudyReport = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(report.source.source, "Study source");
            assert_eq!(report.segmentation, Segmentation::Lines);
            assert_eq!(report.has_flags(), status == 1);
        }
        let fixture = Fixture::new("[]", 0);
        for opts in [
            vec!["--unknown"],
            vec!["--lines", "--lines"],
            vec!["--today", "2026-10-14"],
            vec!["--max-age", "NaN"],
            vec!["--max-age", "-1"],
            vec!["--max-age"],
            vec!["--nli", "model"],
            vec!["--policy", "policy.json"],
        ] {
            let output = fixture.cli(&opts);
            assert_eq!(output.status.code(), Some(2), "{opts:?}");
            assert!(output.stdout.is_empty());
            assert!(!output.stderr.is_empty());
        }
        let output = fixture.cli(&[]); // Incomplete engine results.
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let output = Command::new(env!("CARGO_BIN_EXE_ej-study"))
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        let output = Command::new(env!("CARGO_BIN_EXE_ej-study"))
            .arg("--help")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout)
            .unwrap()
            .contains("usage: ej-study"));
    }

    #[test]
    fn cli_forwards_optional_model_and_age_and_defaults_to_the_source_path() {
        let fixture = Fixture::new(&reply(&[finding("claim-1", Verdict::Backed)]), 0);
        let output = fixture.cli(&[
            "--max-age",
            "0",
            "--nli",
            "model directory",
            "--policy",
            "policy path.json",
        ]);
        assert!(output.status.success());
        let report: StudyReport = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            report.source.source,
            fixture.dir.path().join("source.txt").to_str().unwrap()
        );
        assert_eq!(report.segmentation, Segmentation::Sentences);
        assert_eq!(
            fixture.request()["args"],
            serde_json::json!([
                "--today",
                "2026-10-14",
                "--max-age",
                "0",
                "--nli",
                "model directory",
                "--policy",
                "policy path.json"
            ])
        );
        std::fs::remove_file(fixture.engine.program.with_extension("request")).unwrap();
        // Deleting the source tests a CLI IO failure before the engine is called.
        let missing = fixture.dir.path().join("missing.txt");
        let output = Command::new(env!("CARGO_BIN_EXE_ej-study"))
            .args(["--text", missing.to_str().unwrap()])
            .args(["--source", missing.to_str().unwrap()])
            .args(["--source-date", "2026-10-14", "--today", "2026-10-14"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!fixture.engine.program.with_extension("request").exists());
    }
}
