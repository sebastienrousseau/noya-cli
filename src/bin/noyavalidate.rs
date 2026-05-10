// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! `noyavalidate` — validate YAML syntax and (optionally) schema.
//!
//! Reads one or more YAML documents from a file (or stdin), reports
//! syntax errors via the `miette` fancy renderer, and — when
//! `--schema PATH` is given — validates each parsed document against
//! a JSON Schema 2020-12 contract (the schema may itself be written
//! in YAML or JSON; either parses).
//!
//! `--fix` rewrites the input in-place through the lossless CST
//! formatter (`noyalib::cst::format`), normalising whitespace and
//! quoting without changing semantics. When the input is stdin,
//! the formatted output is written to stdout instead.
//!
//! The argv-parsing surface lives in [`noya_cli::NoyavalidateCli`]
//! so the same Command tree feeds the binary, the build-time
//! codegen, and the `cargo xtask` runner.
//!
//! # Exit codes
//!
//! | Code | Meaning                                       |
//! |------|-----------------------------------------------|
//! | 0    | All documents valid (and fixed if --fix).     |
//! | 1    | Parse error or schema violation.              |
//! | 2    | Usage error (bad args).                       |
//! | 3    | I/O error (reading or writing).               |

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use miette::{NamedSource, Report};
use noya_cli::NoyavalidateCli;

fn read_input(path: Option<&Path>) -> io::Result<(String, String)> {
    match path {
        None => {
            let mut buf = String::new();
            let _ = io::stdin().read_to_string(&mut buf)?;
            Ok(("<stdin>".to_string(), buf))
        }
        Some(p) => {
            let source = fs::read_to_string(p)?;
            Ok((p.display().to_string(), source))
        }
    }
}

fn read_schema(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}

/// Run schema validation across every parsed document. Returns the
/// number of violations found (0 = success). Emits one miette report
/// per failing document so the user sees all issues in one pass.
fn run_schema_validation(
    docs: &[noyalib::Value],
    schema_text: &str,
    schema_path_label: &str,
    source_label: &str,
    full_source: &str,
) -> usize {
    // Parse the schema once.
    let schema: noyalib::Value = match noyalib::from_str(schema_text) {
        Ok(v) => v,
        Err(e) => {
            let report = Report::new(e)
                .with_source_code(NamedSource::new(schema_path_label, schema_text.to_owned()));
            eprintln!("error: parsing schema:");
            eprintln!("{report:?}");
            return 1;
        }
    };

    let mut violations = 0;
    for (i, doc) in docs.iter().enumerate() {
        if let Err(e) = noyalib::validate_against_schema(doc, &schema) {
            violations += 1;
            // For multi-document streams, prefix every diagnostic
            // with the doc number so the user knows which document
            // failed. miette's source-pointer label is empty for
            // span-less errors, so we surface this explicitly.
            if docs.len() > 1 {
                eprintln!("[document {}]", i + 1);
            }
            let report = Report::new(e)
                .with_source_code(NamedSource::new(source_label, full_source.to_owned()));
            eprintln!("{report:?}");
        }
    }
    violations
}

/// Run the lossless CST formatter and write the result back to
/// `path` (or to stdout if `path` is `None`). Returns Ok if the
/// write succeeded.
///
/// Used by the `--fix`-only path (no `--schema`). Comments and
/// formatting survive byte-faithfully.
fn run_fix(path: Option<&Path>, source: &str) -> io::Result<()> {
    let formatted = match noyalib::cst::format(source) {
        Ok(s) => s,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("--fix: formatter rejected the input: {e}"),
            ));
        }
    };
    match path {
        None => {
            let mut stdout = io::stdout().lock();
            stdout.write_all(formatted.as_bytes())?;
        }
        Some(p) => fs::write(p, formatted.as_bytes())?,
    }
    Ok(())
}

/// Outcome of [`run_fix_with_schema`] — the caller threads the
/// `applied` count into the success message and skips the
/// post-fix validate when [`Self::wrote`] is false.
struct FixOutcome {
    /// Number of coercions applied across all documents.
    applied: usize,
    /// Whether the formatted output was actually written to
    /// disk / stdout. The transactional contract: if validation
    /// of the coerced documents would still fail, leave the file
    /// alone so the user keeps their original (buggy) source.
    wrote: bool,
}

/// Apply schema-driven type coercion to the YAML source via
/// [`noyalib::cst::coerce_to_schema`], **transactionally**: only
/// rewrite the input if the coerced output is fully schema-valid.
///
/// The CST-aware coerce path preserves comments and indentation
/// around every coerced scalar — only the bytes of the changed
/// scalar are rewritten. Multi-document streams are handled
/// per-document; the document delimiters and inter-document
/// content survive untouched.
///
/// Behaviour:
///
/// 1. Parse the source via [`noyalib::cst::parse_stream`]
///    (multi-doc-aware).
/// 2. For each document, run [`noyalib::cst::coerce_to_schema`]
///    in-place — only string scalars whose schema-declared type
///    is integer / number / boolean are coerced; everything else
///    is preserved.
/// 3. Re-validate via [`noyalib::validate_against_schema`] on the
///    parsed [`noyalib::Value`] tree. If any violation remains, return
///    without writing — the caller surfaces the residue and
///    exits 1 with the user's original source intact.
/// 4. If validation passes, write the concatenated CST sources
///    back to `path` (or stdout). Comments and formatting
///    survive byte-faithfully.
fn run_fix_with_schema(
    path: Option<&Path>,
    source: &str,
    schema: &noyalib::Value,
) -> io::Result<FixOutcome> {
    // Parse the source as a CST stream. This is what unlocks the
    // comment-preserving path: every byte that isn't part of a
    // coerced scalar will round-trip verbatim.
    let mut docs = match noyalib::cst::parse_stream(source) {
        Ok(d) => d,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("--fix: parse stream: {e}"),
            ));
        }
    };

    let mut applied = 0usize;
    for cst_doc in docs.iter_mut() {
        match noyalib::cst::coerce_to_schema(cst_doc, schema) {
            Ok(n) => applied += n,
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("--fix: cst::coerce_to_schema failed: {e}"),
                ));
            }
        }
    }

    // Transactional gate: validate each coerced document. We have
    // to re-parse each CST back to a Value because
    // `validate_against_schema` operates on the `noyalib::Value`
    // shape; this also surfaces residue (e.g. `port: "abc"`
    // against `type: integer` — not coercible by parse).
    let still_invalid = docs.iter().any(|cst_doc| {
        match noyalib::from_str::<noyalib::Value>(&cst_doc.to_string()) {
            Ok(v) => noyalib::validate_against_schema(&v, schema).is_err(),
            Err(_) => true,
        }
    });
    if still_invalid {
        return Ok(FixOutcome {
            applied,
            wrote: false,
        });
    }

    // Concatenate CST sources for the final write — preserves
    // every untouched byte (including inter-document `---` /
    // `...` separators).
    let mut output = String::with_capacity(source.len());
    for cst_doc in &docs {
        output.push_str(&cst_doc.to_string());
    }

    // Always run the lossless formatter on top so the
    // `--fix --schema` and the `--fix`-only paths produce
    // equivalent whitespace shape. `cst::format` is itself
    // comment-preserving — only whitespace and quoting are
    // normalised. Falls back to the raw concatenation if the
    // formatter rejects the post-coerce text (defensive — would
    // signal a parser bug).
    let final_output = noyalib::cst::format(&output).unwrap_or(output);

    match path {
        None => {
            let mut stdout = io::stdout().lock();
            stdout.write_all(final_output.as_bytes())?;
        }
        Some(p) => fs::write(p, final_output.as_bytes())?,
    }
    Ok(FixOutcome {
        applied,
        wrote: true,
    })
}

fn run() -> ExitCode {
    let args = NoyavalidateCli::parse();

    // `-` as the positional means "explicitly read from stdin" — clap
    // accepts it as a valid PathBuf, so normalise it back to None
    // (the read path's None branch reads stdin).
    let path: Option<PathBuf> = match args.file {
        Some(ref p) if p.as_os_str() == "-" => None,
        other => other,
    };

    let (name, source) = match read_input(path.as_deref()) {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("error: reading input: {e}");
            return ExitCode::from(3);
        }
    };

    // Phase 1: syntax check. The CST-aware --fix path takes the
    // source string directly so it can preserve comments — these
    // parsed `Value`s are used only for validation reporting.
    let docs = match noyalib::load_all_as::<noyalib::Value>(&source) {
        Ok(d) => d,
        Err(e) => {
            let report = Report::new(e).with_source_code(NamedSource::new(name, source.clone()));
            eprintln!("{report:?}");
            return ExitCode::from(1);
        }
    };

    // Phase 2: schema check + optional autofix.
    //
    // Four flag combinations are possible:
    //
    // | `--schema` | `--fix` | Behaviour                                         |
    // | :---:      | :---:   | :---                                              |
    // | no         | no      | syntax check only (Phase 1).                      |
    // | no         | yes     | run lossless formatter (Phase 3).                 |
    // | yes        | no      | strict validate; exit 1 on violation.             |
    // | yes        | yes     | coerce → re-validate → format → write. Exits 1   |
    // |            |         | only if violations remain *after* coercion.       |
    //
    // The `--fix --schema` path uses [`noyalib::coerce_to_schema`]
    // to rewrite string-shaped scalars into the schema's expected
    // type before re-validating. Standalone comments and document
    // structure survive; inline comments on coerced scalar lines
    // are not preserved (the coercion path serialises the parsed
    // [`noyalib::Value`] tree, which omits inline comments).
    let mut total_fixes_via_coerce: usize = 0;
    let mut fix_handled_via_schema_path = false;
    if let Some(schema_path) = args.schema.as_deref() {
        let schema_text = match read_schema(schema_path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error: reading schema {}: {e}", schema_path.display());
                return ExitCode::from(3);
            }
        };
        let schema: noyalib::Value = match noyalib::from_str(&schema_text) {
            Ok(s) => s,
            Err(e) => {
                let report = Report::new(e).with_source_code(NamedSource::new(
                    schema_path.display().to_string(),
                    schema_text.clone(),
                ));
                eprintln!("error: parsing schema:");
                eprintln!("{report:?}");
                return ExitCode::from(1);
            }
        };

        if args.fix {
            // Transactional --fix on the **CST path**: coerce in
            // place via `noyalib::cst::coerce_to_schema` so
            // comments and indentation survive byte-faithfully.
            // Validate the coerced output before committing — if
            // anything still fails, the user's source is left
            // untouched and exit 1 surfaces the residue.
            let outcome = match run_fix_with_schema(path.as_deref(), &source, &schema) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("error: applying --fix: {e}");
                    let code = if e.kind() == io::ErrorKind::InvalidData {
                        1
                    } else {
                        3
                    };
                    return ExitCode::from(code);
                }
            };
            total_fixes_via_coerce = outcome.applied;
            fix_handled_via_schema_path = true;

            if !outcome.wrote {
                // Coercion couldn't bring the input fully into
                // schema-compliance — surface the remaining
                // violations and exit 1 without having modified
                // the file.
                let label = schema_path.display().to_string();
                let _ = run_schema_validation(&docs, &schema_text, &label, &name, &source);
                return ExitCode::from(1);
            }
        } else {
            let label = schema_path.display().to_string();
            let violations = run_schema_validation(&docs, &schema_text, &label, &name, &source);
            if violations > 0 {
                return ExitCode::from(1);
            }
        }
    }

    // Phase 3: `--fix` without `--schema` — pure-formatter path.
    if args.fix && !fix_handled_via_schema_path {
        if let Err(e) = run_fix(path.as_deref(), &source) {
            eprintln!("error: applying --fix: {e}");
            let code = if e.kind() == io::ErrorKind::InvalidData {
                1
            } else {
                3
            };
            return ExitCode::from(code);
        }
    }

    // Suppress the chatter when --fix is reading from stdin
    // — stdout is reserved for the formatted bytes and any
    // trailing message would corrupt downstream consumers.
    let stdin_fix = args.fix && path.is_none();
    if !args.quiet && !stdin_fix {
        let n = docs.len();
        let plural = if n == 1 { "document" } else { "documents" };
        let suffix = match (args.schema.is_some(), args.fix) {
            (true, true) => {
                if total_fixes_via_coerce == 0 {
                    " (schema-checked, no fixes needed)".to_string()
                } else {
                    format!(" (schema-checked, {total_fixes_via_coerce} fix(es) applied)")
                }
            }
            (true, false) => " (schema-checked)".to_string(),
            (false, true) => " (fixed)".to_string(),
            (false, false) => String::new(),
        };
        println!("ok: {n} {plural} valid ({name}){suffix}");
    }
    ExitCode::from(0)
}

fn main() -> ExitCode {
    run()
}
