<!-- SPDX-FileCopyrightText: 2026 Noyalib -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Architecture

`noya-cli` ships two binaries, `noyafmt` and `noyavalidate`, over one
shared library so the binaries, their manpages, and their shell
completions are generated from a single definition and cannot drift.

## Layers

- **`src/lib.rs`** defines the command-line surface once: the
  `clap` derive structs `NoyafmtCli` and `NoyavalidateCli`, and the
  builders `noyafmt_command` and `noyavalidate_command` that return
  the underlying `clap::Command` trees. Both binaries parse argv
  through these; the build script consumes the same trees.
- **`src/bin/noyafmt.rs`** formats YAML through noyalib's CST
  formatter with `rustfmt`-style ergonomics: `--check` for CI gates,
  `--write` for in-place rewrites, stdin to stdout for editors.
- **`src/bin/noyavalidate.rs`** validates syntax, renders errors with
  `miette`, validates each document against a JSON Schema 2020-12
  contract when `--schema` is given (the schema may be YAML or JSON),
  and with `--fix` rewrites the input through the CST formatter,
  normalising whitespace and quoting without changing meaning. The
  README documents the exit codes.
- **`build.rs`** generates the manpages (`clap_mangen`) and the
  bash, fish, zsh, and PowerShell completions (`clap_complete`) into
  `OUT_DIR` at build time. `make assets` copies them into `docs/` and
  `complete/` so the committed copies never lag `--help`; the version
  gate checks the manpages carry the crate version.

## Install contract

`GNUmakefile` implements the Unix contract: `make`, `make test`,
`make install` and `make uninstall` honouring `PREFIX` (default
`/usr/local`) and `DESTDIR`, installing binaries, manpages, and
completions to FHS paths. CI stages an install into a temporary
`DESTDIR` on every push. `pkg/` carries the Homebrew formula, Scoop
manifest, AUR `PKGBUILD`, and `flake.nix`, all generated from the
release build; the channel manifests roll after each release's
artefacts exist because they embed checksums.

## Testing

Integration tests run the binaries as processes. Two libFuzzer targets
feed arbitrary argv through the `clap` trees (`fuzz_argv`) and the
derive structs (`fuzz_derive_parse`); CI replays the seed corpus on
every push.

## Lockstep

The crate pins `noyalib` at the identical `=0.0.X` and releases with
it (core ADR-0005).
