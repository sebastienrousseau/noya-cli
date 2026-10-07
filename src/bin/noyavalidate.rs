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
use noyalib::{CompiledSchema, Value};

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

/// The schema as written (the coercion pass reads it) and compiled
/// once up front (every document is validated against the compiled
/// form).
struct Schema {
    value: Value,
    compiled: CompiledSchema,
}

/// Read, parse and compile the schema at `path`. A schema that does
/// not compile is an error before any input is looked at, so a broken
/// schema can never be "passed" by an input with nothing to check.
/// On failure the diagnostic is printed and the exit code returned.
fn load_schema(path: &Path) -> Result<Schema, u8> {
    let text = fs::read_to_string(path).map_err(|e| {
        eprintln!("error: reading schema {}: {e}", path.display());
        3
    })?;
    let value: Value = noyalib::from_str(&text).map_err(|e| {
        let report = Report::new(e)
            .with_source_code(NamedSource::new(path.display().to_string(), text.clone()));
        eprintln!("error: parsing schema:");
        eprintln!("{report:?}");
        1
    })?;
    let compiled = CompiledSchema::compile(&value).map_err(|e| {
        eprintln!("error: compiling schema {}: {e}", path.display());
        1
    })?;
    Ok(Schema { value, compiled })
}

/// Run schema validation across every parsed document. Returns the
/// number of violations found (0 = success). Emits one miette report
/// per failing document so the user sees all issues in one pass.
fn run_schema_validation(
    docs: &[Value],
    schema: &CompiledSchema,
    source_label: &str,
    full_source: &str,
) -> usize {
    let mut violations = 0;
    for (i, doc) in docs.iter().enumerate() {
        if let Err(e) = schema.validate(doc) {
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
    write_output(path, &formatted)
}

/// Write `text` to `path`, or to stdout when `path` is `None`.
fn write_output(path: Option<&Path>, text: &str) -> io::Result<()> {
    match path {
        None => io::stdout().lock().write_all(text.as_bytes()),
        Some(p) => noya_cli::write_atomic(p, text.as_bytes()),
    }
}

/// Print a `--fix` failure and map it to the exit code: 1 when the
/// input was refused, 3 for an I/O error.
fn fix_failure(e: &io::Error) -> u8 {
    eprintln!("error: applying --fix: {e}");
    if e.kind() == io::ErrorKind::InvalidData {
        1
    } else {
        3
    }
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
/// 1. Parse the source via [`noyalib::cst::parse_stream`] (multi-doc-aware).
/// 2. For each document, run [`noyalib::cst::coerce_to_schema`] in-place — only
///    string scalars whose schema-declared type is integer / number / boolean
///    are coerced; everything else is preserved.
/// 3. Re-validate each coerced document against the compiled schema. If any
///    violation remains, return without writing — the caller surfaces the
///    residue and exits 1 with the user's original source intact.
/// 4. If validation passes, write the concatenated CST sources back to `path`
///    (or stdout). Comments and formatting survive byte-faithfully.
fn run_fix_with_schema(
    path: Option<&Path>,
    source: &str,
    schema: &Schema,
) -> io::Result<FixOutcome> {
    let invalid = |msg: String| io::Error::new(io::ErrorKind::InvalidData, msg);
    // Parse the source as a CST stream. This is what unlocks the
    // comment-preserving path: every byte that isn't part of a
    // coerced scalar will round-trip verbatim.
    let mut docs = noyalib::cst::parse_stream(source)
        .map_err(|e| invalid(format!("--fix: parse stream: {e}")))?;

    let mut applied = 0usize;
    for cst_doc in docs.iter_mut() {
        applied += noyalib::cst::coerce_to_schema(cst_doc, &schema.value)
            .map_err(|e| invalid(format!("--fix: cst::coerce_to_schema failed: {e}")))?;
    }

    // Transactional gate: validate each coerced document. We have
    // to re-parse each CST back to a Value because validation
    // operates on the `noyalib::Value` shape; this also surfaces
    // residue (e.g. `port: "abc"` against `type: integer` — not
    // coercible by parse).
    let still_invalid =
        docs.iter().any(
            |cst_doc| match noyalib::from_str::<Value>(&cst_doc.to_string()) {
                Ok(v) => schema.compiled.validate(&v).is_err(),
                Err(_) => true,
            },
        );
    if still_invalid {
        return Ok(FixOutcome {
            applied,
            wrote: false,
        });
    }

    // Concatenate CST sources for the final write — preserves
    // every untouched byte (including inter-document `---` /
    // `...` separators).
    let output: String = docs.iter().map(ToString::to_string).collect();

    // Always run the lossless formatter on top so the
    // `--fix --schema` and the `--fix`-only paths produce
    // equivalent whitespace shape. `cst::format` is itself
    // comment-preserving — only whitespace and quoting are
    // normalised. Falls back to the raw concatenation if the
    // formatter rejects the post-coerce text (defensive — would
    // signal a parser bug).
    let final_output = noyalib::cst::format(&output).unwrap_or(output);
    write_output(path, &final_output)?;
    Ok(FixOutcome {
        applied,
        wrote: true,
    })
}

/// Parse every document of `source`. With `strict`, the YAML 1.2 strict
/// profile applies: duplicate keys are an error, only `true`/`false`
/// are booleans, indentation must be even, and the tighter resource
/// limits for untrusted input are in force.
fn load_documents(source: &str, strict: bool) -> Result<Vec<Value>, noyalib::Error> {
    if strict {
        noyalib::load_all_with_config(source, &noyalib::ParserConfig::strict())?.collect()
    } else {
        noyalib::load_all_as::<Value>(source)
    }
}

/// What the run was asked to do, shared by every input.
struct Options<'a> {
    schema: Option<&'a Schema>,
    fix: bool,
    quiet: bool,
    strict: bool,
}

/// One input as read and parsed.
struct Input<'a> {
    path: Option<&'a Path>,
    name: String,
    source: String,
}

/// Phase 2 with a schema. Four flag combinations are possible:
///
/// | `--schema` | `--fix` | Behaviour                                         |
/// | :---:      | :---:   | :---                                              |
/// | no         | no      | syntax check only (Phase 1).                      |
/// | no         | yes     | run lossless formatter (Phase 3).                 |
/// | yes        | no      | strict validate; exit 1 on violation.             |
/// | yes        | yes     | coerce, re-validate, format, write. Exits 1 only  |
/// |            |         | if violations remain *after* coercion.            |
///
/// A stream with no documents (an empty or comment-only file) is
/// validated as one null document, the value YAML gives an empty
/// document, so it passes only a schema that accepts null.
///
/// Returns the success-message suffix, or the exit code on failure.
fn schema_phase(
    input: &Input<'_>,
    mut docs: Vec<Value>,
    schema: &Schema,
    fix: bool,
) -> Result<String, u8> {
    let empty = docs.is_empty();
    if empty {
        docs.push(Value::Null);
    }
    if !fix || empty {
        if run_schema_validation(&docs, &schema.compiled, &input.name, &input.source) > 0 {
            return Err(1);
        }
        if fix {
            run_fix(input.path, &input.source).map_err(|e| fix_failure(&e))?;
            return Ok(" (schema-checked, no fixes needed)".to_string());
        }
        return Ok(" (schema-checked)".to_string());
    }
    // Transactional --fix on the **CST path**: coerce in place so
    // comments and indentation survive byte-faithfully. If anything
    // still fails, the source is left untouched and exit 1 surfaces
    // the residue.
    let outcome =
        run_fix_with_schema(input.path, &input.source, schema).map_err(|e| fix_failure(&e))?;
    if !outcome.wrote {
        let _ = run_schema_validation(&docs, &schema.compiled, &input.name, &input.source);
        return Err(1);
    }
    Ok(match outcome.applied {
        0 => " (schema-checked, no fixes needed)".to_string(),
        n => format!(" (schema-checked, {n} fix(es) applied)"),
    })
}

/// Check one input end to end and return its exit code.
fn check_input(path: Option<&Path>, opts: &Options<'_>) -> u8 {
    let (name, source) = match read_input(path) {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("error: reading input: {e}");
            return 3;
        }
    };
    let input = Input { path, name, source };

    // Phase 1: syntax check. The CST-aware --fix path takes the
    // source string directly so it can preserve comments — these
    // parsed `Value`s are used only for validation reporting.
    let docs = match load_documents(&input.source, opts.strict) {
        Ok(d) => d,
        Err(e) => {
            let report = Report::new(e)
                .with_source_code(NamedSource::new(&input.name, input.source.clone()));
            eprintln!("{report:?}");
            return 1;
        }
    };
    let count = docs.len();

    // Phase 2 (schema, optionally with --fix) or Phase 3 (--fix
    // without a schema: pure-formatter path).
    let suffix = match opts.schema {
        Some(schema) => schema_phase(&input, docs, schema, opts.fix),
        None if opts.fix => run_fix(path, &input.source)
            .map(|()| " (fixed)".to_string())
            .map_err(|e| fix_failure(&e)),
        None => Ok(String::new()),
    };
    match suffix {
        Ok(suffix) => {
            report_ok(&input, count, &suffix, opts);
            0
        }
        Err(code) => code,
    }
}

/// Print the success line, unless `--quiet`, or unless `--fix` is
/// reading from stdin: stdout is then reserved for the formatted
/// bytes and any trailing message would corrupt downstream consumers.
fn report_ok(input: &Input<'_>, count: usize, suffix: &str, opts: &Options<'_>) {
    let stdin_fix = opts.fix && input.path.is_none();
    if opts.quiet || stdin_fix {
        return;
    }
    let plural = if count == 1 { "document" } else { "documents" };
    println!("ok: {count} {plural} valid ({}){suffix}", input.name);
}

/// Turn the FILE arguments into the inputs to check: `None` is stdin.
/// No argument, or a lone `-`, reads stdin; `-` next to other files is
/// a usage error (exit 2), since stdin can only be read once.
fn resolve_inputs(files: &[PathBuf]) -> Vec<Option<&Path>> {
    let is_dash = |p: &PathBuf| p.as_os_str() == "-";
    if files.is_empty() || (files.len() == 1 && is_dash(&files[0])) {
        return vec![None];
    }
    if files.iter().any(is_dash) {
        noya_cli::noyavalidate_command()
            .error(
                clap::error::ErrorKind::ArgumentConflict,
                "'-' (stdin) cannot be combined with other FILE arguments",
            )
            .exit();
    }
    files.iter().map(|p| Some(p.as_path())).collect()
}

fn run() -> ExitCode {
    let args = NoyavalidateCli::parse();
    let inputs = resolve_inputs(&args.files);

    let schema = match args.schema.as_deref().map(load_schema).transpose() {
        Ok(s) => s,
        Err(code) => return ExitCode::from(code),
    };
    let opts = Options {
        schema: schema.as_ref(),
        fix: args.fix,
        quiet: args.quiet,
        strict: args.strict,
    };
    // Every input is checked even after a failure, so one run reports
    // every bad file; the exit code is the most severe one seen
    // (3 for I/O over 1 for a parse or schema failure).
    let worst = inputs
        .into_iter()
        .map(|path| check_input(path, &opts))
        .max()
        .unwrap_or(0);
    ExitCode::from(worst)
}

fn main() -> ExitCode {
    run()
}
