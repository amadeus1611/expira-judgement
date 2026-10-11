"""Protocol and process fault tests; no subscription access and no labelled data."""
import json
import math
import os
from pathlib import Path
import random
import subprocess
import sys
import tempfile
import time
import unittest

import codex_adapter as adapter


QUOTE = "The gate is locked."
REPLY = dict(ruling="supports", confidence=0.9, evidence_span=QUOTE, reason="The quote states it.")
USAGE = dict(input_tokens=100, cached_input_tokens=30, output_tokens=20,
             reasoning_output_tokens=10, cache_write_input_tokens=5)


def events(reply=REPLY):
    return [dict(type="thread.started", thread_id="test"), dict(type="turn.started"),
            dict(type="item.completed", item=dict(id="message", type="agent_message", text=json.dumps(reply))),
            dict(type="turn.completed", usage=USAGE.copy())]


def decode(stream=None, reply=REPLY, code=0):
    stream = events(reply) if stream is None else stream
    return adapter.decode("\n".join(json.dumps(event) for event in stream), json.dumps(reply), code, QUOTE)


class ProtocolTests(unittest.TestCase):
    def assert_rejected(self, result):
        self.assertFalse(result["acted"])
        self.assertEqual(result["ruling"], "unsure")
        self.assertEqual(result["confidence"], 0)
        self.assertIsNotNone(result["error"])
        self.assertIsNone(result["cost_usd"])
        self.assertFalse(result["isolation_validated"])

    def test_valid_rulings_and_production_confidence_boundary(self):
        # Independent binary32 values: predecessor of 0.8, then its rounding midpoint.
        # The midpoint rounds down to the even significand; the next double rounds up.
        boundary = [(0, False), (0.79, False), (0.7999999523162842, False),
                    (0.7999999821186066, False),
                    (math.nextafter(0.7999999821186066, 1), True),
                    (math.nextafter(0.8, 0), True), (0.8, True), (1, True)]
        for ruling in ("supports", "contradicts", "unsure"):
            for confidence, acts in boundary:
                with self.subTest(ruling=ruling, confidence=confidence):
                    result = decode(reply=dict(REPLY, ruling=ruling, confidence=confidence))
                    self.assertIsNone(result["error"])
                    self.assertEqual(result["acted"], ruling != "unsure" and acts)
                    self.assertEqual(result["tokens"], USAGE)
                    self.assertIsNone(result["cost_usd"])
                    self.assertFalse(result["isolation_validated"])

    def test_schema_faults_and_usage_preserved(self):
        faults = [dict(REPLY, confidence=x) for x in (-1, 1.1, True, "0.9", None, float("inf"), float("nan"))]
        faults += [dict(REPLY, **{key: value}) for key, value in (
            ("ruling", "yes"), ("ruling", None), ("ruling", []), ("reason", " "),
            ("reason", None), ("evidence_span", None), ("extra", "ignored?"))]
        for key in REPLY:
            reply = REPLY.copy()
            del reply[key]
            faults.append(reply)
        for reply in faults + [[], None, "supports"]:
            with self.subTest(reply=reply):
                result = decode(reply=reply)
                self.assert_rejected(result)
                self.assertEqual(result["tokens"], USAGE)

    def test_citation_faults_and_normalisation(self):
        for span in ("", "....", "The gate is unlocked.", "locked. The guard left"):
            result = decode(reply=dict(REPLY, evidence_span=span))
            self.assert_rejected(result)
            self.assertFalse(result["span_ok"])
        for span in ('"THE   GATE\nis locked."', "…gate is locked…", "'locked.'"):
            self.assertTrue(decode(reply=dict(REPLY, evidence_span=span))["acted"])
        self.assertIsNone(decode(reply=dict(REPLY, ruling="unsure", evidence_span=""))["error"])

    def test_nonzero_exit_cannot_decide_even_with_a_complete_reply(self):
        for code in (1, 2, -9):
            result = decode(code=code)
            self.assert_rejected(result)
            self.assertEqual(result["tokens"], USAGE)

    def test_tools_errors_and_unknown_events_reject_at_every_item_phase(self):
        for phase in ("item.started", "item.updated", "item.completed"):
            for kind in ("command_execution", "mcp_tool_call", "collab_tool_call", "web_search",
                         "file_change", "todo_list", "error", "future_tool", None):
                stream = events()
                stream.insert(2, dict(type=phase, item=dict(id="tool", type=kind, text="")))
                self.assert_rejected(decode(stream))
        for kind in ("error", "turn.failed", "future.event", None):
            stream = events()
            stream.insert(2, dict(type=kind))
            self.assert_rejected(decode(stream))

    def test_missing_duplicate_reordered_and_trailing_events(self):
        original = events()
        for index in range(len(original)):
            for stream in (original[:index] + original[index + 1:],
                           original[:index] + [original[index]] + original[index:]):
                self.assert_rejected(decode(stream))
        self.assert_rejected(decode(original + original))
        self.assert_rejected(decode(original + [dict(type="item.completed", item={})]))
        # Exhaust all 24 event orders; exactly the protocol order can settle a claim.
        import itertools
        for permutation in itertools.permutations(original):
            if list(permutation) != original:
                self.assert_rejected(decode(list(permutation)))

    def test_final_file_must_match_completed_message(self):
        self.assert_rejected(decode(events(), dict(REPLY, ruling="contradicts")))
        # Python value equality alone considers True == 1; JSON types must still match.
        self.assert_rejected(decode(events(dict(REPLY, confidence=True)), dict(REPLY, confidence=1)))
        for final in ("", "{}", "[]", json.dumps(REPLY) + "junk"):
            stream = "\n".join(json.dumps(e) for e in events())
            self.assert_rejected(adapter.decode(stream, final, 0, QUOTE))

    def test_json_faults_including_nested_duplicates_and_nonfinite_numbers(self):
        for text in ('{"a":1,"a":2}', '{"x":{"a":1,"a":2}}', "NaN", "Infinity", "-Infinity"):
            with self.assertRaises(ValueError):
                adapter.strict_json(text)
        valid = "\n".join(json.dumps(e) for e in events())
        for stdout in ("null", "[]", "garbage", valid[:-1], valid + "\n[]", valid + "\nNaN"):
            self.assert_rejected(adapter.decode(stdout, json.dumps(REPLY), 0, QUOTE))
        deep = '[' * 2000 + '0' + ']' * 2000
        self.assert_rejected(adapter.decode(deep, json.dumps(REPLY), 0, QUOTE))
        duplicate = json.dumps(REPLY).replace('"confidence": 0.9', '"confidence": 0.9, "confidence": 1')
        stream = events()
        stream[2]["item"]["text"] = duplicate
        self.assert_rejected(adapter.decode("\n".join(map(json.dumps, stream)), duplicate, 0, QUOTE))

    def test_usage_is_unknown_when_invalid_or_ambiguous(self):
        faults = [None, {}, dict(USAGE, input_tokens=True), dict(USAGE, output_tokens=-1),
                  dict(USAGE, cached_input_tokens=101), dict(USAGE, reasoning_output_tokens=21),
                  dict(USAGE, cache_write_input_tokens="5"), dict(USAGE, input_tokens=2**63)]
        # A negative cache-write total is not also rejected by the input/output relationships.
        faults.append(dict(USAGE, cache_write_input_tokens=-1))
        for usage in faults:
            stream = events()
            stream[-1]["usage"] = usage
            result = decode(stream)
            self.assert_rejected(result)
            self.assertIsNone(result["tokens"])
        stream = events()
        del stream[-1]["usage"]["cache_write_input_tokens"]
        self.assertEqual(decode(stream)["tokens"]["cache_write_input_tokens"], 0)

    def test_nontext_items_and_malformed_structure(self):
        for item in ({}, None, [], dict(id="", type="agent_message", text="x"),
                     dict(id="message", type="reasoning", text=None)):
            stream = events()
            stream.insert(2, dict(type="item.started", item=item))
            self.assert_rejected(decode(stream))
        stream = events()
        stream[0]["thread_id"] = None
        self.assert_rejected(decode(stream))

    def test_item_lifecycle_rejects_duplicates_and_unfinished_items(self):
        item = dict(id="reason", type="reasoning", text="Thinking")
        started = dict(type="item.started", item=item)
        updated = dict(type="item.updated", item=item)
        completed = dict(type="item.completed", item=item)
        for extra in ([started], [updated], [started, started, completed],
                      [completed, completed], [completed, started]):
            stream = events()
            stream[2:2] = extra
            self.assert_rejected(decode(stream))
        stream = events()
        stream[2:2] = [started, updated, completed]
        self.assertTrue(decode(stream)["acted"])

    def test_seeded_fault_sequences_never_gain_a_decision(self):
        rng = random.Random(20261011)
        for _ in range(256):
            stream = events()
            invalid = dict(type=rng.choice(("error", "turn.failed", "future.event")))
            stream.insert(rng.randrange(len(stream) + 1), invalid)
            self.assert_rejected(decode(stream))


@unittest.skipUnless(os.name == "posix", "adapter requires POSIX process groups")
class ProcessTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def fake(self, body="", *, version=adapter.VERSION):
        path = self.root / "fake-codex"
        prefix = ("#!/usr/bin/env python3\nimport json,sys,pathlib,os,time\n"
                  "if '--version' in sys.argv:\n"
                  f" print({version!r});sys.exit(0)\n")
        path.write_text(prefix + body, encoding="utf-8")
        path.chmod(0o700)
        return str(path)

    def ask(self, cli, **kwargs):
        return adapter.ask(dict(claim=QUOTE, quote=QUOTE, label="SECRET_GOLD_LABEL"),
                           model="test-model", state_dir=self.root / "state", cli=cli, **kwargs)

    def test_real_process_success_prompt_and_supported_isolation_flags(self):
        capture = self.root / "capture.json"
        cli = self.fake(f"pathlib.Path({str(capture)!r}).write_text(json.dumps(dict(args=sys.argv,prompt=sys.stdin.read(),cwd=os.getcwd())))\n"
                        f"pathlib.Path(sys.argv[sys.argv.index('--output-last-message')+1]).write_text({json.dumps(REPLY)!r})\n"
                        f"print({chr(10).join(map(json.dumps, events()))!r})\n")
        result = self.ask(cli)
        self.assertTrue(result["acted"])
        self.assertTrue(result["inference_attempted"])
        self.assertEqual(result["cli_version"], adapter.VERSION)
        captured = json.loads(capture.read_text())
        self.assertNotIn("SECRET_GOLD_LABEL", captured["prompt"])
        self.assertEqual(json.loads(captured["prompt"].split("Input JSON:\n")[1]), dict(claim=QUOTE, quote=QUOTE))
        self.assertNotEqual(captured["cwd"], str(self.root))
        for option in ("--ignore-user-config", "--ignore-rules", "--ephemeral", "read-only", "--json", "never"):
            self.assertIn(option, captured["args"])
        self.assertIn("mcp_servers={}", captured["args"])
        self.assertIn("shell_tool", captured["args"])
        self.assertEqual(result["prompt_sha256"], adapter.digest(captured["prompt"]))

    def test_version_failure_never_starts_inference(self):
        marker = self.root / "inference"
        result = self.ask(self.fake(f"pathlib.Path({str(marker)!r}).touch()\n", version="codex-cli 0.160.0"))
        self.assertFalse(result["inference_attempted"])
        self.assertFalse(marker.exists())
        self.assertEqual(result["error"], "version_preflight_failed")

    def test_public_cli_reads_multiline_json_and_returns_failure_status(self):
        for body, code in ((f"pathlib.Path(sys.argv[sys.argv.index('--output-last-message')+1]).write_text({json.dumps(REPLY)!r})\n"
                            f"print({chr(10).join(map(json.dumps, events()))!r})\n", 0),
                           ("sys.exit(1)\n", 1)):
            cli = self.fake(body)
            process = subprocess.run(
                [sys.executable, "-I", adapter.__file__, "--model", "test-model",
                 "--state-dir", str(self.root / "state"), "--cli", cli],
                input=json.dumps(dict(claim=QUOTE, quote=QUOTE), indent=2),
                capture_output=True, text=True, timeout=5,
            )
            self.assertEqual(process.returncode, code)
            result = json.loads(process.stdout)
            self.assertEqual(result["acted"], code == 0)
            self.assertIsNone(result["cost_usd"])

    def test_nonzero_process_with_plausible_output_and_missing_final(self):
        for body in (f"print({chr(10).join(map(json.dumps, events()))!r});sys.exit(1)\n",
                     f"print({chr(10).join(map(json.dumps, events()))!r})\n"):
            result = self.ask(self.fake(body))
            self.assertFalse(result["acted"])
            self.assertIsNotNone(result["error"])
            self.assertEqual(result["tokens"], USAGE)

    def test_timeout_kills_descendants_and_is_bounded(self):
        pid_file = self.root / "descendant"
        cli = self.fake("pid=os.fork()\nif pid == 0:\n time.sleep(30);sys.exit(0)\n"
                        f"pathlib.Path({str(pid_file)!r}).write_text(str(pid))\n"
                        "time.sleep(30)\n")
        started = time.monotonic()
        result = self.ask(cli, timeout=0.5)
        self.assertLess(time.monotonic() - started, 5)
        self.assertEqual(result["error"], "timeout")
        self.assertFalse(result["acted"])
        self.assertTrue(pid_file.exists())
        pid = int(pid_file.read_text())
        # A killed orphan may remain a zombie until PID 1 reaps it; it cannot keep running.
        for _ in range(100):
            stat = Path(f"/proc/{pid}/stat")
            if not stat.exists() or stat.read_text().split()[2] == "Z":
                break
            time.sleep(0.01)
        else:
            self.fail("descendant still running")

    def test_output_and_encoding_limits(self):
        for body, error in ((f"print('x'*{adapter.MAX_OUTPUT + 1})\n", "output_limit"),
                            ("os.write(1,b'\\xff')\n", "invalid_encoding")):
            result = self.ask(self.fake(body))
            self.assertEqual(result["error"], error)
            self.assertFalse(result["acted"])

    def test_diagnostics_are_classified_without_echoing_provider_text(self):
        for text, category in (("401 Unauthorized: PRIVATE_AUTH_TEXT", "authentication_failure"),
                               ("Error parsing config: PRIVATE_AUTH_TEXT", "configuration_failure")):
            result = self.ask(self.fake(f"print({text!r},file=sys.stderr);sys.exit(1)\n"))
            self.assertEqual(result["diagnostic_category"], category)
            self.assertNotIn("PRIVATE_AUTH_TEXT", json.dumps(result))

    def test_invalid_final_encoding_keeps_completed_usage(self):
        cli = self.fake("pathlib.Path(sys.argv[sys.argv.index('--output-last-message')+1]).write_bytes(b'\\xff')\n"
                        f"print({chr(10).join(map(json.dumps, events()))!r})\n")
        result = self.ask(cli)
        self.assertEqual(result["error"], "invalid_final_encoding")
        self.assertEqual(result["tokens"], USAGE)
        self.assertFalse(result["acted"])

    def test_timeout_after_reported_completion_retains_usage(self):
        cli = self.fake("pathlib.Path(sys.argv[sys.argv.index('--output-last-message')+1]).write_text("
                        f"{json.dumps(REPLY)!r})\n"
                        f"print({chr(10).join(map(json.dumps, events()))!r},flush=True)\n"
                        "time.sleep(30)\n")
        result = self.ask(cli, timeout=0.5)
        self.assertEqual(result["error"], "timeout")
        self.assertEqual(result["tokens"], USAGE)
        self.assertFalse(result["acted"])

    def test_missing_executable_and_bad_inputs(self):
        result = self.ask(str(self.root / "missing"))
        self.assertEqual(result["error"], "process_unavailable")
        self.assertFalse(result["inference_attempted"])
        for options in (dict(timeout=0), dict(timeout=301), dict(timeout=float("nan")), dict(effort="unbounded")):
            with self.assertRaises(ValueError):
                self.ask("unused", **options)
        for case in (dict(claim="", quote=QUOTE), dict(claim=QUOTE, quote=None)):
            with self.assertRaises(ValueError):
                adapter.prompt_for(case)


if __name__ == "__main__":
    unittest.main()
