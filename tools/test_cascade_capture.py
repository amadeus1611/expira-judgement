"""Offline integrity tests, using fresh engineering fixtures and fake processes."""
from copy import deepcopy
from fractions import Fraction
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

import cascade_capture as capture

ROWS = [dict(claim="The violet lamp glows.", quote="The violet lamp glows.", label="supports"),
        dict(claim="The rack holds 8 trays.", quote="The rack holds 7 trays.", label="contradicts"),
        dict(claim="The crane moves swiftly.", quote="The crane has moved.", label="unsure")]


def fixture(rows=ROWS):
    lines = [json.dumps(row).encode() + b"\n" for row in rows]
    identities = [dict(case_id="group-" + str(i), row_idx=10 + i,
                       row_sha256=hashlib.sha256(line).hexdigest()) for i, line in enumerate(lines)]
    return b"".join(lines), b"\n".join(json.dumps(i).encode() for i in identities) + b"\n"


def findings(cases, verdicts=("backed", "mismatch", "unsure")):
    return [dict(claim_id=c["claim_id"], verdict=v, layer="deterministic",
                 confidence=0 if v == "unsure" else 1, reason="Engineering fixture.")
            for c, v in zip(cases, verdicts)]


class SchemaTests(unittest.TestCase):
    def setUp(self):
        self.data, self.identities = fixture()
        self.cases = capture.inputs(self.data, self.identities)

    def test_raw_bytes_and_opaque_row_identity(self):
        self.assertEqual(len({c["claim_id"] for c in self.cases}), 3)
        self.assertNotEqual(self.cases[0]["claim_id"], self.cases[0]["case_id"])
        for data, ids in ((self.data.replace(b'"supports"', b'"unsure"'), self.identities),
                          (self.data.rstrip(b"\n"), self.identities),
                          (self.data, b"\n".join(reversed(self.identities.splitlines())))):
            with self.assertRaises(ValueError):
                capture.inputs(data, ids)
        changed = capture.inputs(*fixture([dict(ROWS[0], quote="Other quotation."), *ROWS[1:]]))
        self.assertNotEqual(changed[0]["claim_id"], self.cases[0]["claim_id"])

    def test_invalid_inputs_and_duplicate_ids(self):
        for row in (dict(ROWS[0], label="SUPPORTS"), dict(ROWS[0], claim=""),
                    dict(ROWS[0], quote=None), dict(ROWS[0], extra=True)):
            with self.subTest(row=row), self.assertRaises(ValueError):
                capture.inputs(*fixture([row]))
        companions = [json.loads(line) for line in self.identities.splitlines()]
        for change in (dict(row_idx=True), dict(row_idx=-1), dict(row_idx=2**63), dict(row_idx=11),
                       dict(case_id=""), dict(row_sha256="wrong"), dict(extra=True)):
            bad = deepcopy(companions)
            bad[0].update(change)
            with self.subTest(change=change), self.assertRaises(ValueError):
                capture.inputs(self.data, b"\n".join(json.dumps(i).encode() for i in bad))
        for data, ids in ((b"", b""), (self.data, self.identities.splitlines()[0]),
                          (self.data.replace(b'"label":', b'"label":"unsure","label":', 1), self.identities)):
            with self.assertRaises(ValueError):
                capture.inputs(data, ids)

    def test_empty_quote_retained_and_no_reference_label_in_request(self):
        cases = capture.inputs(*fixture([dict(ROWS[0], quote="")]))
        request = capture.request_for(cases, "2026-10-11", "fixture")
        self.assertEqual(request, [dict(id=cases[0]["claim_id"], text=ROWS[0]["claim"],
                                      evidence=[dict(source="fixture", date="2026-10-11", quote="")])])

    def test_valid_exit_one_and_exit_zero_only_all_backed(self):
        expected = findings(self.cases)
        self.assertEqual(capture.findings_from(json.dumps(expected), 1, self.cases), expected)
        all_backed = findings(self.cases, ("backed",) * 3)
        self.assertEqual(capture.findings_from(json.dumps(all_backed), 0, self.cases), all_backed)
        for result, code in ((expected, 0), (all_backed, 1), (expected, 2), (all_backed, -9)):
            with self.subTest(code=code), self.assertRaises(ValueError):
                capture.findings_from(json.dumps(result), code, self.cases)

    def test_complete_ordered_one_to_one_findings(self):
        expected = findings(self.cases)
        for bad in (expected[:-1], expected + [expected[0]], list(reversed(expected)),
                    [expected[0], expected[0], expected[2]],
                    [dict(expected[0], claim_id="foreign"), *expected[1:]], {}, None):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                capture.findings_from(json.dumps(bad), 1, self.cases)

    def test_schema_impossible_routes_and_confidence_types(self):
        expected = findings(self.cases)
        for change in (dict(layer="local_judge"), dict(layer="claude"), dict(verdict="supported"),
                       dict(verdict="contradicts"), dict(confidence=True), dict(confidence=0.9),
                       dict(confidence=math.inf), dict(confidence=math.nan), dict(reason=""),
                       dict(reason=[]), dict(extra="ignored")):
            bad = deepcopy(expected)
            bad[0].update(change)
            with self.subTest(change=change), self.assertRaises(ValueError):
                capture.findings_from(json.dumps(bad), 1, self.cases)
        with self.assertRaises(ValueError):
            capture.findings_from(json.dumps(expected).replace('"reason":', '"reason":"bad","reason":', 1), 1, self.cases)
        bad = deepcopy(expected)
        del bad[0]["reason"]
        with self.assertRaises(ValueError):
            capture.findings_from(json.dumps(bad), 1, self.cases)

    def test_metrics_and_failed_prefix_denominator(self):
        rows = [dict(ROWS[0], label=l) for l in ("supports", "contradicts", "unsure", "supports", "unsure")]
        cases = capture.inputs(*fixture(rows))
        result = capture.summary(cases, findings(cases, ("backed", "mismatch", "unbacked", "stale", "unsure")), None)
        self.assertEqual((result["stated"], result["stated_right"], result["blocks"], result["block_right"]), (1, 1, 3, 2))
        self.assertEqual(result["abstentions"], 1)
        self.assertEqual(result["pre_escalation_coverage"], 4 / 5)
        self.assertEqual(result["verdict_reference_table"]["stale"]["supports"], 1)
        failed = capture.summary(cases, findings(cases), "timeout")
        self.assertEqual((failed["cases"], failed["operational_failures"], failed["completed_cases"]), (5, 5, 0))
        self.assertEqual(failed["pre_escalation_coverage"], 0)
        self.assertIsNone(failed["nominal_pass_lower_bound"])
        self.assertIsNone(failed["nominal_block_lower_bound"])

    def test_wilson_independent_rational_score_inversion(self):
        self.assertEqual(capture.wilson(0, 0), 0)
        for n in (1, 10, 25, 100, 380, 381, 562, 563, 1000):
            self.assertEqual(capture.wilson(0, n), 0)
            self.assertAlmostEqual(capture.wilson(n, n), n / (n + 3.8416), places=14)
            for k in range(n + 1):
                lower = capture.wilson(k, n)
                for p0 in (Fraction(99, 100), Fraction(95, 100)):
                    delta = k - p0 * n
                    passes = delta >= 0 and delta**2 >= Fraction(196, 100)**2 * p0 * (1 - p0) * n
                    self.assertEqual(lower >= float(p0), passes, (n, k, p0))
                if k:
                    self.assertGreaterEqual(lower, capture.wilson(k - 1, n))
        self.assertLess(capture.wilson(380, 380), 0.99)
        self.assertGreater(capture.wilson(381, 381), 0.99)
        self.assertLess(capture.wilson(561, 562), 0.99)
        self.assertGreater(capture.wilson(562, 563), 0.99)
        for right, total in ((-1, 1), (2, 1), (True, 1), (0, -1), (0.0, 1)):
            with self.assertRaises(ValueError):
                capture.wilson(right, total)


class ProcessTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="ej-capture-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        (self.repo / "source.txt").write_text("Tracked source bytes.")
        for args in (("init", "-q"), ("add", "source.txt"),
                     ("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "fixture")):
            subprocess.run(["git", "-C", str(self.repo), *args], check=True, capture_output=True)
        data, identities = fixture()
        self.data, self.ids, self.sampling = (self.root / name for name in ("data.jsonl", "ids.jsonl", "sampling.json"))
        self.data.write_bytes(data)
        self.ids.write_bytes(identities)
        self.sampling.write_text('{"population":"authored engineering fixtures","independence":"unattested"}')
        self.program = self.root / "fake-ej"

    def fake(self, suffix="", code=1):
        self.program.write_text("#!" + sys.executable + "\n" + '''
import json, sys
from pathlib import Path
request = json.loads(Path(sys.argv[2]).read_text())
assert sys.argv[1] == "check" and sys.argv[3:] == ["--today", "2026-10-11"]
for row in request:
    assert set(row) == {"id", "text", "evidence"}
    assert set(row["evidence"][0]) == {"source", "date", "quote"}
result = [dict(claim_id=r["id"], verdict=v, layer="deterministic",
               confidence=0 if v == "unsure" else 1, reason="Process fixture.")
          for r,v in zip(request, ("backed", "mismatch", "unsure"))]
''' + suffix + "\nprint(json.dumps(result))\nraise SystemExit(" + str(code) + ")\n")
        self.program.chmod(0o700)

    def run_capture(self, name="run", **options):
        return capture.capture(data_path=self.data, identities_path=self.ids, sampling_path=self.sampling,
                               ej=self.program, repo=self.repo, today="2026-10-11", out=self.root / name, **options)

    def test_process_label_free_request_and_atomic_report(self):
        self.fake()
        report = self.run_capture()
        self.assertTrue(report["measurement_complete"])
        self.assertFalse(report["certified"])
        self.assertEqual(report["subscription_attempts"], 0)
        self.assertIsNone(report["cost_usd"])
        self.assertIsNone(report["tokens"])
        self.assertEqual(report["summary"]["cases"], 3)
        self.assertEqual(report["summary"]["pre_escalation_coverage"], 2 / 3)
        self.assertEqual([r["finding"]["verdict"] for r in report["records"]], ["backed", "mismatch", "unsure"])
        manifest = json.loads((self.root / "run/manifest.json").read_text())
        self.assertEqual(manifest["data_sha256"], hashlib.sha256(self.data.read_bytes()).hexdigest())
        self.assertEqual(manifest["executable_sha256"], hashlib.sha256(self.program.read_bytes()).hexdigest())
        self.assertTrue(manifest["source_state"]["worktree_clean"])
        self.assertFalse(manifest["sampling_attested"])
        self.assertEqual(manifest["local_models"], [])
        self.assertIsNone(manifest["escalation"])
        self.assertEqual(manifest["source_state"]["binary_source_relationship"], "unattested")
        self.assertEqual(manifest["runner_sha256"], hashlib.sha256(Path(capture.__file__).read_bytes()).hexdigest())
        self.assertEqual(manifest["helper_sha256"], hashlib.sha256(Path(capture.__file__).with_name("codex_adapter.py").read_bytes()).hexdigest())
        self.assertEqual(json.loads((self.root / "run/report.json").read_text()), report)
        self.assertFalse((self.root / "run/.report.tmp").exists())
        for metric in ("max_ece", "max_p95_ms", "stress_wrong_passes", "exact_confirmation"):
            self.assertIn(metric, report["unmeasured"])

    def test_failures_retain_all_rows_without_scoring_a_prefix(self):
        for i, (suffix, code) in enumerate((("", 2), ("result = result[:1]", 1), ("result.reverse()", 1), ("", 0),
                                          ('print("not JSON"); raise SystemExit(1)', 1),
                                          ('sys.stdout.buffer.write(b"\\xff"); raise SystemExit(1)', 1),
                                          ('print("x" * (1024*1024+1)); raise SystemExit(1)', 1),
                                          ('Path(sys.argv[2]).write_text("[]")', 1))):
            self.fake(suffix, code)
            report = self.run_capture(str(i))
            with self.subTest(suffix=suffix, code=code):
                self.assertFalse(report["measurement_complete"])
                self.assertFalse(report["certified"])
                self.assertEqual(report["summary"]["operational_failures"], 3)
                self.assertIsNone(report["summary"]["nominal_pass_lower_bound"])
                self.assertEqual(len(report["records"]), 3)
                self.assertTrue(all(r["finding"] is None and r["status"] == "operational_failure" for r in report["records"]))

    def test_existing_directory_rejected_including_incomplete_manifest(self):
        self.fake()
        self.run_capture()
        before = (self.root / "run/report.json").read_bytes()
        with self.assertRaises(FileExistsError):
            self.run_capture()
        self.assertEqual((self.root / "run/report.json").read_bytes(), before)
        incomplete = self.root / "incomplete"
        incomplete.mkdir()
        (incomplete / "manifest.json").write_text("{}")
        with self.assertRaises(FileExistsError):
            self.run_capture("incomplete")
        self.assertFalse((incomplete / "report.json").exists())

    def test_preflight_never_spawns_invalid_input(self):
        self.fake()
        self.ids.write_text("{}")
        with patch.object(capture, "run_bounded", side_effect=AssertionError("must not spawn")), self.assertRaises(ValueError):
            self.run_capture()
        self.assertFalse((self.root / "run").exists())

    def test_repository_root_and_configuration_preflight(self):
        nested = self.repo / "nested"
        nested.mkdir()
        with self.assertRaises(ValueError):
            capture.source_state(nested)
        self.fake()
        for options in (dict(source=""), dict(max_age=True), dict(max_age=-1), dict(timeout=0),
                        dict(timeout=301), dict(timeout=float("nan"))):
            with self.subTest(options=options), self.assertRaises(ValueError):
                self.run_capture(**options)
        self.assertFalse((self.root / "run").exists())

    def test_interrupt_leaves_manifest_without_final_report(self):
        self.fake()
        with patch.object(capture, "run_bounded", side_effect=KeyboardInterrupt), self.assertRaises(KeyboardInterrupt):
            self.run_capture()
        self.assertTrue((self.root / "run/manifest.json").exists())
        self.assertFalse((self.root / "run/report.json").exists())

    def test_timeout_kills_descendants_and_retains_all_input(self):
        marker = self.root / "descendant-survived"
        pid_file = self.root / "descendant-pid"
        child = "import time;from pathlib import Path;time.sleep(.6);Path(" + repr(str(marker)) + ").touch()"
        self.fake("import subprocess,time\nchild=subprocess.Popen([sys.executable,'-c'," + repr(child) + "])\n"
                  "Path(" + repr(str(pid_file)) + ").write_text(str(child.pid))\ntime.sleep(10)")
        started = time.monotonic()
        report = self.run_capture(timeout=0.15)
        self.assertLess(time.monotonic() - started, 3)
        self.assertEqual(report["error"], "timeout")
        self.assertEqual(report["summary"]["operational_failures"], 3)
        self.assertTrue(pid_file.exists(), "fixture must spawn a descendant before the timeout")
        time.sleep(0.7)
        self.assertFalse(marker.exists())
        status = Path("/proc") / pid_file.read_text() / "stat"
        if status.exists():
            self.assertEqual(status.read_text().split()[2], "Z", "descendant must be dead, not merely slow")

    def test_each_identity_component_changes_run_id(self):
        self.fake()
        first = self.run_capture()["run_id"]
        (self.repo / "source.txt").write_text("Changed tracked source.")
        second = self.run_capture("source")["run_id"]
        self.assertNotEqual(first, second)
        self.sampling.write_text(self.sampling.read_text() + "\n")
        third = self.run_capture("sampling")["run_id"]
        self.assertNotEqual(second, third)
        self.program.write_text(self.program.read_text() + "\n# executable change\n")
        fourth = self.run_capture("binary")["run_id"]
        self.assertNotEqual(third, fourth)
        data, ids = fixture([dict(ROWS[0], label="unsure"), *ROWS[1:]])
        self.data.write_bytes(data)
        self.ids.write_bytes(ids)
        fifth = self.run_capture("data")["run_id"]
        self.assertNotEqual(fourth, fifth)
        identities = [json.loads(line) for line in self.ids.read_text().splitlines()]
        identities[0]["case_id"] = "new-group"
        self.ids.write_text("\n".join(json.dumps(i) for i in identities))
        self.assertNotEqual(fifth, self.run_capture("groups")["run_id"])

    def test_public_cli_and_existing_output_status(self):
        self.fake()
        command = [sys.executable, "-I", capture.__file__, "--data", str(self.data), "--identities", str(self.ids),
                   "--sampling", str(self.sampling), "--ej", str(self.program), "--repo", str(self.repo),
                   "--today", "2026-10-11", "--out", str(self.root / "public")]
        done = subprocess.run(command, capture_output=True, text=True, timeout=10)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertFalse(json.loads(done.stdout)["certified"])
        refused = subprocess.run(command, capture_output=True, text=True, timeout=10)
        self.assertEqual(refused.returncode, 2)
        self.assertNotIn("Traceback", refused.stderr)


if __name__ == "__main__":
    unittest.main()
