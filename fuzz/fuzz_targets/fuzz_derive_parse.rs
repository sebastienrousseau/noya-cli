// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! The derive path: argv all the way into the typed structs.
//!
//! `fuzz_argv` stops at `Command::try_get_matches_from`, which only
//! establishes that clap accepted the shape. This target goes one layer
//! further, through `Parser::try_parse_from`, where matches are converted
//! into `NoyafmtCli` / `NoyavalidateCli` — and conversion is where typed
//! extraction can fail rather than merely mismatch.
//!
//! `--indent` is the interesting one: it lands in a `usize`, so the
//! fuzzer gets to try values that are negative, absent, enormous, or not
//! numbers at all. `conflicts_with` on `--check` / `--write` / `--stdin`
//! is exercised at the same time, since a conflict is resolved during
//! this step too.
//!
//! Both must return a `Result`. `Parser::parse_from` would call `exit()`
//! on error, which a fuzzer reads as a crash, so the `try_` variants are
//! the only correct entry points here.

#![no_main]

use clap::Parser;
use libfuzzer_sys::fuzz_target;
use noya_cli::{NoyafmtCli, NoyavalidateCli};

fuzz_target!(|data: &[u8]| {
    let Ok(s) = core::str::from_utf8(data) else {
        return;
    };
    // NUL-separated so one input becomes a realistic multi-argument argv.
    let mut argv: Vec<&str> = s.split('\0').collect();
    if argv.len() > 64 {
        argv.truncate(64);
    }

    let mut fmt_argv = argv.clone();
    fmt_argv.insert(0, "noyafmt");
    let _ = NoyafmtCli::try_parse_from(fmt_argv);

    argv.insert(0, "noyavalidate");
    let _ = NoyavalidateCli::try_parse_from(argv);
});
