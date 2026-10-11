# Supply chain gate (2026-10-11)

## Problem
The workspace pulls about 190 third-party crates (tokenizers, ort, ureq, rustls, proptest and their trees). Nothing
checked that their licences are compatible with a private, closed-source product, that none carries a known RustSec
vulnerability, or that every crate comes from crates.io. A new dependency could bring in a copyleft licence, a
vulnerable version or a git source without anyone noticing.

## Sources
- cargo-deny book (checks: advisories, bans, licenses, sources): https://embarkstudios.github.io/cargo-deny/
- cargo-deny licenses configuration (`allow`, `confidence-threshold`, `private.ignore`): https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html
- cargo-deny advisories configuration (`version = 2`, `ignore` with reasons): https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html
- cargo-deny GitHub Action: https://github.com/EmbarkStudios/cargo-deny-action
- RUSTSEC-2024-0436 (paste unmaintained): https://rustsec.org/advisories/RUSTSEC-2024-0436
- SPDX licence list: https://spdx.org/licenses/

## Licence
- cargo-deny: MIT OR Apache-2.0. A tool run in CI, not linked into the code.
- Licences found with `cargo deny list` on Cargo.lock at this commit: MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception,
  BSD-2-Clause, BSD-3-Clause, BSL-1.0, 0BSD, ISC, Unicode-3.0, Unlicense, Zlib, CDLA-Permissive-2.0, and
  LGPL-2.1-or-later (r-efi, offered as MIT OR Apache-2.0 OR LGPL-2.1-or-later).

## Candidates
1. Allow every licence found. Rejected: it would let LGPL in as an accepted choice and grow silently.
2. Allow only the licences some crate actually needs after dual-licence resolution. Every dual- or tri-licensed crate
   resolves to MIT or Apache-2.0; the crates that need more are subtle (BSD-3-Clause), ring / rustls-webpki / untrusted
   (ISC), unicode-ident (Unicode-3.0 for its tables) and webpki-roots / webpki-root-certs (CDLA-Permissive-2.0, the
   Mozilla root certificate data). cargo-deny warns about allowed licences nobody uses, so the list stays minimal.
3. cargo-audit only (advisories). Rejected: no licence, ban or source checks.

## Decision
Candidate 2, in `deny.toml`: allow MIT, Apache-2.0, BSD-3-Clause, ISC, Unicode-3.0, CDLA-Permissive-2.0; workspace
crates (private, UNLICENSED) are not checked. Advisories: any vulnerability fails; one unmaintained notice is ignored
with its reason (RUSTSEC-2024-0436, paste, a compile-time macro reached only through tokenizers 0.21, no safe upgrade).
Duplicate versions warn (base64, getrandom, r-efi, rand_core, smallvec, syn, windows-sys today); wildcards are denied;
sources are crates.io only. `cargo deny check` runs in the CI `check` job.
