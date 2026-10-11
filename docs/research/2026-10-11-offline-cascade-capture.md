# Research: capturing the actual cascade without overstating its measurements

## Problem
Whole-cascade measurement lacks a tested runner around actual `ej check`. A separately assembled Python judge chain
could disagree with production routing, and batch exit 1 is a normal result rather than an inference failure.
Missing outputs or unsafe index resumes could silently change the denominator. Final findings do not retain all-input
probabilities or component timings. The first bounded change must capture production outcomes without certifying them.

## Sources
- Inspected `crates/ej-cli/src/main.rs`, `crates/ej-core/src/lib.rs` and `crates/ej-judge/src/lib.rs`: claim JSON,
  findings, exit statuses, deterministic-first routing, unproven-policy skipping and calibrated self-gating.
- Read CPython 3.12.14 [subprocess source](https://github.com/python/cpython/blob/v3.12.14/Lib/subprocess.py),
  `communicate` and timeout cleanup. Killing only the parent does not establish descendant termination; reuse the
  repository's already tested bounded POSIX process-group helper, with file-backed outputs and a hard read limit.
- Re-read the saved [SciPy interval source](https://github.com/scipy/scipy/blob/main/scipy/stats/_binomtest.py) and
  licence. Wilson is a score-test inversion. Independent rational score inequalities can check threshold boundaries
  without comparing the same interval formula with itself. Existing v1 uses z=1.96, not SciPy's full-precision quantile.
- Existing statistical design and limitations: `docs/validation/general-99.md`, its staged roadmap and
  `docs/research/2026-10-11-rigorous-99-validation.md`. Group identifiers alone cannot attest independent sampling.

## Licence
Read CPython's complete [LICENSE](https://github.com/python/cpython/blob/v3.12.14/LICENSE): Python software and
documentation use PSF License Version 2 with historical licences; documentation examples additionally permit 0BSD.
SciPy's source LICENSE is BSD-3-Clause, re-read from the saved upstream file. No implementation is copied or dependency
added; process handling reuses our fresh diagnostic helper. No model, corpus or paper text is introduced. Existing
VitaminC files remain outside the checkout under their previously recorded CC BY-SA 3.0 data obligations.

## Candidates
1. Reimplement the cascade and combine diagnostic Codex replies. Rejected for production measurement: no Rust Codex
   transport is integrated, and endpoint authentication and tool isolation remain unverified.
2. Add every trace, inference, statistical and resume feature at once. Deferred: large scope hides integrity faults.
3. Capture one fixed offline production batch first, strict identities/bytes, explicit complete/failure records and
   nominal statistics. No resume or paid calls, no eligible local model until its qualification can be attested.

## Decision
Implement candidate 3 as a diagnostic deterministic pre-escalation capture. Hash raw data, group identities, sampling
declaration, executable, runner/helper and tracked source state; snapshot the executable before running. Strip labels
from requests. Reject reordered, missing, duplicate or foreign findings and inconsistent exits. Keep all input IDs in
failed-run accounting. Persist a manifest before execution and publish a final report atomically; reject existing run
directories instead of implementing unsafe resume. Report abstentions separately from blocks, counts and nominal
Wilson compatibility only. Exact confirmation, sampling attestation, ECE, fresh stress and quiet local p95 stay
unmeasured. Always mark certification unavailable. Add fault tests and real CLI integration controls before measuring.
