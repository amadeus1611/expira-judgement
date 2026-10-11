# Reproducible cloud runtime (2026-10-11)

## Problem
The new cloud machine had no Cargo toolchain. After installing the pinned Rust 1.97.0 toolchain, the locked workspace
build failed because the egress proxy denied the ort-sys binary download from cdn.pyke.io. Setup must preserve the
repository's toolchain pin, dependency declarations, lockfile, TLS verification and artifact checksums.

## Sources
- The pinned ort-sys 2.0.0-rc.10 build script, fetched by Cargo with registry checksum verification:
  https://github.com/pykeio/ort/blob/v2.0.0-rc.10/ort-sys/build.rs
  It requires ONNX Runtime 1.22.0 and supports ORT_LIB_LOCATION and ORT_PREFER_DYNAMIC_LINK.
- ONNX Runtime 1.22.0 release metadata and wheel SHA-256, read over HTTPS:
  https://pypi.org/pypi/onnxruntime/1.22.0/json
- Rustup installer and checksum, fetched over HTTPS:
  https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init
  and the corresponding .sha256 file.
- Mutation-test CI guidance, read from the project's source:
  https://github.com/sourcefrog/cargo-mutants/blob/main/book/src/ci.md
- Timeout semantics, read from the project's source:
  https://github.com/sourcefrog/cargo-mutants/blob/main/book/src/timeouts.md

## Licence
The ONNX Runtime wheel's LICENSE was extracted and read: MIT. Its ThirdPartyNotices.txt is retained alongside the
runtime under /workspace/setup. The cargo-mutants source LICENSE was read: MIT. Existing supply-chain policy and
its sources are documented in 2026-10-11-supply-chain.md; no licence allowlist changes are needed.

## Candidates
1. Change Cargo features or skip the NLI crate. Rejected: this would change the repository or narrow required checks.
2. Disable verification for the binary download. Rejected: this would remove an integrity guarantee.
3. Use the supported system-library override with the matching ONNX Runtime library from its PyPI wheel. Chosen.
4. Allow cdn.pyke.io and use the crate's default download. Unnecessary for the verified override used here.

## Decision
Install Rust 1.97.0 with rustfmt and clippy under /workspace. Verify the rustup installer against its published SHA-256.
Verify the Linux x86_64 ONNX Runtime 1.22.0 wheel against its published SHA-256 before extracting shared libraries:
`b978aa63a9a22095479c38371a9b359d4c15173cbb164eaad5f2cd27d666aa65`.
Activate the runtime through ORT_LIB_LOCATION, ORT_PREFER_DYNAMIC_LINK and LD_LIBRARY_PATH in an external helper.
Keep all setup artifacts outside the checkout and use locked Cargo installs/builds. Pin cargo-mutants 27.1.0 and
cargo-deny 0.20.2. Exercise the complete saved installer and the CLI, then save install_script and start_skill.

This restores build, lint, test and CLI capabilities without validating model inference: EJ_NLI_DIR is unset.
Git HTTPS read and push work with existing platform authentication. Native GitHub API CONNECT requests were denied,
but the platform GitHub CLI route subsequently created PR #3 and inspected its CI using existing credentials.
Hugging Face access remains blocked by the current egress policy; domain additions are saved as a draft, not claimed
to be active. Claude CLI/authentication is absent, so no paid escalation is attempted. Data access and an authenticated
escalation transport remain external prerequisites for cascade measurement.
