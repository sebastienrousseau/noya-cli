<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/noyalib/v1/logos/noyalib.svg" alt="Noyalib logo" width="128" />
</p>

<h1 align="center">noya-cli</h1>

<p align="center">
  <strong><code>noyafmt</code> and <code>noyavalidate</code> —
  the YAML formatter and validator built on the noyalib
  library.</strong>
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/noya-cli/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/noya-cli/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/noya-cli"><img src="https://img.shields.io/crates/v/noya-cli.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/noyalib"><img src="https://img.shields.io/badge/docs.rs-noyalib-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://github.com/sebastienrousseau/noya-cli/releases"><img src="https://img.shields.io/github/v/release/sebastienrousseau/noya-cli?style=for-the-badge&label=release&color=blueviolet" alt="GitHub Release" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/noyalib"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/noyalib?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
</p>

---

## Contents

- [Install](#install) — every channel mapped
- [Requirements](#requirements) — toolchain floor, platforms, the core pin
- [Quick Start](#quick-start) — common workflows
- [`noyafmt`](#noyafmt) — formatter reference
- [`noyavalidate`](#noyavalidate) — validator + autofix reference
- [Exit codes](#exit-codes) — for shell pipelines and CI gates
- [Examples](#examples) — runnable demo scripts
- [Shell completions and man pages](#shell-completions-and-man-pages)
- [Verification](#verification) — cosign + SLSA cookbook
- [When not to use these tools](#when-not-to-use-these-tools)
- [Documentation](#documentation)
- [License](#license)

---

## Install

| Channel | Command |
|---|---|
| Pre-built binaries | download for 8 targets (Linux gnu/musl, macOS, Windows on x86_64/aarch64) from the [latest release](https://github.com/sebastienrousseau/noya-cli/releases/latest) — each with manpages, completions, SHA-256, and SLSA provenance |
| Linux packages | `.deb` and `.rpm` on the [latest release](https://github.com/sebastienrousseau/noya-cli/releases/latest) |
| Homebrew (macOS/Linux) | `brew install sebastienrousseau/tap/noya-cli` |
| Scoop (Windows) | `scoop bucket add sebastienrousseau https://github.com/sebastienrousseau/scoop-bucket` then `scoop install noya-cli` |
| AUR (Arch) | [`noyalib-bin`](https://aur.archlinux.org/packages/noyalib-bin), e.g. `yay -S noyalib-bin` |
| Cargo (crates.io) | `cargo install noya-cli --locked` |
| Cargo (from source) | `cargo install --locked --path .` |
| GNU Make (binaries + manpages + completions) | `make install` — honors `PREFIX` (default `/usr/local`) and `DESTDIR`; `make uninstall` reverses it |
| Container (GHCR) | `docker run --rm -v "$(pwd):/work" -w /work ghcr.io/sebastienrousseau/noyafmt:latest --check ci/*.yaml` |

### Formatter only, if you do not need `noyavalidate`

```bash
cargo install noya-cli --locked --no-default-features --features noyafmt
```

The default install includes `noyavalidate`, whose schema errors are
rendered by [`miette`](https://crates.io/crates/miette) with source
excerpts and carets — worth the weight for a validator, since pointing at
the offending line *is* the job. It does mean the default pulls **130**
crates against **32** for the formatter alone, most of the difference
being `miette`'s `fancy` renderer and its backtrace stack.

If you only ever run `noyafmt`, the command above is a smaller supply
chain and a faster build. Nothing is degraded for anyone who wants the
diagnostics — it is a choice, not a default.

Releases ship the crate archive with a CycloneDX SBOM and sigstore
bundles, plus the pre-built binary archives and Linux packages
above, each with SHA-256 and a SLSA build-provenance attestation —
see [Verification](#verification).

**MSRV: Rust 1.86.0** — the lowest toolchain this crate can be
**built and tested** on, matching the noyalib core floor.
`criterion 0.8` (the benchmark dev-dependency) declares
`rust-version = 1.86`, so `cargo check --all-targets` and the
bench suite fail on 1.85 (`criterion@0.8.2 requires rustc 1.86`),
though `cargo check --lib` still builds. `clap_builder 4.6` in the
CLI dep tree is edition-2024. We publish the number we verify.

---

## Requirements

- **Rust 1.86.0 or newer** to build from source: `rust-version` in
  the manifest, enforced by the `msrv-core` CI job on every push.
- **Any tier-1 platform.** CI runs the tests on Linux, macOS, and
  Windows with the stable, beta, and nightly toolchains; stable is the
  gate, beta and nightly are early warning.
- **The matching core.** This crate pins `noyalib` at the identical
  `=0.0.X` and releases in lockstep with it; Cargo resolves that pin
  for you.
- **Nothing at runtime**: the binaries are static where the platform
  allows (musl on Linux) and link only the C runtime elsewhere.

## Quick Start

```bash
# Format a file in-place; comments + indentation preserved.
noyafmt --write config.yaml

# CI gate — exits 1 if any file would change.
noyafmt --check ci/*.yaml

# Validate syntax + JSON Schema 2020-12.
noyavalidate --schema schema.yaml deploy.yaml

# Validate + auto-fix obvious type slips (port: "8080" → 8080).
noyavalidate --schema schema.yaml --fix deploy.yaml
```

---

## `noyafmt`

YAML formatter mirroring the `rustfmt` / `prettier` ergonomics:

```bash
noyafmt config.yaml                # print formatted source to stdout (default)
noyafmt --write config.yaml        # rewrite in place
noyafmt --check ci/*.yaml          # CI gate
noyafmt --indent 4 config.yaml     # override default 2-space indent
cat foo.yaml | noyafmt --stdin     # editor pipe (Vim, Emacs, …)
git ls-files '*.yaml' | xargs noyafmt --check
```

The formatter runs through noyalib's lossless CST: comments,
anchor positions, and document structure are preserved
byte-for-byte; only whitespace and quoting are normalised.

| Flag | Effect |
|---|---|
| `--check` | Verify each FILE is formatted; print files that need formatting; exit 1 if any do. Non-destructive. |
| `--write` | Rewrite each FILE in place. Default is to print to stdout. Mutually exclusive with `--check`. |
| `--stdin` | Read from stdin, write to stdout. Mutually exclusive with FILE arguments. |
| `--indent N` | Indentation width in spaces (default: 2). |

---

## `noyavalidate`

YAML syntax checker with optional **JSON Schema 2020-12**
enforcement and **schema-driven autofix**.

```bash
noyavalidate manifest.yaml                          # syntax only
noyavalidate --schema schema.yaml deploy.yaml       # + schema check
noyavalidate --schema schema.yaml --fix deploy.yaml # + autofix
cat manifest.yaml | noyavalidate                    # stdin
```

The autofix engine
([`coerce_to_schema`](https://docs.rs/noyalib/latest/noyalib/fn.coerce_to_schema.html))
rewrites string-shaped scalars into the schema's expected type
when the parse succeeds. Loops until convergence; unparseable
inputs (`port: "abc"` against `type: integer`) are left in place
so a follow-up `validate_against_schema` call surfaces the
residue.

Diagnostics use [`miette`](https://crates.io/crates/miette) for
rustc-style source pointers:

```text
× schema violation: "8080" is not of type "integer"
   ╭─[deploy.yaml:3:7]
 2 │ replicas: 3
 3 │ port: "8080"
   ·       ─┬───
   ·        ╰── here
 4 │ host: api
   ╰────
   help: pass --fix to coerce string-shaped scalars to the
         schema's declared type.
```

| Flag | Effect |
|---|---|
| `-s, --schema PATH` | Validate each document against JSON Schema 2020-12 at PATH (the schema may itself be YAML or JSON). |
| `--fix` | Rewrite FILE in place via the CST formatter (lossless: byte-faithful for everything except normalised whitespace / line endings). With stdin input, the formatted bytes go to stdout. |
| `-q, --quiet` | Suppress success output. |

---

## Exit codes

| Code | `noyafmt` | `noyavalidate` |
|---|---|---|
| 0 | success (or no changes if `--check`) | all valid |
| 1 | parse / I/O error, or `--check` found unformatted file(s) | parse error or schema violation |
| 2 | invalid usage (bad arg combination) | invalid usage |
| 3 | — | I/O error (read / write) |

---

## Examples

End-to-end runnable demos under
[`examples/`](examples/):

| Script | What it shows |
|---|---|
| [`format-precommit.sh`](examples/format-precommit.sh) | Drop-in `git pre-commit` hook gating commits on `noyafmt --check`. |
| [`validate-k8s.sh`](examples/validate-k8s.sh) | CI step that runs `noyavalidate --schema` over a directory of Kubernetes manifests. |
| [`fix-quoted-numbers.sh`](examples/fix-quoted-numbers.sh) | Walkthrough of the `--fix` autofix flow: quoted scalar → schema-typed integer, with the surrounding comment preserved. |

```bash
chmod +x crates/noya-cli/examples/*.sh
crates/noya-cli/examples/fix-quoted-numbers.sh
```

---

## Shell completions and man pages

Tarball releases ship pre-built completions for bash, fish, zsh,
and PowerShell, plus roff man pages. Distro packages drop them
into the standard system locations
(`/usr/share/bash-completion/completions/`,
`/usr/share/man/man1/`, …).

`make install` places all of them in the standard locations. If
installing via `cargo install`, the pre-built copies live in this
repository (`complete/`, `docs/*.1`) — or regenerate them from the
clap definitions:

```bash
git clone https://github.com/sebastienrousseau/noya-cli
cd noya-cli
make assets    # regenerates complete/* and docs/*.1 via build.rs
```

CI enforces that the committed copies are bit-identical to what the
clap definitions generate (`make check-assets`), so they cannot
drift from `--help`.

---

## Verification

Every release artefact (crate archive, SBOM) ships with a cosign
keyless signature and a SLSA build-provenance attestation. Verify
before trusting a download:

```bash
COSIGN_EXPERIMENTAL=1 cosign verify-blob \
  --certificate-identity-regexp 'https://github.com/sebastienrousseau/noya-cli/' \
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
  --bundle <artefact>.bundle \
  <artefact>

gh attestation verify --owner sebastienrousseau <artefact>
```

Full cookbook including the offline / FIPS-bound flow:
[`pkg/VERIFY.md`](https://github.com/sebastienrousseau/noyalib/blob/main/pkg/VERIFY.md).

---

## When not to use these tools

- **You need to format YAML faster than human-perceivable
  latency.** `noyafmt` runs end-to-end on a 1 MiB document in
  ~10 ms; for `<100 KB` documents that's already invisible.
  But for a streaming editor pipeline that wants per-keystroke
  formatting, the LSP server (`noyalib-lsp`) issues incremental
  `TextEdit[]`s instead.
- **You need YAML 1.1-only behaviour, top to bottom.**
  `noyavalidate` follows YAML 1.2; the `legacy_booleans` opt-in
  is exposed at the library level but not yet plumbed through
  the CLI.
- **You need to embed the formatter or validator in your own
  Rust binary.** Use the [`noyalib`](https://crates.io/crates/noyalib)
  library directly — every CLI feature flows through public
  library APIs (`cst::format_with_config`,
  `validate_against_schema`, `coerce_to_schema`).

---

## Documentation

The four entry points, identical across every repo in the family:

- **[User Manual](https://sebastienrousseau.github.io/noya-cli/manual/)** — this crate's rendered book: its guides, architecture, and release notes; the family manual for the core library is at [https://sebastienrousseau.github.io/noyalib/manual/](https://sebastienrousseau.github.io/noyalib/manual/)
- **[API reference](https://docs.rs/noya-cli)** — rustdoc on docs.rs
- **[Developer docs](DEVELOPMENT.md)** — this repo's dev entry point, pointing at the family guide
- **[Ecosystem map](https://github.com/sebastienrousseau/noyalib/blob/main/docs/ECOSYSTEM.md)** — the six crates, the lockstep model, the scorecard

- **Engineering policies** (MSRV, SemVer, security, performance, concurrency, platform support, feature flags):
  [`doc/POLICIES.md`](https://github.com/sebastienrousseau/noyalib/blob/main/docs/POLICIES.md)
- **Security policy**:
  [`SECURITY.md`](https://github.com/sebastienrousseau/noyalib/blob/main/SECURITY.md)
- **CLI flag reference**:
  [`docs/cli-reference.md`](docs/cli-reference.md)
- **Recipes (CI gates, pre-commit, editor integration)**:
  [`docs/recipes.md`](docs/recipes.md)
- **Workspace README**:
  <https://github.com/sebastienrousseau/noyalib#readme>
- **Per-channel install + verify**:
  [`pkg/VERIFY.md`](https://github.com/sebastienrousseau/noyalib/blob/main/pkg/VERIFY.md)
- **Library API the binaries call into**:
  <https://docs.rs/noyalib>

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT](https://opensource.org/licenses/MIT), at your option.
