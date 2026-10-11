#!/usr/bin/env python3
"""Require offline tests to catch this declared family of 15 protocol faults.

This is targeted fault injection, not exhaustive Python mutation coverage. Each
variant runs in a disposable directory against fake CLI processes, never Codex.
"""
import json
from pathlib import Path
import subprocess
import sys
import tempfile

FAULTS = {
    "accept_failed_process": ("if returncode != 0:", "if False:"),
    "accept_mismatched_final": (
        "if canonical(reply) != canonical(strict_json(messages[0])):", "if False:"),
    "accept_unknown_reply_field": ('set(reply) != set(SCHEMA["required"])', "False"),
    "weaken_confidence_bar": ("f32(confidence) >= f32(0.8)", "f32(confidence) >= f32(0.7)"),
    "reject_equal_confidence_bar": ("f32(confidence) >= f32(0.8)", "f32(confidence) > f32(0.8)"),
    "use_double_confidence_gate": ("f32(confidence) >= f32(0.8)", "confidence >= 0.8"),
    "accept_false_citation": ("return bool(span) and span in norm(quote)", "return True"),
    "report_unknown_cost_zero": ('"cost_usd": None', '"cost_usd": 0'),
    "discard_usage_on_rejection": ('result["tokens"] = usage_from(events)', 'result["tokens"] = None'),
    "accept_unfinished_item": ('stage != "done" or active or', 'stage != "done" or'),
    "accept_nonfinite_json": ("parse_constant=constant", "parse_constant=None"),
    "accept_negative_token_counts": ("not 0 <= v <= (2**63 - 1)", "v > (2**63 - 1)"),
    "accept_empty_reason": ('or not reply["reason"].strip()', ""),
    "accept_unsupported_cli_version": ("version.strip() != VERSION", "False"),
    "accept_emitted_tool": ('("agent_message", "reasoning")', '("agent_message", "reasoning", "mcp_tool_call")'),
}


def main():
    root = Path(__file__).resolve().parent
    source = (root / "codex_adapter.py").read_text(encoding="utf-8")
    tests = (root / "test_codex_adapter.py").read_text(encoding="utf-8")
    failed = False
    for name, (before, after) in FAULTS.items():
        if source.count(before) != 1:
            raise SystemExit(f"{name}: expected exactly one mutation site; review the fault family")
        with tempfile.TemporaryDirectory(prefix="ej-codex-fault-") as directory:
            path = Path(directory)
            (path / "codex_adapter.py").write_text(source.replace(before, after), encoding="utf-8")
            (path / "test_codex_adapter.py").write_text(tests, encoding="utf-8")
            try:
                process = subprocess.run(
                    [sys.executable, "-I", "-m", "unittest", "discover", "-s", directory, "-p", "test_*.py"],
                    capture_output=True, text=True, timeout=15,
                )
                status = ("caught" if process.returncode == 1 and "FAILED (" in process.stderr
                          else "missed" if process.returncode == 0 else "unviable")
            except subprocess.TimeoutExpired:
                status = "timeout"
        print(json.dumps(dict(fault=name, status=status)), flush=True)
        failed |= status != "caught"
    return int(failed)


if __name__ == "__main__":
    raise SystemExit(main())
