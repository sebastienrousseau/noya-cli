// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! The core's ultra-complex fixture through both binaries: `noyavalidate`
//! accepts it, and `noyafmt`'s output projects onto the same JSON.

#![allow(missing_docs)]

use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ultra-complex")
        .join(name)
}

fn json_model(yaml: &str) -> Vec<serde_json::Value> {
    noyalib::load_all_as::<noyalib::Value>(yaml)
        .expect("parses")
        .into_iter()
        .map(|d| serde_json::to_value(d.untag()).expect("JSON model"))
        .collect()
}

#[test]
fn noyavalidate_accepts_the_fixture() {
    let out = Command::new(env!("CARGO_BIN_EXE_noyavalidate"))
        .arg(fixture("valid.yaml"))
        .output()
        .expect("run noyavalidate");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn noyafmt_output_projects_onto_the_expected_json() {
    let out = Command::new(env!("CARGO_BIN_EXE_noyafmt"))
        .arg(fixture("valid.yaml"))
        .output()
        .expect("run noyafmt");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let formatted = String::from_utf8(out.stdout).expect("utf-8");
    let expected: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(fixture("valid.json")).unwrap()).unwrap();
    assert_eq!(json_model(&formatted), expected, "formatted:\n{formatted}");
}
