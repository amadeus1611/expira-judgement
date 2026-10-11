#!/usr/bin/env python3
"""Capture one fixed actual `ej check` batch, deterministic diagnostics only.

python3 -I tools/cascade_capture.py --data FILE --identities FILE --sampling FILE \
    --ej target/debug/ej --repo . --today YYYY-MM-DD --out NEW_DIRECTORY

Data: {claim,quote,label} JSONL. Companion: {case_id,row_idx,row_sha256}, where
row_sha256 hashes the exact corresponding data line, including its newline.
No local model, subscription, retry, resume or certification. POSIX only.
"""
import argparse
import datetime
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

# -I omits the script directory. Add only our own directory for our existing helper.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from codex_adapter import run_bounded, strict_json

PROTOCOL = "ej-deterministic-capture-v1"
LABELS = ("supports", "contradicts", "unsure")
VERDICTS = ("backed", "mismatch", "unbacked", "stale", "unsure")
MAX_INPUT = 32 * 1024 * 1024
MAX_CASES = 10000


def sha(data):
    return hashlib.sha256(data).hexdigest()


def encoded(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, allow_nan=False).encode("utf-8")


def file_hash(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def read_input(path):
    with Path(path).open("rb") as stream:
        raw = stream.read(MAX_INPUT + 1)
    if not raw or len(raw) > MAX_INPUT:
        raise ValueError("invalid_input_size")
    return raw


def inputs(data, identities):
    """Require explicit row-byte binding; grouping itself is not independence."""
    lines, companions = data.splitlines(keepends=True), identities.splitlines()
    if not 0 < len(lines) <= MAX_CASES or len(lines) != len(companions):
        raise ValueError("input_count_mismatch")
    cases, seen, data_hash = [], set(), sha(data)
    for line, companion in zip(lines, companions):
        row, identity = strict_json(line), strict_json(companion)
        if (not isinstance(row, dict) or set(row) != {"claim", "quote", "label"}
                or not isinstance(row["claim"], str) or not row["claim"].strip()
                or not isinstance(row["quote"], str) or row["label"] not in LABELS):
            raise ValueError("invalid_data_row")
        if (not isinstance(identity, dict) or set(identity) != {"case_id", "row_idx", "row_sha256"}
                or not isinstance(identity["case_id"], str) or not identity["case_id"].strip()
                or type(identity["row_idx"]) is not int or not 0 <= identity["row_idx"] < 2**63
                or identity["row_idx"] in seen or identity["row_sha256"] != sha(line)):
            raise ValueError("identity_mismatch")
        seen.add(identity["row_idx"])
        identifier = sha(encoded([data_hash, identity["row_idx"], identity["case_id"], sha(line)]))
        cases.append(dict(row, **identity, claim_id=identifier))
    return cases


def request_for(cases, today, source):
    """Only production Claim fields; reference labels never enter the CLI request."""
    return [dict(id=c["claim_id"], text=c["claim"], evidence=[
        dict(source=source, date=today, quote=c["quote"])]) for c in cases]


def findings_from(stdout, code, cases):
    if code not in (0, 1):
        raise ValueError("process_failed")
    findings = strict_json(stdout)
    if not isinstance(findings, list) or len(findings) != len(cases):
        raise ValueError("finding_count_mismatch")
    for finding, case in zip(findings, cases):
        if (not isinstance(finding, dict)
                or set(finding) != {"claim_id", "verdict", "layer", "confidence", "reason"}
                or finding["claim_id"] != case["claim_id"]
                or finding["verdict"] not in VERDICTS or finding["layer"] != "deterministic"
                or type(finding["confidence"]) not in (int, float)
                or not math.isfinite(finding["confidence"])
                or finding["confidence"] != (0 if finding["verdict"] == "unsure" else 1)
                or not isinstance(finding["reason"], str) or not finding["reason"].strip()):
            raise ValueError("finding_rejected")
    expected_code = int(any(f["verdict"] != "backed" for f in findings))
    if code != expected_code:
        raise ValueError("inconsistent_exit")
    return findings


def wilson(right, total):
    """Nominal v1 compatibility, z=1.96; no sampling/selection guarantee."""
    if type(right) is not int or type(total) is not int or not 0 <= right <= total:
        raise ValueError("invalid_counts")
    if total == 0 or right == 0:
        return 0.0
    p, z2 = right / total, 1.96**2
    return (p + z2 / (2 * total) - 1.96 * math.sqrt(
        p * (1 - p) / total + z2 / (4 * total**2))) / (1 + z2 / total)


def summary(cases, findings, failure):
    """Never score a successful prefix or remove failed rows from coverage."""
    table = {v: {label: 0 for label in LABELS} for v in VERDICTS}
    if failure is None:
        for case, finding in zip(cases, findings):
            table[finding["verdict"]][case["label"]] += 1
    stated = sum(table["backed"].values())
    blocks = sum(sum(table[v].values()) for v in ("mismatch", "unbacked", "stale"))
    block_right = sum(table[v][l] for v in ("mismatch", "unbacked", "stale")
                      for l in ("contradicts", "unsure"))
    return dict(cases=len(cases), group_count=len({c["case_id"] for c in cases}),
                completed_cases=len(cases) if failure is None else 0,
                operational_failures=len(cases) if failure is not None else 0,
                abstentions=sum(table["unsure"].values()), stated=stated,
                stated_right=table["backed"]["supports"], blocks=blocks, block_right=block_right,
                pre_escalation_coverage=(stated + blocks) / len(cases),
                nominal_pass_lower_bound=wilson(table["backed"]["supports"], stated) if failure is None else None,
                nominal_block_lower_bound=wilson(block_right, blocks) if failure is None else None,
                verdict_reference_table=table)


def source_state(repo):
    """Snapshot tracked bytes and dirty state; binary/source relationship is unattested."""
    def git(*args):
        return subprocess.run(["git", "-C", str(repo), *args], check=True,
                              capture_output=True, timeout=10).stdout
    if Path(git("rev-parse", "--show-toplevel").decode().strip()).resolve() != Path(repo).resolve():
        raise ValueError("repo_must_be_root")
    paths = git("ls-files", "-z").split(b"\0")
    hashes = {os.fsdecode(p): file_hash(Path(repo) / os.fsdecode(p)) for p in paths if p}
    status = git("status", "--porcelain=v1", "--untracked-files=all")
    return dict(head=git("rev-parse", "HEAD").decode().strip(),
                tracked_sha256=sha(encoded(hashes)), worktree_clean=not bool(status),
                status_sha256=sha(status), binary_source_relationship="unattested")


def write_new(path, value):
    with Path(path).open("xb") as stream:
        stream.write(encoded(value) + b"\n")
        stream.flush()
        os.fsync(stream.fileno())


def capture(*, data_path, identities_path, sampling_path, ej, repo, today, out,
            source="diagnostic quote; original provenance not retained", max_age=None, timeout=60):
    """One bounded invocation. A missing report means the preregistered run is incomplete."""
    if (datetime.date.fromisoformat(today).isoformat() != today
            or not isinstance(source, str) or not source.strip()
            or not math.isfinite(timeout) or not 0 < timeout <= 300
            or (max_age is not None and (type(max_age) is not int or not 0 <= max_age <= 365000))):
        raise ValueError("invalid_configuration")
    data, identities, sampling = (read_input(p) for p in (data_path, identities_path, sampling_path))
    cases = inputs(data, identities)
    declaration = strict_json(sampling)
    if not isinstance(declaration, dict) or not declaration:
        raise ValueError("invalid_sampling_declaration")
    request = encoded(request_for(cases, today, source))
    program = Path(ej).resolve()
    if not program.is_file() or not os.access(program, os.X_OK):
        raise ValueError("invalid_executable")
    manifest = dict(protocol=PROTOCOL, mode="deterministic_pre_escalation_diagnostic",
                    data_sha256=sha(data), identities_sha256=sha(identities), sampling_sha256=sha(sampling),
                    sampling_declaration=declaration, sampling_attested=False, source_state=source_state(repo),
                    executable_sha256=file_hash(program), request_sha256=sha(request),
                    runner_sha256=file_hash(__file__), helper_sha256=file_hash(Path(__file__).with_name("codex_adapter.py")),
                    python_version=sys.version, today=today, max_age_days=max_age,
                    evidence_source=source, evidence_date_meaning="opened diagnostic date, not publication date",
                    timeout_seconds=timeout, output_limit_bytes=1024 * 1024,
                    local_models=[], escalation=None, prompt_sha256=None, policy_sha256=None,
                    input_identities=[{k: c[k] for k in ("claim_id", "case_id", "row_idx", "row_sha256")} for c in cases])
    manifest["run_id"] = sha(encoded(manifest))
    output = Path(out).resolve()
    output.mkdir(parents=True, exist_ok=False)  # No overwrite, retry or unsafe resume.
    write_new(output / "manifest.json", manifest)
    started, failure, code, findings, stdout = time.monotonic(), None, None, [], ""
    with tempfile.TemporaryDirectory(prefix="ej-capture-") as directory:
        work = Path(directory)
        snapshot = work / "ej"
        claims = work / "claims.json"
        try:
            shutil.copyfile(program, snapshot)
            snapshot.chmod(0o500)
            claims.write_bytes(request)
            claims.chmod(0o400)
            if file_hash(snapshot) != manifest["executable_sha256"]:
                raise ValueError("executable_changed")
            command = [str(snapshot), "check", str(claims), "--today", today]
            if max_age is not None:
                command.extend(["--max-age", str(max_age)])
            stdout, code, failure, _ = run_bounded(command, "", directory, timeout)
            if file_hash(snapshot) != manifest["executable_sha256"] or file_hash(claims) != sha(request):
                raise ValueError("snapshot_changed")
            if failure is None:
                findings = findings_from(stdout, code, cases)
        except (OSError, ValueError, TypeError, RecursionError, OverflowError):
            failure = "batch_rejected"
    measurements = summary(cases, findings, failure)
    records = [dict(claim_id=c["claim_id"], case_id=c["case_id"], row_idx=c["row_idx"], label=c["label"],
                    status="complete" if failure is None else "operational_failure",
                    finding=findings[i] if failure is None else None, error=failure) for i, c in enumerate(cases)]
    report = dict(run_id=manifest["run_id"], protocol=PROTOCOL, certified=False,
                  measurement_complete=failure is None, error=failure, returncode=code,
                  captured_stdout_sha256=sha(stdout.encode()), batch_wall_ms=(time.monotonic() - started) * 1000,
                  timing_meaning="batch startup/load/wall time; not quiet local per-ruling latency",
                  subscription_attempts=0, tokens=None, cost_usd=None,
                  unmeasured=["exact_confirmation", "sampling_independence", "max_ece",
                              "stress_wrong_passes", "max_p95_ms", "whole_cascade_escalation"],
                  summary=measurements, records=records)
    write_new(output / ".report.tmp", report)
    os.replace(output / ".report.tmp", output / "report.json")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("data", "identities", "sampling", "ej", "repo", "today", "out"):
        parser.add_argument("--" + name, required=True)
    parser.add_argument("--source", default="diagnostic quote; original provenance not retained")
    parser.add_argument("--max-age", type=int)
    parser.add_argument("--timeout", type=float, default=60)
    args = parser.parse_args()
    try:
        report = capture(data_path=args.data, identities_path=args.identities, sampling_path=args.sampling,
                         ej=args.ej, repo=args.repo, today=args.today, out=args.out,
                         source=args.source, max_age=args.max_age, timeout=args.timeout)
    except (OSError, ValueError, TypeError, RecursionError, OverflowError, subprocess.SubprocessError):
        print("capture: preflight failed or run directory already exists", file=sys.stderr)
        return 2
    print(json.dumps({"run_id": report["run_id"], "measurement_complete": report["measurement_complete"],
                      "certified": False, "summary": report["summary"]}, allow_nan=False))
    return 0 if report["measurement_complete"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
