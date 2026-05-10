// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! Shared CLI surface for the `noyafmt` and `noyavalidate` binaries.
//!
//! The same [`clap::Command`] builders the binaries use to parse
//! their argv at runtime are also consumed by the build script
//! (`build.rs`) and the `cargo xtask` runner — so the binaries,
//! the man pages, and the shell completions can never drift.
//!
//! # Surface
//!
//! - [`NoyafmtCli`] / [`NoyavalidateCli`] — the parsed-args structs
//!   produced by `clap`'s derive macros. `main()` in each binary
//!   matches against fields of these.
//! - [`noyafmt_command`] / [`noyavalidate_command`] — the
//!   underlying [`clap::Command`] tree. Used by `clap_complete` and
//!   `clap_mangen` to generate completions and man pages
//!   respectively.
//!
//! # Cargo features
//!
//! This crate exposes no optional features of its own — both
//! binaries always ship with the same dispatch surface. The
//! transitive `noyalib` dependency is consumed with its **default
//! feature set** (`std` + the always-on parser / serializer /
//! Value / CST). To opt into optional `noyalib` features
//! (`schema`, `parallel`, `miette`, …), pin the version directly
//! and select features at the consuming binary's `Cargo.toml`;
//! the `noyalib` feature matrix is canonicalised in
//! [`crates/noyalib/src/lib.rs`](https://docs.rs/noyalib).
//!
//! # MSRV
//!
//! **Rust 1.85.0** stable. The `clap_builder` 4.6 dep pulls
//! edition-2024 helpers and floors the MSRV at 1.85; the core
//! `noyalib` library still builds on **1.75**. CI verifies both
//! floors via the `Per-crate MSRV` workflow job. The bump
//! policy is documented in the workspace
//! [`POLICIES.md`](https://github.com/sebastienrousseau/noyalib/blob/main/doc/POLICIES.md#1-msrv-minimum-supported-rust-version).
//!
//! # Panics
//!
//! Public functions in this crate do not panic. The two
//! binaries (`noyafmt`, `noyavalidate`) handle argv-parse
//! failures via clap's error path — surfaced as exit code `2`,
//! never as a panic.
//!
//! # Errors
//!
//! Binary exit codes follow Unix convention:
//! `0` on success, `1` on a YAML/schema problem, `2` on
//! argv-parse error. Library-side errors flow through
//! [`noyalib::Error`] (not re-exported here — call sites
//! that want library access should depend on `noyalib`
//! directly).
//!
//! # Concurrency
//!
//! `NoyafmtCli` / `NoyavalidateCli` are `Send + Sync` (plain
//! POD parsed-args structs). The `clap::Command` builders
//! return owned values; cheap to clone. No interior mutability.
//!
//! # Platform support
//!
//! Tier-1 (CI-verified each PR): `aarch64-apple-darwin`,
//! `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`. Both
//! binaries write via an *atomic file replacement* pattern
//! (write to a sibling temp file → `sync_all` → `rename`), so
//! concurrent readers always see either the pre-edit or the
//! post-edit contents — never a half-written truncation.
//!
//! # Performance
//!
//! Each YAML file in argv flows through the underlying
//! `noyalib::cst::parse_document` call (formatter) or
//! `noyalib::from_str::<Value>` (validator) — both run in
//! `O(n)` over input bytes. Argv-batch processing is sequential
//! by design (deterministic exit code on the first failure);
//! pipelines that need parallelism should fan out via `xargs -P`
//! at the shell layer rather than burying threading in the CLI.
//! End-to-end overhead per file: parse + serialise dominates;
//! argv parsing and file I/O are negligible (<1 ms) for files
//! up to a few MiB.
//!
//! # Security
//!
//! `#![forbid(unsafe_code)]` (workspace lint). No FFI. No
//! network I/O. The binaries only read files passed on argv;
//! they do not read environment variables. Resource-limit
//! gates are inherited from `noyalib`'s `ParserConfig`
//! defaults; pass `--strict` to opt into the tighter
//! `ParserConfig::strict()` preset. Full posture:
//! [`SECURITY.md`](https://github.com/sebastienrousseau/noyalib/blob/main/SECURITY.md).
//!
//! # API stability and SemVer
//!
//! Pre-1.0 (`0.0.x`): the argv contract (long flags, exit
//! codes, stdin/stdout shape) is **stable** within a 0.0.x
//! line — bug fixes only. Adding a new flag is allowed within
//! a 0.0.x bump; removing or renaming a flag, or repurposing
//! an exit code, is held to a 0.x bump (e.g. 0.0.x → 0.1.0).
//! The Rust library surface (`NoyafmtCli`, `NoyavalidateCli`,
//! `noyafmt_command`, `noyavalidate_command`) is also covered by
//! the workspace SemVer policy in
//! [`POLICIES.md`](https://github.com/sebastienrousseau/noyalib/blob/main/doc/POLICIES.md#2-semver--api-stability).
//! `cargo-semver-checks` runs in CI on every PR and blocks
//! accidental SemVer-incompatible changes.
//!
//! # Documentation
//!
//! - **Engineering policies** — workspace
//!   [`POLICIES.md`](https://github.com/sebastienrousseau/noyalib/blob/main/doc/POLICIES.md)
//!   covers MSRV, SemVer, security, performance, concurrency,
//!   platform support, feature flags.
//! - **CLI flag reference**:
//!   [`doc/cli-reference.md`](https://github.com/sebastienrousseau/noyalib/blob/main/crates/noya-cli/doc/cli-reference.md).
//! - **Recipes** (pre-commit, CI gate, schema validation, k8s,
//!   Helm, Compose, GitHub Actions): the
//!   [`examples/`](https://github.com/sebastienrousseau/noyalib/tree/main/crates/noya-cli/examples)
//!   directory.

use clap::{CommandFactory, Parser};
use std::path::PathBuf;

/// CLI surface for `noyafmt` — the YAML formatter.
///
/// Mirrors the `rustfmt` / `prettier` ergonomics so it slots into
/// existing developer workflows: `--check` for CI gates, `--write`
/// for in-place rewrites, stdin/stdout for editor integration.
#[derive(Debug, Parser)]
#[command(
    name = "noyafmt",
    about = "Format YAML files via the noyalib CST formatter",
    long_about = "noyafmt — auto-format YAML via the noyalib CST.\n\n\
                  Reads YAML from FILE arguments (or stdin via --stdin) and\n\
                  rewrites them through noyalib's lossless CST formatter.\n\
                  Comments, anchor positions, and document structure are\n\
                  preserved byte-for-byte; only whitespace and quoting are\n\
                  normalised.",
    version = env!("CARGO_PKG_VERSION"),
    after_help = "EXAMPLES:\n  \
                  noyafmt config.yaml               # print formatted source to stdout\n  \
                  noyafmt --write config.yaml       # rewrite in place\n  \
                  noyafmt --check ci/*.yaml         # CI gate\n  \
                  cat foo.yaml | noyafmt --stdin",
)]
pub struct NoyafmtCli {
    /// Verify each FILE is formatted; print the list of files that
    /// need formatting and exit 1 if any do. Non-destructive.
    /// Suitable as a pre-commit / CI gate.
    #[arg(long, conflicts_with = "write")]
    pub check: bool,

    /// Rewrite each FILE in place. Default is to print the formatted
    /// source to stdout.
    #[arg(long)]
    pub write: bool,

    /// Read from stdin, write to stdout. Mutually exclusive with
    /// FILE arguments.
    #[arg(long, conflicts_with = "files")]
    pub stdin: bool,

    /// Indentation width in spaces.
    #[arg(long, value_name = "N", default_value_t = 2)]
    pub indent: usize,

    /// YAML files to format. Pass `--stdin` to read from stdin
    /// instead.
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

/// CLI surface for `noyavalidate` — the YAML validator.
///
/// Validates YAML syntax, optionally enforces a JSON Schema 2020-12
/// contract, and can normalise the input through the lossless CST
/// formatter via `--fix`.
#[derive(Debug, Parser)]
#[command(
    name = "noyavalidate",
    about = "Validate YAML syntax and (optionally) a JSON Schema",
    long_about = "noyavalidate — check YAML syntax (and optional JSON Schema).\n\n\
                  Reads one or more YAML documents from a file (or stdin),\n\
                  reports syntax errors via the miette fancy renderer, and —\n\
                  when --schema PATH is given — validates each parsed\n\
                  document against a JSON Schema 2020-12 contract (the\n\
                  schema may itself be written in YAML or JSON).\n\n\
                  --fix rewrites the input in-place through the lossless\n\
                  CST formatter, normalising whitespace and quoting without\n\
                  changing semantics. When the input is stdin, the\n\
                  formatted output is written to stdout instead.",
    version = env!("CARGO_PKG_VERSION"),
    after_help = "EXIT CODES:\n  \
                  0    All documents valid (and fixed if --fix)\n  \
                  1    Parse error or schema violation\n  \
                  2    Usage error\n  \
                  3    I/O error",
)]
pub struct NoyavalidateCli {
    /// Validate each document against the JSON Schema 2020-12 at
    /// PATH (the schema may itself be YAML or JSON).
    #[arg(short = 's', long, value_name = "PATH")]
    pub schema: Option<PathBuf>,

    /// Rewrite FILE in place via the CST formatter (lossless:
    /// byte-faithful for everything except normalised whitespace
    /// and line endings). With stdin input, the formatted bytes go
    /// to stdout.
    #[arg(long)]
    pub fix: bool,

    /// Suppress success output.
    #[arg(short, long)]
    pub quiet: bool,

    /// YAML file to validate. Use `-` or omit for stdin.
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}

/// Build the [`clap::Command`] for `noyafmt`.
///
/// Used by the build script and `cargo xtask` to drive
/// `clap_complete` and `clap_mangen` against the same Command tree
/// the binary uses at runtime.
#[must_use]
pub fn noyafmt_command() -> clap::Command {
    NoyafmtCli::command()
}

/// Build the [`clap::Command`] for `noyavalidate`.
///
/// Used by the build script and `cargo xtask` to drive
/// `clap_complete` and `clap_mangen` against the same Command tree
/// the binary uses at runtime.
#[must_use]
pub fn noyavalidate_command() -> clap::Command {
    NoyavalidateCli::command()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── noyafmt parsing ───────────────────────────────────────────
    #[test]
    fn noyafmt_help_flag_renders() {
        let r = NoyafmtCli::try_parse_from(["noyafmt", "--help"]);
        let err = r.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn noyafmt_version_flag_renders() {
        let r = NoyafmtCli::try_parse_from(["noyafmt", "--version"]);
        let err = r.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
    }

    #[test]
    fn noyafmt_check_with_files() {
        let cli = NoyafmtCli::try_parse_from(["noyafmt", "--check", "a.yaml", "b.yaml"]).unwrap();
        assert!(cli.check);
        assert!(!cli.write);
        assert_eq!(cli.files.len(), 2);
    }

    #[test]
    fn noyafmt_write_with_file() {
        let cli = NoyafmtCli::try_parse_from(["noyafmt", "--write", "x.yaml"]).unwrap();
        assert!(cli.write);
        assert_eq!(cli.files.len(), 1);
    }

    #[test]
    fn noyafmt_stdin_alone() {
        let cli = NoyafmtCli::try_parse_from(["noyafmt", "--stdin"]).unwrap();
        assert!(cli.stdin);
        assert!(cli.files.is_empty());
    }

    #[test]
    fn noyafmt_indent_separate_value() {
        let cli = NoyafmtCli::try_parse_from(["noyafmt", "--indent", "4", "--stdin"]).unwrap();
        assert_eq!(cli.indent, 4);
    }

    #[test]
    fn noyafmt_indent_eq_value() {
        let cli = NoyafmtCli::try_parse_from(["noyafmt", "--indent=8", "--stdin"]).unwrap();
        assert_eq!(cli.indent, 8);
    }

    #[test]
    fn noyafmt_indent_default_is_two() {
        let cli = NoyafmtCli::try_parse_from(["noyafmt", "--stdin"]).unwrap();
        assert_eq!(cli.indent, 2);
    }

    #[test]
    fn noyafmt_indent_non_numeric_errors() {
        let r = NoyafmtCli::try_parse_from(["noyafmt", "--indent", "abc", "--stdin"]);
        assert!(r.is_err());
    }

    #[test]
    fn noyafmt_unknown_option_errors() {
        let r = NoyafmtCli::try_parse_from(["noyafmt", "--frobnicate"]);
        assert!(r.is_err());
    }

    #[test]
    fn noyafmt_check_and_write_rejected() {
        let r = NoyafmtCli::try_parse_from(["noyafmt", "--check", "--write", "f.yaml"]);
        let err = r.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn noyafmt_stdin_with_files_rejected() {
        let r = NoyafmtCli::try_parse_from(["noyafmt", "--stdin", "f.yaml"]);
        let err = r.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    // ── noyavalidate parsing ──────────────────────────────────────
    #[test]
    fn noyavalidate_help_flag_renders() {
        let r = NoyavalidateCli::try_parse_from(["noyavalidate", "--help"]);
        assert_eq!(r.unwrap_err().kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn noyavalidate_schema_short_form() {
        let cli =
            NoyavalidateCli::try_parse_from(["noyavalidate", "-s", "s.json", "in.yaml"]).unwrap();
        assert_eq!(cli.schema.unwrap().to_string_lossy(), "s.json");
        assert_eq!(cli.file.unwrap().to_string_lossy(), "in.yaml");
    }

    #[test]
    fn noyavalidate_schema_long_form() {
        let cli =
            NoyavalidateCli::try_parse_from(["noyavalidate", "--schema=schema.yaml", "x.yaml"])
                .unwrap();
        assert_eq!(cli.schema.unwrap().to_string_lossy(), "schema.yaml");
    }

    #[test]
    fn noyavalidate_fix_quiet_flags() {
        let cli = NoyavalidateCli::try_parse_from(["noyavalidate", "--fix", "--quiet", "in.yaml"])
            .unwrap();
        assert!(cli.fix);
        assert!(cli.quiet);
    }

    #[test]
    fn noyavalidate_no_args_means_stdin() {
        let cli = NoyavalidateCli::try_parse_from(["noyavalidate"]).unwrap();
        assert!(cli.file.is_none());
    }

    // ── Command introspection (used by build.rs / xtask) ──────────
    #[test]
    fn commands_render_help_without_panic() {
        let mut a = noyafmt_command();
        let mut b = noyavalidate_command();
        let _ = a.render_help();
        let _ = b.render_help();
    }
}
