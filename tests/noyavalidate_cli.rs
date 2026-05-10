// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! Integration tests for the `noyavalidate` CLI.
//!
//! Uses the `CARGO_BIN_EXE_noyavalidate` env var Cargo injects for
//! same-crate integration tests. The whole module is gated on the
//! `noyavalidate` feature since that's what compiles the binary.

#![cfg(feature = "noyavalidate")]

use std::io::Write;
use std::process::{Command, Stdio};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_noyavalidate"))
}

fn run_with_stdin(stdin: &str, args: &[&str]) -> (i32, String, String) {
    let mut child = bin()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn noyavalidate");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

fn run_with_file(path: &std::path::Path, args: &[&str]) -> (i32, String, String) {
    let output = bin()
        .args(args)
        .arg(path)
        .output()
        .expect("spawn noyavalidate");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

fn tmp(name: &str, contents: &str) -> std::path::PathBuf {
    let dir =
        std::env::temp_dir().join(format!("noyavalidate_cli_{}_{}", name, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.yaml"));
    std::fs::write(&path, contents).unwrap();
    path
}

// ── Valid YAML ───────────────────────────────────────────────────────────

#[test]
fn valid_single_doc_file() {
    let p = tmp("single", "name: ok\nvalue: 42\n");
    let (code, stdout, _) = run_with_file(&p, &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("ok: 1 document valid"));
}

#[test]
fn valid_multi_doc_file() {
    let p = tmp("multi", "---\na: 1\n---\nb: 2\n---\nc: 3\n");
    let (code, stdout, _) = run_with_file(&p, &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("ok: 3 documents valid"));
}

#[test]
fn valid_quiet_suppresses_output() {
    let p = tmp("quiet", "name: ok\n");
    let (code, stdout, _) = run_with_file(&p, &["-q"]);
    assert_eq!(code, 0);
    assert!(
        stdout.is_empty(),
        "quiet mode must not print; got: {stdout:?}"
    );
}

#[test]
fn valid_long_quiet_flag() {
    let p = tmp("long_quiet", "name: ok\n");
    let (code, stdout, _) = run_with_file(&p, &["--quiet"]);
    assert_eq!(code, 0);
    assert!(stdout.is_empty());
}

// ── Stdin ────────────────────────────────────────────────────────────────

#[test]
fn valid_stdin_implicit() {
    let (code, stdout, _) = run_with_stdin("name: ok\n", &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("<stdin>"));
    assert!(stdout.contains("ok: 1 document valid"));
}

#[test]
fn valid_stdin_explicit_dash() {
    let (code, stdout, _) = run_with_stdin("name: ok\n", &["-"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("<stdin>"));
}

// ── Parse errors ─────────────────────────────────────────────────────────

#[test]
fn parse_error_exits_1() {
    let p = tmp("bad", "key: [1, 2, 3\nnext: v\n");
    let (code, _, stderr) = run_with_file(&p, &[]);
    assert_eq!(code, 1);
    // Diagnostic code identifies the kind of error.
    assert!(stderr.contains("noyalib::parse"), "stderr was: {stderr}");
}

#[test]
fn parse_error_includes_filename_in_diagnostic() {
    let p = tmp("named", "key: [unclosed\n");
    let (code, _, stderr) = run_with_file(&p, &[]);
    assert_eq!(code, 1);
    let name = p.display().to_string();
    assert!(
        stderr.contains(&name),
        "filename {name:?} missing from stderr: {stderr}"
    );
}

// ── Help / version ───────────────────────────────────────────────────────

#[test]
fn help_flag_short() {
    let output = bin().arg("-h").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // clap renders Usage: (mixed case) rather than the old USAGE: header.
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("--quiet"));
}

#[test]
fn help_flag_long() {
    let output = bin().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("EXIT CODES:"));
}

#[test]
fn version_flag_short() {
    let output = bin().arg("-V").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("noyavalidate "));
}

#[test]
fn version_flag_long() {
    let output = bin().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("noyavalidate "));
}

// ── Usage errors ─────────────────────────────────────────────────────────

#[test]
fn unknown_flag_exits_2() {
    let output = bin().arg("--bogus").output().unwrap();
    assert_eq!(output.status.code().unwrap(), 2);
    let stderr = String::from_utf8(output.stderr).unwrap();
    // clap reports unknown args as "unexpected argument".
    assert!(stderr.contains("unexpected argument"));
    assert!(stderr.contains("Usage:"));
}

#[test]
fn too_many_files_exits_2() {
    let output = bin().args(["a.yaml", "b.yaml"]).output().unwrap();
    assert_eq!(output.status.code().unwrap(), 2);
    let stderr = String::from_utf8(output.stderr).unwrap();
    // clap rejects extras as "unexpected argument 'b.yaml' found".
    assert!(stderr.contains("unexpected argument"));
}

#[test]
fn stdin_combined_with_file_exits_2() {
    // clap accepts `-` as the positional, so the second argument
    // collides as an unexpected positional. The old hand-rolled
    // parser rejected the combination explicitly; clap does it via
    // its standard "unexpected argument" path.
    let output = bin().args(["-", "a.yaml"]).output().unwrap();
    assert_eq!(output.status.code().unwrap(), 2);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("unexpected argument"));
}

// ── I/O errors ───────────────────────────────────────────────────────────

#[test]
fn missing_file_exits_3() {
    let output = bin()
        .arg("/tmp/__noyavalidate_definitely_not_a_real_file__.yaml")
        .output()
        .unwrap();
    assert_eq!(output.status.code().unwrap(), 3);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("reading input"));
}

// ── Positional `--` separator ────────────────────────────────────────────

#[test]
fn double_dash_allows_path_with_leading_dash() {
    let dir = std::env::temp_dir().join(format!("noyavalidate_dd_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("-weirdname.yaml");
    std::fs::write(&path, "ok: true\n").unwrap();
    let output = bin().args(["--"]).arg(&path).output().unwrap();
    assert_eq!(output.status.code().unwrap(), 0);
}

// ── Phase 3.2: --schema ──────────────────────────────────────────────────

#[test]
fn schema_flag_with_no_path_exits_2() {
    let output = bin().arg("--schema").output().unwrap();
    assert_eq!(output.status.code().unwrap(), 2);
    let stderr = String::from_utf8(output.stderr).unwrap();
    // clap reports "a value is required for '--schema <PATH>'".
    assert!(stderr.contains("--schema") && stderr.contains("value"));
}

#[test]
fn schema_match_exits_0() {
    let schema = tmp(
        "schema_ok",
        "type: object\nrequired: [port]\nproperties:\n  port: { type: integer }\n",
    );
    let yaml = tmp("data_ok", "port: 8080\n");
    let output = bin()
        .arg("--schema")
        .arg(&schema)
        .arg(&yaml)
        .output()
        .unwrap();
    assert_eq!(output.status.code().unwrap(), 0);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("schema-checked"));
}

#[test]
fn schema_short_flag_works() {
    let schema = tmp("schema_short", "type: object\nrequired: [x]\n");
    let yaml = tmp("data_short", "x: 1\n");
    let output = bin().arg("-s").arg(&schema).arg(&yaml).output().unwrap();
    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn schema_eq_form_works() {
    let schema = tmp("schema_eq", "type: object\nrequired: [x]\n");
    let yaml = tmp("data_eq", "x: 1\n");
    let arg = format!("--schema={}", schema.display());
    let output = bin().arg(arg).arg(&yaml).output().unwrap();
    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn schema_violation_exits_1() {
    let schema = tmp(
        "schema_v",
        "type: object\nrequired: [port]\nproperties:\n  port: { type: integer }\n",
    );
    let yaml = tmp("data_v", "port: not-int\n");
    let output = bin()
        .arg("--schema")
        .arg(&schema)
        .arg(&yaml)
        .output()
        .unwrap();
    assert_eq!(output.status.code().unwrap(), 1);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("schema violation"), "stderr: {stderr}");
}

#[test]
fn schema_missing_file_exits_3() {
    let yaml = tmp("data_msch", "port: 1\n");
    let output = bin()
        .arg("--schema")
        .arg("/tmp/__noyavalidate_no_such_schema__.yaml")
        .arg(&yaml)
        .output()
        .unwrap();
    assert_eq!(output.status.code().unwrap(), 3);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("reading schema"));
}

#[test]
fn schema_validates_each_doc_in_multi_doc_stream() {
    let schema = tmp("schema_multi", "type: object\nrequired: [port]\n");
    // Second document is missing the required field.
    let yaml = tmp("data_multi", "---\nport: 1\n---\nhost: x\n");
    let output = bin()
        .arg("--schema")
        .arg(&schema)
        .arg(&yaml)
        .output()
        .unwrap();
    assert_eq!(output.status.code().unwrap(), 1);
    let stderr = String::from_utf8(output.stderr).unwrap();
    // The label should disambiguate which document failed.
    assert!(stderr.contains("document 2"), "stderr: {stderr}");
}

// ── Phase 3.2: --fix ─────────────────────────────────────────────────────

#[test]
fn fix_normalises_whitespace_in_place() {
    let yaml = tmp("fix_in_place", "port:    8080\nhost:    localhost\n");
    let output = bin().arg("--fix").arg(&yaml).output().unwrap();
    assert_eq!(output.status.code().unwrap(), 0);
    let after = std::fs::read_to_string(&yaml).unwrap();
    assert_eq!(after, "port: 8080\nhost: localhost\n");
}

#[test]
fn fix_with_stdin_writes_clean_stdout() {
    let (code, stdout, _) = run_with_stdin("port:    8080\n", &["--fix"]);
    assert_eq!(code, 0);
    // No success chatter — the formatted bytes are the only stdout
    // content so downstream consumers can pipe through cleanly.
    assert_eq!(stdout, "port: 8080\n");
}

#[test]
fn fix_with_invalid_yaml_exits_1() {
    let yaml = tmp("fix_bad", "port: [unclosed\n");
    let output = bin().arg("--fix").arg(&yaml).output().unwrap();
    assert_eq!(output.status.code().unwrap(), 1);
}

#[test]
fn schema_and_fix_combine() {
    let schema = tmp(
        "combo_s",
        "type: object\nrequired: [port]\nproperties:\n  port: { type: integer }\n",
    );
    let yaml = tmp("combo_d", "port:    8080\n");
    let output = bin()
        .arg("--schema")
        .arg(&schema)
        .arg("--fix")
        .arg(&yaml)
        .output()
        .unwrap();
    assert_eq!(output.status.code().unwrap(), 0);
    let stdout = String::from_utf8(output.stdout).unwrap();
    // Already-valid input — `--fix` runs the formatter only and the
    // success message reports zero coercions.
    assert!(
        stdout.contains("schema-checked, no fixes needed"),
        "got: {stdout}"
    );
    let after = std::fs::read_to_string(&yaml).unwrap();
    assert_eq!(after, "port: 8080\n");
}

#[test]
fn schema_and_fix_coerces_quoted_integer() {
    // Headline `coerce_to_schema` use case: `port: "8080"` is a
    // string in YAML 1.2, but the schema says it must be an
    // integer. With `--schema --fix`, the CLI rewrites the file
    // through `noyalib::coerce_to_schema` so re-validation passes.
    let schema = tmp(
        "coerce_s",
        "type: object\nrequired: [port]\nproperties:\n  port: { type: integer }\n",
    );
    let yaml = tmp("coerce_d", "port: \"8080\"\n");
    let output = bin()
        .arg("--schema")
        .arg(&schema)
        .arg("--fix")
        .arg(&yaml)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code().unwrap(),
        0,
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("1 fix(es) applied"), "got: {stdout}");
    let after = std::fs::read_to_string(&yaml).unwrap();
    assert_eq!(after, "port: 8080\n");
}

#[test]
fn fix_skipped_when_schema_violates() {
    // Schema check happens before --fix. If the data is rejected,
    // the file must NOT be rewritten — that would silently wipe the
    // original buggy input.
    let schema = tmp("guard_s", "type: object\nrequired: [port]\n");
    let original = "host: localhost\n"; // missing required `port`
    let yaml = tmp("guard_d", original);
    let output = bin()
        .arg("--schema")
        .arg(&schema)
        .arg("--fix")
        .arg(&yaml)
        .output()
        .unwrap();
    assert_eq!(output.status.code().unwrap(), 1);
    let after = std::fs::read_to_string(&yaml).unwrap();
    assert_eq!(
        after, original,
        "fix must not run if schema rejected the input"
    );
}
