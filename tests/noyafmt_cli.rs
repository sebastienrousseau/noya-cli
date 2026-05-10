// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! `noyafmt` CLI integration tests.
//!
//! Exercises the binary the way users will: spawn it, feed it stdin
//! or files, assert exit codes and output. The CLI contract is part
//! of the user-facing surface and should regress visibly if changed.

#![allow(missing_docs)]

use std::io::Write;
use std::process::{Command, Stdio};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_noyafmt")
}

fn fmt_stdin(input: &str, extra_args: &[&str]) -> (i32, String, String) {
    let mut cmd = Command::new(bin())
        .arg("--stdin")
        .args(extra_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn noyafmt");
    cmd.stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = cmd.wait_with_output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

#[test]
fn stdin_formats_messy_yaml() {
    let (code, stdout, _) = fmt_stdin("a:    1\nb:    2\n", &[]);
    assert_eq!(code, 0);
    assert_eq!(stdout, "a: 1\nb: 2\n");
}

#[test]
fn stdin_with_indent_4() {
    let (code, stdout, _) = fmt_stdin("a:\n  b: 1\n", &["--indent", "4"]);
    assert_eq!(code, 0);
    assert_eq!(stdout, "a:\n    b: 1\n");
}

#[test]
fn check_finds_unformatted_file() {
    let dir = tempdir();
    let path = dir.join("messy.yaml");
    std::fs::write(&path, "a:    1\n").unwrap();

    let out = Command::new(bin())
        .arg("--check")
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains(path.to_str().unwrap()),
        "stdout should list the unformatted path; got {stdout:?}",
    );
}

#[test]
fn check_clean_file_exits_zero() {
    let dir = tempdir();
    let path = dir.join("clean.yaml");
    std::fs::write(&path, "a: 1\n").unwrap();

    let out = Command::new(bin())
        .arg("--check")
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty());
}

#[test]
fn write_rewrites_file_in_place() {
    let dir = tempdir();
    let path = dir.join("messy.yaml");
    std::fs::write(&path, "a:    1\n").unwrap();

    let out = Command::new(bin())
        .arg("--write")
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "a: 1\n");
}

#[test]
fn check_and_write_are_mutually_exclusive() {
    let out = Command::new(bin())
        .arg("--check")
        .arg("--write")
        .arg("dummy")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8(out.stderr).unwrap();
    // clap reports mutually-exclusive flags as
    // `the argument '--check' cannot be used with '--write'`.
    assert!(stderr.contains("cannot be used with"));
}

#[test]
fn stdin_and_files_are_mutually_exclusive() {
    let out = Command::new(bin())
        .arg("--stdin")
        .arg("foo.yaml")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn no_args_is_usage_error() {
    let out = Command::new(bin()).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn help_prints_usage_and_exits_zero() {
    let out = Command::new(bin()).arg("--help").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("noyafmt"));
    // clap renders the usage section as `Usage:` (mixed case, no
    // colon-prefix the way the hand-rolled USAGE block did).
    assert!(stdout.contains("Usage:"));
}

#[test]
fn version_prints_crate_version() {
    let out = Command::new(bin()).arg("--version").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("noyafmt "));
}

// ── helpers ──────────────────────────────────────────────────────────

fn tempdir() -> std::path::PathBuf {
    let base = std::env::temp_dir();
    let name = format!(
        "noyafmt-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let dir = base.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
