// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! Error-path and side-effecting coverage for both binaries.
//!
//! The existing suites cover the happy paths and argv validation. The
//! branches exercised here are the ones a user actually hits when
//! something is wrong — unparsable input, an unreadable file, a broken
//! schema — plus `--write`, which is the only path that mutates the
//! user's files and so should regress loudly.

#![allow(missing_docs)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn fmt_bin() -> &'static str {
    env!("CARGO_BIN_EXE_noyafmt")
}
fn validate_bin() -> &'static str {
    env!("CARGO_BIN_EXE_noyavalidate")
}

/// A unique scratch directory per test — no `tempfile` dep in this crate.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("noya-cli-cov-{tag}-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn write_file(dir: &Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    fs::write(&p, body).expect("write fixture");
    p
}

fn run(bin: &str, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(bin).args(args).output().expect("spawn");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn run_stdin(bin: &str, args: &[&str], input: &str) -> (i32, String, String) {
    let mut child = Command::new(bin)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

// ── noyafmt ──────────────────────────────────────────────────────────

/// An unparsable FILE takes the `Err(e)` arm of the per-file loop:
/// the path and error go to stderr and the process exits 1.
#[test]
fn noyafmt_unparsable_file_reports_and_exits_1() {
    let d = scratch("fmt-bad");
    let f = write_file(&d, "bad.yaml", "a: [1, 2\n");
    let (code, _, stderr) = run(fmt_bin(), &[f.to_str().unwrap()]);
    assert_eq!(code, 1, "stderr was: {stderr}");
    assert!(
        stderr.contains("bad.yaml"),
        "error should name the offending file: {stderr}"
    );
}

/// A missing FILE hits the same error arm via the read failure rather
/// than the parse failure.
#[test]
fn noyafmt_missing_file_reports_and_exits_1() {
    let (code, _, stderr) = run(fmt_bin(), &["definitely-not-here-12345.yaml"]);
    assert_eq!(code, 1);
    assert!(!stderr.is_empty(), "expected an error on stderr");
}

/// Unparsable stdin takes the `Err` arm of `run_stdin`.
#[test]
fn noyafmt_unparsable_stdin_exits_1() {
    let (code, _, stderr) = run_stdin(fmt_bin(), &["--stdin"], "a: [1, 2\n");
    assert_eq!(code, 1);
    assert!(stderr.contains("error"), "stderr was: {stderr}");
}

/// `--write` on an unformatted file rewrites it in place and leaves an
/// already-formatted file byte-identical (the `changed` short-circuit).
#[test]
fn noyafmt_write_rewrites_only_when_changed() {
    let d = scratch("fmt-write");
    let f = write_file(&d, "messy.yaml", "a:    1\nb:    2\n");
    let (code, _, stderr) = run(fmt_bin(), &["--write", f.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {stderr}");
    let after = fs::read_to_string(&f).unwrap();
    assert_eq!(after, "a: 1\nb: 2\n", "--write should reformat in place");

    // Second run: already formatted, so the write branch is skipped.
    let mtime_before = fs::metadata(&f).unwrap().len();
    let (code2, _, _) = run(fmt_bin(), &["--write", f.to_str().unwrap()]);
    assert_eq!(code2, 0);
    assert_eq!(fs::read_to_string(&f).unwrap(), after);
    assert_eq!(fs::metadata(&f).unwrap().len(), mtime_before);
}

/// `--check` prints the path of each unformatted file to stdout.
#[test]
fn noyafmt_check_lists_unformatted_paths() {
    let d = scratch("fmt-check");
    let f = write_file(&d, "needs.yaml", "a:    1\n");
    let (_, stdout, _) = run(fmt_bin(), &["--check", f.to_str().unwrap()]);
    assert!(
        stdout.contains("needs.yaml"),
        "--check should print the path: {stdout:?}"
    );
}

// ── noyavalidate ─────────────────────────────────────────────────────

/// An unparsable schema takes the schema-parse `Err` arm and exits 1
/// with a miette report rather than panicking.
#[test]
fn noyavalidate_unparsable_schema_exits_1() {
    let d = scratch("val-schema");
    let doc = write_file(&d, "doc.yaml", "a: 1\n");
    let schema = write_file(&d, "schema.yaml", "type: [object\n");
    let (code, _, stderr) = run(
        validate_bin(),
        &["--schema", schema.to_str().unwrap(), doc.to_str().unwrap()],
    );
    assert_eq!(code, 1, "stderr: {stderr}");
    assert!(
        stderr.contains("schema"),
        "error should mention the schema: {stderr}"
    );
}

/// A missing schema file is an I/O failure, distinct from a parse
/// failure, and must still exit non-zero rather than panicking.
#[test]
fn noyavalidate_missing_schema_exits_nonzero() {
    let d = scratch("val-noschema");
    let doc = write_file(&d, "doc.yaml", "a: 1\n");
    let (code, _, stderr) = run(
        validate_bin(),
        &["--schema", "no-such-schema.yaml", doc.to_str().unwrap()],
    );
    assert_ne!(code, 0);
    assert!(!stderr.is_empty());
}

/// `--fix` on input the formatter rejects takes the InvalidData arm and
/// maps it to exit 1 (not the generic 3).
#[test]
fn noyavalidate_fix_on_unparsable_input_exits_1() {
    let d = scratch("val-fix");
    let f = write_file(&d, "broken.yaml", "a: [1, 2\n");
    let (code, _, stderr) = run(validate_bin(), &["--fix", f.to_str().unwrap()]);
    assert_eq!(code, 1, "stderr: {stderr}");
    assert!(stderr.contains("fix") || stderr.contains("error"));
}

/// `--fix` with a schema whose own text is unparsable must fail on the
/// schema before touching the document.
#[test]
fn noyavalidate_fix_with_broken_schema_exits_1() {
    let d = scratch("val-fixschema");
    let doc = write_file(&d, "doc2.yaml", "a: 1\n");
    let schema = write_file(&d, "schema2.yaml", "type: [object\n");
    let (code, _, stderr) = run(
        validate_bin(),
        &[
            "--fix",
            "--schema",
            schema.to_str().unwrap(),
            doc.to_str().unwrap(),
        ],
    );
    assert_eq!(code, 1, "stderr: {stderr}");
}

/// `--fix` on well-formed but unformatted YAML takes the success arm of
/// `run_fix` and rewrites the file in place.
#[test]
fn noyavalidate_fix_reformats_valid_file() {
    let d = scratch("val-fixok");
    let f = write_file(&d, "messy.yaml", "a:    1\nb:    2\n");
    let (code, _, stderr) = run(validate_bin(), &["--fix", f.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {stderr}");
    let after = fs::read_to_string(&f).unwrap();
    assert!(
        !after.contains("a:    1"),
        "--fix should normalise whitespace, got: {after:?}"
    );
}

/// `--fix --schema` with a *valid* schema reaches `run_fix_with_schema`
/// — the broken-schema test above exits before ever getting here.
#[test]
fn noyavalidate_fix_with_valid_schema_coerces_and_writes() {
    let d = scratch("val-fixcoerce");
    // `port` is a quoted string in the source but an integer per schema;
    // the CST coercer should rewrite it losslessly.
    let doc = write_file(&d, "cfg.yaml", "port: \"8080\"\n");
    let schema = write_file(
        &d,
        "cfg.schema.yaml",
        "type: object\nproperties:\n  port:\n    type: integer\n",
    );
    let (code, _, stderr) = run(
        validate_bin(),
        &[
            "--fix",
            "--schema",
            schema.to_str().unwrap(),
            doc.to_str().unwrap(),
        ],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    let after = fs::read_to_string(&doc).unwrap();
    assert!(
        after.contains("8080"),
        "value should survive the fix: {after:?}"
    );
}

/// `--fix --schema` where the document cannot be coerced into the
/// schema takes the `still_invalid` short-circuit: nothing is written
/// and the run reports the residual violation.
#[test]
fn noyavalidate_fix_with_schema_leaves_uncoercible_untouched() {
    let d = scratch("val-fixresidue");
    let doc = write_file(&d, "bad.yaml", "port: not-a-number\n");
    let before = fs::read_to_string(&doc).unwrap();
    let schema = write_file(
        &d,
        "bad.schema.yaml",
        "type: object\nproperties:\n  port:\n    type: integer\n",
    );
    let (code, _, _) = run(
        validate_bin(),
        &[
            "--fix",
            "--schema",
            schema.to_str().unwrap(),
            doc.to_str().unwrap(),
        ],
    );
    assert_ne!(code, 0, "uncoercible input should not report success");
    assert_eq!(
        fs::read_to_string(&doc).unwrap(),
        before,
        "uncoercible input must not be rewritten"
    );
}

/// Schema validation without `--fix` on a conforming document is the
/// plain success path.
#[test]
fn noyavalidate_schema_validation_passes_on_conforming_doc() {
    let d = scratch("val-ok");
    let doc = write_file(&d, "ok.yaml", "port: 8080\n");
    let schema = write_file(
        &d,
        "ok.schema.yaml",
        "type: object\nproperties:\n  port:\n    type: integer\n",
    );
    let (code, _, stderr) = run(
        validate_bin(),
        &["--schema", schema.to_str().unwrap(), doc.to_str().unwrap()],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
}

/// `noyafmt FILE` with neither `--check` nor `--write` falls through to
/// the default arm: the formatted source goes to stdout and the file on
/// disk is left untouched.
#[test]
fn noyafmt_default_prints_to_stdout_without_touching_file() {
    let d = scratch("fmt-default");
    let f = write_file(&d, "plain.yaml", "a:    1\n");
    let (code, stdout, stderr) = run(fmt_bin(), &[f.to_str().unwrap()]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout, "a: 1\n", "formatted source should go to stdout");
    assert_eq!(
        fs::read_to_string(&f).unwrap(),
        "a:    1\n",
        "default mode must not modify the file"
    );
}

/// `noyavalidate --fix` with no FILE reads stdin and takes the
/// `path: None` arm, writing the fixed document to stdout.
#[test]
fn noyavalidate_fix_from_stdin_writes_to_stdout() {
    let (code, stdout, stderr) = run_stdin(validate_bin(), &["--fix"], "a:    1\nb:    2\n");
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("a: 1"),
        "fixed output should reach stdout: {stdout:?}"
    );
}

/// Plain syntax check from stdin with no flags — the Phase-1-only path.
#[test]
fn noyavalidate_plain_stdin_check_succeeds() {
    let (code, _, stderr) = run_stdin(validate_bin(), &[], "a: 1\n");
    assert_eq!(code, 0, "stderr: {stderr}");
}
