// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! File contents and file names are untrusted. Control characters in
//! them (ANSI escapes, OSC title sequences, bell) must never reach the
//! terminal or a CI log raw; they are shown as U+FFFD instead.

#![cfg(feature = "noyavalidate")]

use std::path::PathBuf;
use std::process::Command;

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("noya_cli_escape_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn run(bin: &str, args: &[&std::ffi::OsStr]) -> (i32, String, String) {
    let out = Command::new(bin)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

fn assert_clean(what: &str, text: &str) {
    let bad: Vec<char> = text
        .chars()
        .filter(|c| c.is_control() && *c != '\n' && *c != '\t')
        .collect();
    assert!(
        bad.is_empty(),
        "{what} carries raw control characters {bad:?}: {text:?}"
    );
}

const HOSTILE: &str =
    "a: \"\u{1b}]0;PWNED\u{7}\u{1b}[2J\u{1b}[31mfake ok\u{1b}[0m\"\nb: [unclosed\n";

#[test]
fn parse_error_snippet_escapes_control_characters() {
    let d = dir("parse");
    let f = d.join("esc.yaml");
    std::fs::write(&f, HOSTILE).unwrap();
    let (code, _, stderr) = run(env!("CARGO_BIN_EXE_noyavalidate"), &[f.as_os_str()]);
    assert_eq!(code, 1);
    assert_clean("stderr", &stderr);
    assert!(stderr.contains('\u{fffd}'), "{stderr}");
}

#[test]
fn schema_violation_message_escapes_control_characters() {
    let d = dir("schema");
    let schema = d.join("s.yaml");
    std::fs::write(&schema, "type: object\nrequired: [\"\u{1b}[2Jx\"]\n").unwrap();
    let f = d.join("k.yaml");
    std::fs::write(&f, "k: v\n").unwrap();
    let (code, _, stderr) = run(
        env!("CARGO_BIN_EXE_noyavalidate"),
        &["--schema".as_ref(), schema.as_os_str(), f.as_os_str()],
    );
    assert_eq!(code, 1);
    assert_clean("stderr", &stderr);
}

#[test]
fn diagnostic_labels_still_point_at_the_right_text_after_escaping() {
    // The escape changes byte lengths; the caret must still land on
    // the unclosed flow sequence on line 2, not drift.
    let d = dir("span");
    let f = d.join("esc.yaml");
    std::fs::write(&f, HOSTILE).unwrap();
    let (_, _, stderr) = run(env!("CARGO_BIN_EXE_noyavalidate"), &[f.as_os_str()]);
    assert!(stderr.contains("b: [unclosed"), "{stderr}");
}

#[cfg(unix)]
#[test]
fn printed_file_names_are_escaped() {
    let d = dir("names");
    let f = d.join("x\u{1b}[31mred.yaml");
    std::fs::write(&f, "a:    1\n").unwrap();
    let (code, stdout, _) = run(
        env!("CARGO_BIN_EXE_noyafmt"),
        &["--check".as_ref(), f.as_os_str()],
    );
    assert_eq!(code, 1);
    assert_clean("noyafmt --check stdout", &stdout);
    assert!(stdout.contains("x\u{fffd}[31mred.yaml"), "{stdout:?}");

    let (_, stdout, _) = run(env!("CARGO_BIN_EXE_noyavalidate"), &[f.as_os_str()]);
    assert_clean("noyavalidate stdout", &stdout);
}

#[test]
fn schema_reports_are_capped_per_input() {
    let d = dir("cap");
    let schema = d.join("s.yaml");
    std::fs::write(&schema, "type: object\nrequired: [name]\n").unwrap();
    let f = d.join("many.yaml");
    let body: String = (0..120).map(|i| format!("---\nother: {i}\n")).collect();
    std::fs::write(&f, body).unwrap();
    let (code, _, stderr) = run(
        env!("CARGO_BIN_EXE_noyavalidate"),
        &["--schema".as_ref(), schema.as_os_str(), f.as_os_str()],
    );
    assert_eq!(code, 1);
    assert_eq!(stderr.matches("[document ").count(), 50, "{stderr}");
    assert!(stderr.contains("70 more"), "{stderr}");
}
