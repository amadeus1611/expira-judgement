# Codex verification of engineering ratchets (2026-10-11)

## Setup
- Baseline: main `9114f8f`; pending ratchets: `8824525`, `c44945d`, WIP `9ed1a15`.
- Working branch: `codex/ej-ratchets`. Merge `c8c852a` preserves both histories, current layer-1 bounds/name rules,
  all handoff sections and main's ordered cold-start instructions. No new decision-logic change in this session.
- Machine: Linux x86_64; cgroup CPU quota 4 cores, memory limit 32 GiB; four mutation workers.
- Rust 1.97.0, cargo-mutants 27.1.0, cargo-deny 0.20.2. Runtime setup and sources:
  `docs/research/2026-10-11-cloud-runtime.md`. Locked builds; TLS and artifact checksum verification retained.

## Results
Baseline fmt, clippy (`--workspace --all-targets --locked -- -D warnings`) and workspace tests passed before work.
The runner reported 104 passing tests at baseline and 117 on the reconciled ratchet branch. In each run one of those
is `ej-nli::tests::model_smoke`, which returns immediately because EJ_NLI_DIR is unset: model inference was not tested.
The final documentation changes are checked by the same workspace suite before push.

The full mutation command exited 0:

```sh
. /workspace/setup/activate.sh
cd /workspace/expira-judgement
PROPTEST_CASES=256 cargo mutants -j 4 --timeout 60 \
  -p ej-core -p ej-evidence -p ej-judge -p ej-delegate -p ej-habits -p ej-bench \
  --exclude crates/ej-bench/src/main.rs --output /workspace/setup/ratchets-mutants
```

| Crate | Generated | Caught | Unviable | Missed | Timeouts |
|---|---:|---:|---:|---:|---:|
| ej-bench library | 292 | 275 | 17 | 0 | 0 |
| ej-core | 6 | 6 | 0 | 0 | 0 |
| ej-delegate | 46 | 39 | 7 | 0 | 0 |
| ej-evidence | 368 | 356 | 12 | 0 | 0 |
| ej-habits | 38 | 36 | 2 | 0 | 0 |
| ej-judge | 106 | 82 | 24 | 0 | 0 |
| Total | 856 | 794 | 62 | 0 | 0 |

Counts come from this run's `mutants.out/{mutants.json,caught.txt,unviable.txt,missed.txt,timeout.txt,outcomes.json}`.
The runner reported 7 minutes. Unviable mutants are compile failures, not test kills. The prior branch's handoff
reported 78 missed ej-bench mutants before its WIP changes; that historical baseline was not independently rerun here.
The pending CI change adds ej-bench with only its CLI wrapper excluded. The current bounds and name rules were included.

`cargo deny check` exited 0: advisories, bans, licences and sources passed. Existing duplicate-version warnings remain;
the documented paste advisory exception and licence policy were not broadened.

`cargo build --workspace --locked` passed. A three-case CLI smoke check verified `backed` for exact evidence,
`mismatch` for a changed USD figure, and `unsure` for an unproven paraphrase; exit 1 correctly reflects failed findings.
The complete reusable installer passed twice, including checksum verification and locked workspace builds.

## Configuration and remaining work
Saved install_script and start_skill in the environment draft, including tool activation, native runtime override,
existing-checkout guidance and readiness checks. Saved custom HTTPS destinations api.github.com, huggingface.co and
datasets-server.huggingface.co; package-manager presets are preserved. Draft saving does not apply runtime changes
or publish the environment. Setup outputs and full logs are outside the checkout under /workspace/setup.

Git HTTPS read works through the platform proxy. GitHub API CONNECT requests and Hugging Face requests return 403
under the current policy. PR creation, CI inspection and merge cannot be claimed until access works. VitaminC was
not rebuilt, and no whole-cascade, audited-label or model-latency result is claimed. Claude CLI is absent and
ANTHROPIC_API_KEY is unset. Escalation calls: 0; escalation cost: $0. No frozen data or standard v1 bars changed;
the proven-99% goal remains open.
