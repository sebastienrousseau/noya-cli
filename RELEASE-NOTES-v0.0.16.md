<!-- SPDX-FileCopyrightText: 2026 Noyalib -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# noya-cli v0.0.16 Release Notes

Lockstep release with `noyalib` v0.0.16 (ADR-0005 strict-lockstep: the
`noyafmt` / `noyavalidate` binaries publish `=X.Y.Z` pinned to the core).
No change to the CLI contract (flags, exit codes, output).

## What changed

- **`noyalib` pin `=0.0.15` → `=0.0.16`** and the crate version bumped
  in lockstep.
- **MSRV raised 1.85.0 → 1.86.0**, matching the single lockstep floor
  adopted in v0.0.16.
- **Release workflow** publish steps made idempotent for clean re-runs.

## Corrected after release (ships in v0.0.17)

- **`crossbeam-epoch` RUSTSEC-2026-0204** (invalid-pointer-dereference)
  was present in the v0.0.16 lockfile via a transitive dependency and
  bumped to the patched 0.9.20 on `main` afterward.

## Engineering / CI (post-release, no user-facing change)

- Signed-history enforcement, upstream audit imports, and a
  `dependabot-vet` auto-refresh workflow.
- New CI gates brought toward the core's bar: a coverage gate, an MSRV
  gate, CodeQL, and OpenSSF Scorecard. `src/lib.rs` (the shared clap
  command tree) is at 100 % coverage; the enforced binary floor is set
  at the honest testable ceiling, since the residual is defensive I/O
  and parser-disagreement error arms unreachable via valid input.

## What did not change

- The `noyafmt` / `noyavalidate` flag surface, exit codes, and output.
- `#![forbid(unsafe_code)]` — intact.

## Upgrading

```bash
cargo install noya-cli --version 0.0.16 --locked
```

Requires **Rust 1.86.0+**.
