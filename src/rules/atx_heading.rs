use std::collections::HashSet;

use crate::ast::NodeKind;
use crate::document::Document;
use crate::rule::Rule;
use crate::text::split_lines;
use crate::violation::{Span, Violation};

/// ATX heading lines are canonical: the `#` marker, a single space, then the
/// text, with no closing run of `#`s. `##  Heading ##` becomes `## Heading`.
/// The heading text is read from the source, so inline formatting is preserved.
pub struct AtxHeading;

const ID: &str = "atx-heading";

impl Rule for AtxHeading {
    fn id(&self) -> &'static str {
        ID
    }

    fn detect(&self, doc: &Document) -> Vec<Violation> {
        convert(doc).0
    }

    fn fix(&self, doc: &Document) -> Option<String> {
        Some(convert(doc).1)
    }
}

struct Canonical {
    text: String,
    changed: bool,
    had_closing: bool,
}

/// Detect and fix in one pass so the two halves can never disagree.
fn convert(doc: &Document) -> (Vec<Violation>, String) {
    let lines = split_lines(&doc.source);

    let mut heading_lines = HashSet::new();
    doc.tree().walk(&mut |node| {
        if let NodeKind::Heading { setext: false, .. } = node.kind {
            heading_lines.insert(node.span.start);
        }
    });

    let mut violations = Vec::new();
    let mut out = String::with_capacity(doc.source.len());
    for (index, line) in lines.iter().enumerate() {
        let line_number = index + 1;
        if heading_lines.contains(&line_number)
            && let Some(canonical) = canonicalize(line.content)
        {
            if canonical.changed {
                let message = if canonical.had_closing {
                    "remove the closing sequence of #s from the heading"
                } else {
                    "use a single space after the heading marker"
                };
                violations.push(Violation {
                    rule_id: ID,
                    message: message.to_string(),
                    span: Span {
                        line: line_number,
                        column: 1,
                        length: line.content.chars().count(),
                    },
                });
            }
            out.push_str(&canonical.text);
            out.push_str(line.terminator);
            continue;
        }

        out.push_str(line.content);
        out.push_str(line.terminator);
    }

    (violations, out)
}

fn canonicalize(line: &str) -> Option<Canonical> {
    let indent_len = line.len() - line.trim_start_matches(' ').len();
    let indent = &line[..indent_len];
    let after_indent = &line[indent_len..];

    let hashes = after_indent.chars().take_while(|c| *c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }

    let after_hashes = after_indent[hashes..].trim();
    let (content, had_closing) = strip_closing_hashes(after_hashes);

    let marker = "#".repeat(hashes);
    let text = if content.is_empty() {
        format!("{indent}{marker}")
    } else {
        format!("{indent}{marker} {content}")
    };

    Some(Canonical {
        changed: text != line,
        had_closing,
        text,
    })
}

/// Remove a trailing closing sequence of `#`s. Per CommonMark it only counts as
/// a closing sequence when preceded by whitespace (or the heading is empty), so
/// `foo#` keeps its hash.
fn strip_closing_hashes(text: &str) -> (&str, bool) {
    let without = text.trim_end_matches('#');
    if without.len() == text.len() {
        return (text, false);
    }
    if without.is_empty() {
        return ("", true);
    }
    if without.ends_with([' ', '\t']) {
        return (without.trim_end(), true);
    }
    (text, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(source: &str) -> String {
        AtxHeading.fix(&Document::new(source)).unwrap()
    }

    fn detect(source: &str) -> Vec<Violation> {
        AtxHeading.detect(&Document::new(source))
    }

    #[test]
    fn collapses_multiple_spaces_after_the_marker() {
        assert_eq!(fix("#  Heading\n"), "# Heading\n");
        let violations = detect("#  Heading\n");
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].message,
            "use a single space after the heading marker"
        );
    }

    #[test]
    fn removes_a_closing_hash_sequence() {
        assert_eq!(fix("## Heading ##\n"), "## Heading\n");
        let violations = detect("## Heading ##\n");
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].message,
            "remove the closing sequence of #s from the heading"
        );
    }

    #[test]
    fn leaves_a_canonical_heading_untouched() {
        assert_eq!(fix("# Heading\n"), "# Heading\n");
        assert!(detect("# Heading\n").is_empty());
    }

    #[test]
    fn keeps_a_trailing_hash_that_is_part_of_the_text() {
        assert_eq!(fix("# C#\n"), "# C#\n");
        assert!(detect("# C#\n").is_empty());
    }

    #[test]
    fn empties_a_closed_heading_with_no_text() {
        assert_eq!(fix("## ##\n"), "##\n");
    }

    #[test]
    fn preserves_inline_formatting_in_the_text() {
        assert_eq!(fix("# *foo* #\n"), "# *foo*\n");
    }

    #[test]
    fn ignores_hashes_inside_a_code_block() {
        let source = "```\n#  x\n```\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn normalises_multiple_headings() {
        assert_eq!(fix("#  A\n\n##  B ##\n"), "# A\n\n## B\n");
    }

    #[test]
    fn preserves_crlf_line_endings() {
        assert_eq!(fix("#  H\r\n"), "# H\r\n");
    }
}
