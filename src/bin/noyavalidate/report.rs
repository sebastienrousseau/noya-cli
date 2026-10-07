// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! Rendering diagnostics over untrusted input.
//!
//! The source text is escaped once per input (control characters
//! become U+FFFD, see [`noya_cli::text`]) and shared by every report on
//! that input through an `Arc`, so a stream with many failing
//! documents does not copy the whole file once per report. Messages,
//! help and labels are escaped too, and label spans are moved onto the
//! escaped text.

use std::fmt;
use std::sync::Arc;

use miette::{Diagnostic, LabeledSpan, NamedSource, Report, SourceCode};
use noya_cli::text::{OffsetMap, escape_controls, escape_source};

/// One input's escaped source, shared by every report rendered on it.
pub struct SharedSource {
    named: Arc<NamedSource<String>>,
    map: OffsetMap,
}

impl SharedSource {
    /// Escape `text` (labelled `label`) for display.
    pub fn new(label: &str, text: &str) -> Self {
        let (escaped, map) = escape_source(text);
        Self {
            named: Arc::new(NamedSource::new(escape_controls(label), escaped)),
            map,
        }
    }
}

/// A diagnostic copied out of another one with every piece of text
/// escaped, pointing at a [`SharedSource`].
#[derive(Debug)]
struct Escaped {
    message: String,
    code: Option<String>,
    help: Option<String>,
    labels: Vec<LabeledSpan>,
    source: Option<Arc<NamedSource<String>>>,
}

fn escape_display(d: &dyn fmt::Display) -> String {
    escape_controls(&d.to_string()).into_owned()
}

fn remap(label: &LabeledSpan, map: &OffsetMap) -> LabeledSpan {
    let start = map.map(label.offset());
    let end = map.map(label.offset() + label.len());
    let text = label.label().map(|l| escape_controls(l).into_owned());
    if label.primary() {
        LabeledSpan::new_primary_with_span(text, (start, end - start))
    } else {
        LabeledSpan::new_with_span(text, (start, end - start))
    }
}

impl Escaped {
    fn new(diag: &dyn Diagnostic, source: Option<&SharedSource>) -> Self {
        let labels = match (diag.labels(), source) {
            (Some(labels), Some(src)) => labels.map(|l| remap(&l, &src.map)).collect(),
            _ => Vec::new(),
        };
        Self {
            message: escape_display(&diag),
            code: diag.code().map(|c| escape_display(&c)),
            help: diag.help().map(|h| escape_display(&h)),
            labels,
            source: source.map(|s| Arc::clone(&s.named)),
        }
    }
}

impl fmt::Display for Escaped {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Escaped {}

impl Diagnostic for Escaped {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.code
            .as_ref()
            .map(|c| Box::new(c) as Box<dyn fmt::Display>)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.help
            .as_ref()
            .map(|h| Box::new(h) as Box<dyn fmt::Display>)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        if self.labels.is_empty() {
            return None;
        }
        Some(Box::new(self.labels.iter().cloned()))
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        self.source.as_deref().map(|s| s as &dyn SourceCode)
    }
}

/// Render `diag` over `source` (if any) with every piece of untrusted
/// text escaped.
pub fn render(diag: &dyn Diagnostic, source: Option<&SharedSource>) -> String {
    format!("{:?}", Report::new(Escaped::new(diag, source)))
}
