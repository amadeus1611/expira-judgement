#!/usr/bin/env python3
"""Require offline tests to catch 20 declared capture-integrity faults.

This is targeted fault injection, not exhaustive Python mutation coverage. Every
variant and the unmodified baseline run in a disposable directory against fake
processes. No real model, subscription, test dataset or repository edit is used.
"""
import json
from pathlib import Path
import subprocess
import sys
import tempfile


FAULTS = {
    "ignore_identity_row_bytes": ('identity["row_sha256"] != sha(line)', "False"),
    "accept_duplicate_row_identity": ('identity["row_idx"] in seen', "False"),
    "accept_unknown_data_field": ('set(row) != {"claim", "quote", "label"}', "False"),
    "accept_unknown_finding_field": (
        'set(finding) != {"claim_id", "verdict", "layer", "confidence", "reason"}', "False"),
    "accept_foreign_or_reordered_finding": ('finding["claim_id"] != case["claim_id"]', "False"),
    "accept_inconsistent_exit": ("if code != expected_code:", "if False:"),
    "reject_valid_nonpass_exit": ("if code not in (0, 1):", "if code != 0:"),
    "accept_unconfigured_layer": ('finding["layer"] != "deterministic"', "False"),
    "accept_boolean_confidence": ('type(finding["confidence"]) not in (int, float)', "False"),
    "accept_arbitrary_deterministic_confidence": (
        'finding["confidence"] != (0 if finding["verdict"] == "unsure" else 1)', "False"),
    "leak_reference_label_to_request": (
        'dict(id=c["claim_id"], text=c["claim"], evidence=[',
        'dict(id=c["claim_id"], text=c["claim"], label=c["label"], evidence=['),
    "count_abstentions_as_blocks": (
        'blocks = sum(sum(table[v].values()) for v in ("mismatch", "unbacked", "stale"))',
        'blocks = sum(sum(table[v].values()) for v in ("mismatch", "unbacked", "stale", "unsure"))'),
    "count_supported_reference_as_correct_block": (
        'for l in ("contradicts", "unsure"))',
        'for l in ("supports", "contradicts", "unsure"))'),
    "discard_failed_row_count": (
        "operational_failures=len(cases) if failure is not None else 0", "operational_failures=0"),
    "score_failed_successful_prefix": (
        "if failure is None:\n        for case, finding in zip(cases, findings):",
        "if True:\n        for case, finding in zip(cases, findings):"),
    "remove_abstentions_from_coverage_denominator": (
        "pre_escalation_coverage=(stated + blocks) / len(cases)",
        "pre_escalation_coverage=(stated + blocks) / (stated + blocks or 1)"),
    "weaken_wilson_confidence": (
        "p, z2 = right / total, 1.96**2\n    return (p + z2 / (2 * total) - 1.96 * math.sqrt(",
        "p, z2 = right / total, 1.64**2\n    return (p + z2 / (2 * total) - 1.64 * math.sqrt("),
    "report_unknown_cost_zero": (
        "subscription_attempts=0, tokens=None, cost_usd=None,",
        "subscription_attempts=0, tokens=None, cost_usd=0,"),
    "certify_diagnostic_capture": ("protocol=PROTOCOL, certified=False,", "protocol=PROTOCOL, certified=True,"),
    "overwrite_existing_run": (
        "output.mkdir(parents=True, exist_ok=False)",
        "shutil.rmtree(output, ignore_errors=True); output.mkdir(parents=True, exist_ok=False)"),
}


def trial(source, helper, tests):
    """Keep source imports and process tests isolated from the working checkout."""
    with tempfile.TemporaryDirectory(prefix="ej-cascade-fault-") as directory:
        path = Path(directory)
        for name, text in (("cascade_capture.py", source), ("codex_adapter.py", helper),
                           ("test_cascade_capture.py", tests)):
            (path / name).write_text(text, encoding="utf-8")
        try:
            process = subprocess.run(
                [sys.executable, "-I", "-m", "unittest", "discover", "-s", directory,
                 "-p", "test_cascade_capture.py"],
                capture_output=True, text=True, timeout=15,
            )
        except subprocess.TimeoutExpired:
            return "timeout"
        return ("caught" if process.returncode == 1 and "FAILED (" in process.stderr
                else "missed" if process.returncode == 0 else "unviable")


def main():
    root = Path(__file__).resolve().parent
    source = (root / "cascade_capture.py").read_text(encoding="utf-8")
    helper = (root / "codex_adapter.py").read_text(encoding="utf-8")
    tests = (root / "test_cascade_capture.py").read_text(encoding="utf-8")
    # A failing unmodified suite must never make every fault appear caught.
    baseline = trial(source, helper, tests)
    print(json.dumps(dict(baseline="passed" if baseline == "missed" else baseline)), flush=True)
    if baseline != "missed":
        return 1
    failed = False
    for name, (before, after) in FAULTS.items():
        if source.count(before) != 1:
            raise SystemExit(f"{name}: expected exactly one mutation site; review the fault family")
        status = trial(source.replace(before, after), helper, tests)
        print(json.dumps(dict(fault=name, status=status)), flush=True)
        failed |= status != "caught"
    return int(failed)


if __name__ == "__main__":
    raise SystemExit(main())
