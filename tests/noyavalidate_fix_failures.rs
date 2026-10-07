// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! `noyavalidate` failures after the input has parsed: `--fix` refused
//! by the formatter (exit 1), `--fix` unable to write its output
//! (exit 3), and stdin that is not UTF-8 (exit 3). In every case the
//! user's file must be left exactly as it was.

#![cfg(feature = "noyavalidate")]
#![allow(missing_docs)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn validate_bin() -> &'static str {
    env!("CARGO_BIN_EXE_noyavalidate")
}

/// A unique scratch directory per test; no `tempfile` dep in this crate.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("noya-cli-fixfail-{tag}-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn write_file(dir: &Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    fs::write(&p, body).expect("write fixture");
    p
}

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(validate_bin())
        .args(args)
        .output()
        .expect("spawn");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// A block scalar whose last line is whitespace with no final newline
/// (yaml-test-suite L24T): it parses, but the formatter cannot re-lay
/// it out without changing its value, so it refuses.
const UNFORMATTABLE: &str = "foo: |\n  x\n   ";

#[test]
fn fix_refused_by_the_formatter_exits_1_and_keeps_the_file() {
    let d = scratch("refused");
    let f = write_file(&d, "in.yaml", UNFORMATTABLE);
    let (code, stdout, stderr) = run(&["--fix", f.to_str().unwrap()]);
    assert_eq!(code, 1, "stderr: {stderr}");
    assert!(
        stderr.contains("error: applying --fix:") && stderr.contains("formatter rejected"),
        "stderr: {stderr}"
    );
    assert!(!stdout.contains("ok:"), "stdout: {stdout}");
    assert_eq!(fs::read_to_string(&f).unwrap(), UNFORMATTABLE);
}

#[test]
fn stdin_that_is_not_utf8_exits_3() {
    let mut child = Command::new(validate_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"a: \xff\xfe\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(3), "stderr: {stderr}");
    assert!(
        stderr.contains("error: reading input <stdin>"),
        "stderr: {stderr}"
    );
}

/// Write failures need a directory the binary cannot create its
/// temporary file in; unix permissions give that portably.
#[cfg(unix)]
mod write_failures {
    use super::{run, scratch, write_file};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    /// Make `dir` read-only for the duration of `body`, then restore it
    /// so the scratch directory can be cleaned up. Returns `None` when
    /// the directory is still writable (running as root), where the
    /// failure cannot be provoked and the test has nothing to check.
    fn read_only<T>(dir: &Path, body: impl FnOnce() -> T) -> Option<T> {
        fs::set_permissions(dir, fs::Permissions::from_mode(0o555)).unwrap();
        let probe = dir.join(".probe");
        let writable = fs::write(&probe, "").is_ok();
        let out = if writable {
            let _ = fs::remove_file(&probe);
            eprintln!("skipped: {} is writable despite mode 0555", dir.display());
            None
        } else {
            Some(body())
        };
        fs::set_permissions(dir, fs::Permissions::from_mode(0o755)).unwrap();
        out
    }

    fn fixture(tag: &str, input: &str) -> (PathBuf, PathBuf, PathBuf) {
        let d = scratch(tag);
        let schema_dir = scratch(&format!("{tag}-schema"));
        let schema = write_file(
            &schema_dir,
            "s.yaml",
            "type: [object, \"null\"]\nproperties:\n  port: { type: integer }\n",
        );
        let f = write_file(&d, "in.yaml", input);
        (d, f, schema)
    }

    fn assert_io_failure(result: Option<(i32, String, String)>, f: &Path, original: &str) {
        let Some((code, stdout, stderr)) = result else {
            return;
        };
        assert_eq!(code, 3, "stderr: {stderr}");
        assert!(
            stderr.contains("error: applying --fix:"),
            "stderr: {stderr}"
        );
        assert!(!stdout.contains("ok:"), "stdout: {stdout}");
        assert_eq!(fs::read_to_string(f).unwrap(), original);
    }

    #[test]
    fn fix_that_cannot_write_exits_3() {
        let input = "a:   1\n";
        let (d, f, _) = fixture("nowrite", input);
        let res = read_only(&d, || run(&["--fix", f.to_str().unwrap()]));
        assert_io_failure(res, &f, input);
    }

    #[test]
    fn fix_with_schema_that_cannot_write_its_coercion_exits_3() {
        let input = "port: \"8080\"\n";
        let (d, f, schema) = fixture("coerce-nowrite", input);
        let args = [
            "--fix",
            "--schema",
            schema.to_str().unwrap(),
            f.to_str().unwrap(),
        ];
        let res = read_only(&d, || run(&args));
        assert_io_failure(res, &f, input);
    }

    #[test]
    fn fix_with_schema_on_an_empty_file_that_cannot_write_exits_3() {
        let input = "# nothing but a comment\n";
        let (d, f, schema) = fixture("empty-nowrite", input);
        let args = [
            "--fix",
            "--schema",
            schema.to_str().unwrap(),
            f.to_str().unwrap(),
        ];
        let res = read_only(&d, || run(&args));
        assert_io_failure(res, &f, input);
    }
}
