"""Real Rust CLI controls on authored fixtures, without weights or subscriptions."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

import cascade_capture as capture
from test_cascade_capture import fixture, ROWS


class ActualCliTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.repo = Path(__file__).resolve().parents[1]
        subprocess.run(["cargo", "build", "--locked", "-p", "ej-cli"], cwd=cls.repo, check=True,
                       capture_output=True, timeout=180)
        target = Path(os.environ.get("CARGO_TARGET_DIR", cls.repo / "target"))
        cls.ej = (target if target.is_absolute() else cls.repo / target) / "debug/ej"

    def test_actual_cli_capture_all_pass_mixed_and_empty_quote(self):
        with tempfile.TemporaryDirectory(prefix="ej-real-capture-") as directory:
            root = Path(directory)
            for name, rows in (("mixed", ROWS), ("backed", [ROWS[0]]),
                               ("no-quote", [dict(ROWS[0], quote="", label="unsure")])):
                data, ids = fixture(rows)
                (root / "data").write_bytes(data)
                (root / "ids").write_bytes(ids)
                (root / "sampling").write_text('{"population":"fresh authored controls"}')
                report = capture.capture(data_path=root / "data", identities_path=root / "ids",
                                         sampling_path=root / "sampling", ej=self.ej, repo=self.repo,
                                         today="2026-10-11", out=root / name)
                self.assertTrue(report["measurement_complete"], report["error"])
                self.assertFalse(report["certified"])
                expected = {"mixed": ["backed", "mismatch", "unsure"], "backed": ["backed"], "no-quote": ["unbacked"]}[name]
                self.assertEqual([r["finding"]["verdict"] for r in report["records"]], expected)
                self.assertEqual(report["returncode"], 0 if name == "backed" else 1)

    def test_unproven_policy_skips_nonexistent_model_before_loading(self):
        with tempfile.TemporaryDirectory(prefix="ej-unproven-control-") as directory:
            root = Path(directory)
            cases = capture.inputs(*fixture())
            (root / "claims.json").write_bytes(capture.encoded(capture.request_for(cases, "2026-10-11", "fixture")))
            (root / "policy.json").write_text('{"judges":{"nli":{"passed_v1":false}}}')
            args = [str(self.ej), "check", str(root / "claims.json"), "--today", "2026-10-11"]
            plain = subprocess.run(args, capture_output=True, text=True, timeout=10)
            disabled = subprocess.run([*args, "--nli", str(root / "model-must-not-exist"), "--policy", str(root / "policy.json")],
                                      capture_output=True, text=True, timeout=10)
            self.assertEqual(plain.returncode, 1)
            self.assertEqual(disabled.returncode, 1, disabled.stderr)
            self.assertEqual(json.loads(plain.stdout), json.loads(disabled.stdout))
            self.assertIn("not used", disabled.stderr)
            self.assertFalse((root / "model-must-not-exist").exists())


if __name__ == "__main__":
    unittest.main()
