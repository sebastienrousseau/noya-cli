// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! Argv parsing for both binaries.
//!
//! `noyafmt` and `noyavalidate` are run from CI pipelines, pre-commit
//! hooks and shell one-liners, so their argv is attacker-adjacent
//! whenever a filename is: a path with an embedded newline or a leading
//! dash arrives here before any of the crate's own logic runs.
//!
//! `try_get_matches_from` is the non-exiting entry point — the `get_`
//! variants call `exit()` on error, which a fuzzer reads as a crash. The
//! invariant is that parsing *returns*, whether Ok or Err.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = core::str::from_utf8(data) else {
        return;
    };
    // Split on NUL so one input yields a realistic multi-argument argv
    // rather than a single blob clap would reject immediately.
    let mut argv: Vec<&str> = s.split('\0').collect();
    argv.insert(0, "noyafmt");
    if argv.len() > 64 {
        argv.truncate(64);
    }

    let _ = noya_cli::noyafmt_command().try_get_matches_from(argv.iter());

    let mut argv2: Vec<&str> = s.split('\0').collect();
    argv2.insert(0, "noyavalidate");
    if argv2.len() > 64 {
        argv2.truncate(64);
    }
    let _ = noya_cli::noyavalidate_command().try_get_matches_from(argv2.iter());
});
