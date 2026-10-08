// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! Making untrusted text safe to print.
//!
//! File contents and file names come from whoever wrote the files. A
//! control character in them (an ANSI colour or cursor sequence, an
//! OSC window-title sequence, a bell) would be interpreted by the
//! terminal or the CI log viewer if printed raw, so it is shown as
//! U+FFFD instead. Newlines and tabs are kept, and so is a carriage
//! return that is part of a CRLF line ending.

use std::borrow::Cow;
use std::path::Path;

/// The replacement shown in place of a control character.
pub const REPLACEMENT: char = '\u{fffd}';

/// Whether `c`, followed by `next`, must be replaced before printing.
fn must_replace(c: char, next: Option<char>) -> bool {
    match c {
        '\n' | '\t' => false,
        '\r' => next != Some('\n'),
        _ => c.is_control(),
    }
}

/// Replace every control character in `s` (other than newline, tab
/// and the carriage return of a CRLF) with U+FFFD. Borrows when there
/// is nothing to replace.
#[must_use]
pub fn escape_controls(s: &str) -> Cow<'_, str> {
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if must_replace(c, chars.peek().copied()) {
            return Cow::Owned(escape_source(s).0);
        }
    }
    Cow::Borrowed(s)
}

/// A path as it is safe to print: lossy UTF-8 with control characters
/// replaced.
#[must_use]
pub fn escape_path(path: &Path) -> String {
    escape_controls(&path.display().to_string()).into_owned()
}

/// Translates byte offsets in an original text into offsets in its
/// escaped form, so diagnostic spans keep pointing at the same text
/// after replacements changed the byte lengths.
#[derive(Debug, Clone, Default)]
pub struct OffsetMap {
    /// `(original offset of a replaced character, bytes added by all
    /// replacements up to and including it)`, in offset order.
    shifts: Vec<(usize, usize)>,
}

impl OffsetMap {
    /// The escaped-text offset for `offset` in the original text.
    #[must_use]
    pub fn map(&self, offset: usize) -> usize {
        let i = self.shifts.partition_point(|&(at, _)| at < offset);
        match i {
            0 => offset,
            _ => offset + self.shifts[i - 1].1,
        }
    }
}

/// Escape `s` as [`escape_controls`] does and return the offset map
/// from the original text to the escaped one.
#[must_use]
pub fn escape_source(s: &str) -> (String, OffsetMap) {
    let mut out = String::with_capacity(s.len());
    let mut map = OffsetMap::default();
    let mut added = 0usize;
    let mut chars = s.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        let next = chars.peek().map(|&(_, n)| n);
        if must_replace(c, next) {
            out.push(REPLACEMENT);
            added += REPLACEMENT.len_utf8() - c.len_utf8();
            map.shifts.push((at, added));
        } else {
            out.push(c);
        }
    }
    (out, map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_is_borrowed() {
        assert!(matches!(escape_controls("a: 1\n\tb\r\n"), Cow::Borrowed(_)));
    }

    #[test]
    fn escapes_ansi_osc_bell_and_c1() {
        let s = "\u{1b}[31mred\u{7}\u{9b}x\u{7f}";
        assert_eq!(
            escape_controls(s),
            "\u{fffd}[31mred\u{fffd}\u{fffd}x\u{fffd}"
        );
    }

    #[test]
    fn keeps_crlf_but_not_a_lone_carriage_return() {
        assert_eq!(escape_controls("a\r\nb\rc"), "a\r\nb\u{fffd}c");
    }

    #[test]
    fn offsets_after_a_replacement_shift_by_the_added_bytes() {
        let src = "x\u{1b}y\u{9b}z";
        let (out, map) = escape_source(src);
        // Each original character must map onto itself in the output.
        for (at, c) in src.char_indices() {
            let mapped = map.map(at);
            let got = out[mapped..].chars().next().unwrap();
            let want = if c.is_control() { REPLACEMENT } else { c };
            assert_eq!(got, want, "offset {at}");
        }
        assert_eq!(map.map(src.len()), out.len());
    }

    #[test]
    fn escape_path_replaces_controls_in_names() {
        assert_eq!(escape_path(Path::new("a\u{1b}b.yaml")), "a\u{fffd}b.yaml");
    }
}
