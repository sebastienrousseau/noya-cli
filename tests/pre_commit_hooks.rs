// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! The hosted pre-commit hooks in `.pre-commit-hooks.yaml` must be
//! able to fail. pre-commit passes the staged files as arguments and
//! fails a hook on a nonzero exit or on a file the hook modified.

use noyalib::Value;

fn hooks() -> Vec<Value> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/.pre-commit-hooks.yaml");
    let text = std::fs::read_to_string(path).unwrap();
    noyalib::from_str::<Vec<Value>>(&text).unwrap()
}

fn entry(id: &str) -> String {
    hooks()
        .into_iter()
        .find(|h| h["id"].as_str() == Some(id))
        .and_then(|h| h["entry"].as_str().map(str::to_owned))
        .unwrap_or_else(|| panic!("hook {id} missing"))
}

#[test]
fn noyafmt_hook_formats_in_place() {
    // Plain `noyafmt` prints to stdout and exits 0: the hook never
    // failed and never changed a file.
    assert_eq!(entry("noyafmt"), "noyafmt --write");
}

#[test]
fn noyafmt_check_hook_checks() {
    assert_eq!(entry("noyafmt-check"), "noyafmt --check");
}

#[test]
fn noyavalidate_hook_takes_the_files_pre_commit_passes() {
    assert_eq!(entry("noyavalidate"), "noyavalidate");
}
