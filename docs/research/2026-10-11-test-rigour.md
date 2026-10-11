# Test rigour (2026-10-11)

## Problem
Example-based unit tests show that a few chosen inputs work; they say nothing about inputs nobody thought of, and passing
tests do not show the tests would catch a bug. Layer 1 is the only layer allowed to call a claim proven, and the
calibration and delegation maths decide what is acted on, so their tests must be measured for strength, not counted.

## Sources
- proptest (property-based testing for Rust: invariants checked on generated inputs, with shrinking to a minimal failing case): https://proptest-rs.github.io/proptest/
- cargo-mutants (mutation testing for Rust: injects small faults and reports any that no test catches): https://mutants.rs/
- cargo-mutants CI guidance (fail on uncaught mutants, check changed code on PRs, all code on main): https://mutants.rs/ci.html
- Just et al., "Are mutants a valid substitute for real faults in software testing?" (FSE 2014; mutant detection correlates with real-fault detection): https://doi.org/10.1145/2635868.2635929

## Licence
- proptest: MIT OR Apache-2.0 (crates.io). Dev-dependency only.
- cargo-mutants: MIT (crates.io). A tool run in CI, not linked into the code.

## Candidates
1. Property tests for layer 1 invariants (no panic on any input; calendar maths vs an independent day count; verbatim span
   backed; negated span never backed; changed figure mismatch; future evidence unusable). Done, 2,000 cases each.
2. Property tests for the calibration and threshold maths (Wilson bound below the rate and monotone; fitted thresholds meet
   their target on the data they were fitted on; Platt preserves order) and delegation (chosen start tier is never costlier
   than any other).
3. Mutation testing with zero surviving mutants as a gate on the decision-critical crates (ej-evidence, ej-judge, ej-bench
   calib/standard, ej-delegate, ej-habits); crates that need a model or the network are covered by their own smoke tests.
4. Later ratchets: coverage measurement (cargo-llvm-cov), fuzzing (cargo-fuzz, needs nightly), dependency audit (cargo-deny).

## Decision
1 adopted. 2 and 3 in progress in this session; each surviving mutant is fixed by a test that kills it, or by removing the
code it shows to be unneeded, never by skipping it without a written reason.
