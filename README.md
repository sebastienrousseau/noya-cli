<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/noyalib/v1/logos/noyalib.svg" alt="Noyalib logo" width="128" />
</p>

<h1 align="center">noyalib-cli</h1>

<p align="center">
  <strong><code>noyafmt</code> and <code>noyavalidate</code> —
  the YAML formatter and validator that ship as the CLI half of
  the noyalib workspace.</strong>
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/noyalib/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/noyalib/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/noyalib"><img src="https://img.shields.io/crates/v/noyalib.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/noyalib"><img src="https://img.shields.io/badge/docs.rs-noyalib-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://github.com/sebastienrousseau/noyalib/releases"><img src="https://img.shields.io/github/v/release/sebastienrousseau/noyalib?style=for-the-badge&label=release&color=blueviolet" alt="GitHub Release" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/noyalib"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/noyalib?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
</p>

---

## Contents

- [Install](#install) — every channel mapped
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
| Cargo (crates.io) | `cargo install noya-cli --locked` |
| Cargo (from source) | `cargo install --locked --path .` |
| Homebrew (personal tap) | `brew tap sebastienrousseau/tap && brew install noyalib` |
| Arch (AUR) | `yay -S noyalib-bin` (binary) or `yay -S noyalib` (source) |
| Scoop (Windows) | `scoop bucket add sebastienrousseau https://github.com/sebastienrousseau/scoop-bucket && scoop install noyalib` |
| Nix | `nix run github:sebastienrousseau/noyalib` |
| Container (GHCR) | `docker run --rm -v "$(pwd):/work" -w /work ghcr.io/sebastienrousseau/noyafmt:latest --check ci/*.yaml` |

Pre-built tarballs for **14 targets** (Linux gnu + musl, macOS
Intel + Apple Silicon + universal, Windows x86_64 + i686 +
aarch64) are attached to every GitHub Release. Each artefact is
signed with cosign keyless and carries a SLSA L3 build provenance
attestation — see
[Verification](#verification) for the verify commands or
[`pkg/VERIFY.md`](https://github.com/sebastienrousseau/noyalib/blob/main/pkg/VERIFY.md)
for the full cookbook.

**MSRV: Rust 1.85.0.** The `clap` dep tree pulls
`clap_builder 4.6` (edition 2024); the noyalib core library
itself stays at 1.75.

---

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
[`crates/noya-cli/examples/`](examples/):

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

If installing via `cargo install`, regenerate locally:

```bash
git clone https://github.com/sebastienrousseau/noyalib
cd noyalib
cargo xtask completions    # writes complete/{noyafmt,noyavalidate}.{bash,fish,zsh,ps1}
cargo xtask manpages       # writes doc/{noyafmt,noyavalidate}.1
```

---

## Verification

Every release artefact ships with a cosign keyless signature
(`<artefact>.sig` + `<artefact>.pem`) and a SLSA L3 build
provenance attestation. Verify before trusting a downloaded
binary:

```bash
COSIGN_EXPERIMENTAL=1 cosign verify-blob \
  --certificate-identity-regexp 'https://github.com/sebastienrousseau/noyalib/' \
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
  --certificate <artefact>.pem \
  --signature   <artefact>.sig \
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

- **Engineering policies** (MSRV, SemVer, security, performance, concurrency, platform support, feature flags):
  [`doc/POLICIES.md`](https://github.com/sebastienrousseau/noyalib/blob/main/doc/POLICIES.md)
- **Security policy**:
  [`SECURITY.md`](https://github.com/sebastienrousseau/noyalib/blob/main/SECURITY.md)
- **CLI flag reference**:
  [`doc/cli-reference.md`](https://github.com/sebastienrousseau/noyalib/blob/main/crates/noya-cli/doc/cli-reference.md)
- **Recipes (CI gates, pre-commit, editor integration)**:
  [`doc/recipes.md`](https://github.com/sebastienrousseau/noyalib/blob/main/crates/noya-cli/doc/recipes.md)
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
