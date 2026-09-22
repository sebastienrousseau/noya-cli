<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/noyalib/v1/logos/noyalib.svg" alt="noya-cli logo" width="128" />
</p>

<h1 align="center">noya-cli</h1>

<p align="center">
  <code>noyafmt</code> and <code>noyavalidate</code>: comment-preserving YAML formatting, validation, and schema-driven repair.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/noya-cli/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/noya-cli/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/noya-cli"><img src="https://img.shields.io/crates/v/noya-cli.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Registry" /></a>
  <a href="https://docs.rs/noya-cli"><img src="https://img.shields.io/badge/docs.rs-noya--cli-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/noya-cli"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/noya-cli?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
  <a href="LICENSE-APACHE"><img src="https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg?style=for-the-badge" alt="License: Apache-2.0 OR MIT" /></a>
  <a href="https://github.com/sebastienrousseau/noya-cli/blob/main/docs/POLICIES.md"><img src="https://img.shields.io/badge/MSRV-1.86.0-93450a.svg?style=for-the-badge&logo=rust" alt="MSRV 1.86.0" /></a>
</p>

---

## Contents

**Getting started**

- [Install](#install) — binaries, packages, Cargo, and containers
- [Requirements](#requirements) — toolchain floor, platforms
- [Quick Start](#quick-start) — format and validate YAML

**The noya-cli ecosystem**

- [The noya-cli ecosystem](#the-noya-cli-ecosystem) — command-line tools and companion surfaces

**Library reference**

- [Capabilities at a glance](#capabilities-at-a-glance) — the current surface by theme
- [Ecosystem comparison](#ecosystem-comparison) — short matrix; full table at [`docs/COMPARISON.md`](docs/COMPARISON.md)
- [Benchmarks](#benchmarks) — headline numbers; full table at [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md)
- [Features](#features) — module-level capability list
- [Configuration](#configuration) — core options
- [Examples](#examples) — runnable example index

**Operational**

- [When not to use noya-cli](#when-not-to-use-noya-cli) — limitations
- [Development](#development) — make targets, fuzzing, CI
- [Security](#security) — guarantees and compliance
- [Documentation](#documentation) — all reference docs
- [Stability guarantees](#stability-guarantees) — SemVer axis, output stability, minimum toolchain discipline
- [License](#license)

---

## Install

### As a Rust library

```toml
[dependencies]
noya-cli = "0.0.51"
```

Most users install the binaries directly:

| Channel | Command |
| :--- | :--- |
| Cargo | `cargo install noya-cli --locked` |
| Formatter only | `cargo install noya-cli --locked --no-default-features --features noyafmt` |
| Homebrew | `brew install sebastienrousseau/tap/noya-cli` |
| Scoop | `scoop install noya-cli` after adding the Sebastien Rousseau bucket |
| Container | `docker run --rm -v "$(pwd):/work" -w /work ghcr.io/sebastienrousseau/noya-cli:latest config.yaml` |
| Release archive | Download the signed archive for the target from [GitHub Releases](https://github.com/sebastienrousseau/noya-cli/releases/latest) |

Release archives include the binaries, manpages, completions, checksums, and
provenance. Linux releases include GNU and static musl builds.

---

## Requirements

- Rust **1.86.0 or newer** when building from source.
- Linux, macOS, and Windows are tested on stable, beta, and nightly Rust.
- The crate pins `noyalib` at exactly `=0.0.51` under the lockstep release
  contract.

| Surface | Minimum toolchain | Enforcement |
| :--- | :---: | :--- |
| Binaries and library | Rust 1.86.0 | manifest and MSRV CI |
| Complete test and benchmark surface | Rust 1.86.0 | all-target CI |

---

## Quick Start

```bash
noyafmt --write config.yaml
noyafmt --check config/ deploy.yaml
noyavalidate --schema schema.yaml deploy.yaml
noyavalidate --schema schema.yaml --fix deploy.yaml
```

`noyafmt` edits through noyalib's CST so untouched comments and document
structure survive. `noyavalidate` supports syntax checks, JSON Schema 2020-12,
and conservative coercion of string-shaped scalar values.

---

## The noya-cli ecosystem

`noya-cli` is the command-line delivery surface in the lockstep noyalib family.

| Component | Purpose | Use case |
| :--- | :--- | :--- |
| `noyafmt` | Lossless YAML formatter | Local edits, pre-commit, and CI formatting gates |
| `noyavalidate` | YAML and JSON Schema validator | Deployment and configuration validation |
| [`noyalib`](https://github.com/sebastienrousseau/noyalib) | Core library | Embed the same parser and editing engine |
| [`noyalib-lsp`](https://github.com/sebastienrousseau/noyalib-lsp) | Editor server | Format and diagnose on save |

---

## Capabilities at a glance

| Area | Capability | Status |
| :--- | :--- | :--- |
| Formatting | Check, stdout, stdin, and in-place modes | Stable |
| Validation | YAML syntax and JSON Schema 2020-12 | Stable |
| Repair | Schema-driven scalar coercion | Stable |
| Distribution | Archives, Cargo, Homebrew, Scoop, packages, container | Automated |
| Shell integration | Bash, fish, zsh, and PowerShell completions | Generated |
| Documentation | Generated manpages from the Clap definitions | CI-gated |

---

## Ecosystem comparison

The command-line tools use the same parser and lossless editor as the core
library. The comparison focuses on operational behaviour rather than syntax
coverage alone.

| Project | Comment-preserving format | JSON Schema | Signed binaries |
| :--- | :---: | :---: | :---: |
| **noya-cli** | Yes | Yes | Yes |
| `yq` | Tool-dependent | No | Project-dependent |
| `yamllint` | Lint only | No | Project-dependent |
| `prettier` YAML | Reprints documents | No | npm package |

See [`docs/COMPARISON.md`](docs/COMPARISON.md) for the evidence and complete matrix.

---

## Benchmarks

Benchmark claims remain tied to checked-in Criterion harnesses and documented
hardware. CI smoke-runs the harnesses without asserting noisy wall-clock values.

| Scenario | Result | Environment |
| :--- | ---: | :--- |
| CLI dispatch | Measured by `cli_dispatch` | Criterion release build |
| Format 1 MiB YAML | Approximately 10 ms | Published project reference machine |
| Benchmark compilation | Per push | CI smoke gate |

See [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) for methodology and full results.

---

## Features

- `noyafmt --check`, `--write`, `--stdin`, and configurable indentation.
- `noyavalidate` syntax, schema, quiet, and `--fix` modes.
- Stable exit codes for shell pipelines and CI.
- Hosted GitHub Action and pre-commit hooks.
- FHS-aware `make install` and `make uninstall` using `PREFIX` and `DESTDIR`.
- Generated manpages and shell completions from one Clap command definition.

---

## Configuration

| Option | Effect |
| :--- | :--- |
| `noyafmt --check` | Exit 1 when a file would change |
| `noyafmt --write` | Rewrite files in place |
| `noyafmt --indent N` | Set indentation width |
| `noyavalidate --schema PATH` | Apply a YAML or JSON schema |
| `noyavalidate --fix` | Coerce repairable values before validation |
| `noyavalidate --quiet` | Suppress successful output |

The generated [`CLI reference`](docs/cli-reference.md) is authoritative for the
complete option surface.

---

## Examples

- [`format-precommit.sh`](examples/format-precommit.sh): formatting gate for commits.
- [`validate-k8s.sh`](examples/validate-k8s.sh): schema validation over manifests.
- [`fix-quoted-numbers.sh`](examples/fix-quoted-numbers.sh): schema-driven repair.

Run the scripts from a checkout after building or installing `noya-cli`.

---

## When not to use noya-cli

- Use `noyalib` directly when embedding formatting or validation in a Rust
  application.
- Use `noyalib-lsp` for per-keystroke editor diagnostics and incremental edits.
- Choose a YAML 1.1-specific tool when the complete YAML 1.1 resolver contract is
  required; these tools default to YAML 1.2.

The [detailed README reference](docs/README-REFERENCE.md) retains the full flag,
exit-code, verification, and integration discussion.

---

## Development

```bash
make
make test
make clippy
make fmt
make assets
make check-assets
```

CI runs the OS and toolchain matrix, coverage, fuzz regression, strict rustdoc,
feature checks, README examples, packaging smoke tests, and generated-asset drift
checks. See [`DEVELOPMENT.md`](DEVELOPMENT.md).

---

## Security

Report vulnerabilities privately according to [`SECURITY.md`](SECURITY.md).
The Rust workspace forbids `unsafe` code, dependencies are reviewed and audited,
and releases carry checksums, CycloneDX SBOMs, Sigstore signatures, and SLSA
provenance. The formatter does not execute YAML tags.

---

## Documentation

- [User Manual](https://sebastienrousseau.github.io/noya-cli/manual/)
- [API reference](https://docs.rs/noya-cli)
- [Developer documentation](DEVELOPMENT.md)
- [Ecosystem map](https://github.com/sebastienrousseau/noyalib/blob/main/docs/ECOSYSTEM.md)
- [CLI reference](docs/cli-reference.md)
- [Recipes](docs/recipes.md)
- [Engineering policies](docs/POLICIES.md)
- [Compliance grade](docs/COMPLIANCE-GRADE.md)
- [Detailed README reference](docs/README-REFERENCE.md)

---

## Stability guarantees

- During `0.0.x`, the patch component is the breaking-change axis.
- Formatting output and exit-code changes are breaking behaviour changes.
- The MSRV may rise only on the breaking axis with a changelog explanation.
- Manpages and completions are generated from the live command definitions and
  checked for drift in CI.

---

## License

Licensed under either [Apache License 2.0](LICENSE-APACHE) or
[MIT](LICENSE-MIT), at your option.
