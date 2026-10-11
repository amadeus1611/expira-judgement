#!/usr/bin/env python3
"""Measure the escalation layer (layer 3) through a subscription CLI, no API key: each case goes to
`claude -p` with the checker system prompt and the ruling JSON schema, the cited span is checked against the quote
in code (as ej-judge does), and every reply is written with its usage and cost.

    python3 -I tools/escalation_bench.py CASES.jsonl OUT.jsonl --model claude-haiku-5-5 [--n 1000] [--jobs 6] [--seed 7]

CASES.jsonl rows are {"claim","quote","label"}. OUT.jsonl rows: index, label, ruling, confidence, span_ok, acted
(a non-unsure ruling whose span is found in the quote), cost_usd, tokens, error. Rows already in OUT are skipped, so a
run can be resumed. Prints a summary: accuracy of acted "supports" calls and its 95% Wilson lower bound."""
import concurrent.futures as cf, json, math, os, random, re, subprocess, sys

SYSTEM = ("You check whether quoted evidence supports a claim. Use only the quotes given; do not use outside knowledge. "
          "supports: the quotes state the claim or directly imply it, with the same figures, units, dates, names and "
          "polarity. contradicts: the quotes state something incompatible with the claim. unsure: anything else, "
          "including partial support. evidence_span: copy the exact words from the quote that decide your ruling (empty "
          "only when unsure). reason: one sentence. confidence: your probability that the ruling is right.")
SCHEMA = json.dumps({"type": "object", "properties": {
    "ruling": {"type": "string", "enum": ["supports", "contradicts", "unsure"]}, "confidence": {"type": "number"},
    "evidence_span": {"type": "string"}, "reason": {"type": "string"}},
    "required": ["ruling", "confidence", "evidence_span", "reason"], "additionalProperties": False})


def norm(s):
    return " ".join(s.lower().split())


def span_ok(span, quote):
    sp = norm(span.strip("\"'….")).strip()
    return bool(sp) and sp in norm(quote)


def ask(model, case):
    prompt = f"Claim: {case['claim']}\n\nQuotes:\n[1] \"{case['quote']}\""
    cmd = ["claude", "-p", "--model", model, "--output-format", "json", "--no-session-persistence",
           "--strict-mcp-config", "--setting-sources", "", "--tools", "", "--system-prompt", SYSTEM,
           "--json-schema", SCHEMA, prompt]
    try:
        out = subprocess.run(cmd, capture_output=True, text=True, timeout=300, cwd="/tmp")
        d = json.loads(out.stdout)
        r = d.get("structured_output") or json.loads(d.get("result") or "{}")
        u = d.get("usage", {})
        return {"ruling": r.get("ruling"), "confidence": r.get("confidence"), "span": r.get("evidence_span", ""),
                "cost_usd": d.get("total_cost_usd"), "tokens": {k: u.get(k) for k in
                ("input_tokens", "output_tokens", "cache_read_input_tokens", "cache_creation_input_tokens")}}
    except Exception as e:  # one failed call is recorded, never guessed
        return {"error": str(e)[:200]}


def wilson(k, n, z=1.96):
    if n == 0:
        return 0.0
    p = k / n
    d = 1 + z * z / n
    return max(0.0, (p + z * z / (2 * n) - z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n))) / d)


def main(a):
    cases_path, out_path = a[1], a[2]
    opt = lambda k, d: a[a.index(k) + 1] if k in a else d
    model, n, jobs, seed = opt("--model", "claude-haiku-5-5"), int(opt("--n", 1000)), int(opt("--jobs", 6)), int(opt("--seed", 7))
    cases = [json.loads(l) for l in open(cases_path)]
    idx = sorted(random.Random(seed).sample(range(len(cases)), min(n, len(cases))))
    done = set()
    if os.path.exists(out_path):
        done = {json.loads(l)["index"] for l in open(out_path)}
    todo = [i for i in idx if i not in done]
    with open(out_path, "a") as f, cf.ThreadPoolExecutor(jobs) as pool:
        for i, r in zip(todo, pool.map(lambda i: ask(model, cases[i]), todo)):
            c = cases[i]
            r.update(index=i, label=c["label"], model=model,
                     span_ok=span_ok(r.get("span", ""), c["quote"]) if r.get("ruling") in ("supports", "contradicts") else None)
            r["acted"] = r.get("ruling") in ("supports", "contradicts") and bool(r["span_ok"])
            f.write(json.dumps(r) + "\n")
            f.flush()
    rows = [json.loads(l) for l in open(out_path)]
    sup = [r for r in rows if r.get("acted") and r["ruling"] == "supports"]
    k = sum(r["label"] == "supports" for r in sup)
    cost = sum(r.get("cost_usd") or 0 for r in rows)
    errs = sum(1 for r in rows if r.get("error"))
    print(json.dumps({"model": model, "rows": len(rows), "errors": errs, "acted_supports": len(sup), "right": k,
                      "wilson_lb": round(wilson(k, len(sup)), 4), "unsure": sum(r.get("ruling") == "unsure" for r in rows),
                      "span_rejects": sum(1 for r in rows if r.get("span_ok") is False), "cost_usd": round(cost, 4)}))


if __name__ == "__main__":
    if len(sys.argv) < 3:
        raise SystemExit(__doc__)
    main(sys.argv)
