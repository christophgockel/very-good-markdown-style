use std::collections::HashSet;

use crate::ast::NodeKind;
use crate::document::Document;
use crate::rule::Rule;
use crate::text::split_lines;
use crate::violation::{Span, Violation};

/// Each blockquote `>` marker is followed by exactly one space. Only the markers
/// are touched; any further indentation is content (a nested list, say) and is
/// preserved. Empty quote lines are left to the trailing-whitespace rule.
pub struct BlockquoteMarker;

const ID: &str = "blockquote-marker";

impl Rule for BlockquoteMarker {
    fn id(&self) -> &'static str {
        ID
    }

    fn short_reason(&self) -> &'static str {
        "Put one space after each blockquote marker."
    }

    fn rationale(&self) -> &'static str {
        "A single space after `>` keeps quotes readable in the source and \
         consistent between authors. Only the marker spacing changes. Deeper \
         indentation is content and is left as written."
    }

    fn detect(&self, doc: &Document) -> Vec<Violation> {
        rewrite(doc).0
    }

    fn fix(&self, doc: &Document) -> Option<String> {
        Some(rewrite(doc).1)
    }
}

/// Detect and fix in one pass so the two halves can never disagree.
fn rewrite(doc: &Document) -> (Vec<Violation>, String) {
    let lines = split_lines(&doc.source);
    let quote_lines = quote_line_numbers(doc);

    let mut violations = Vec::new();
    let mut out = String::with_capacity(doc.source.len());
    for (index, line) in lines.iter().enumerate() {
        let line_number = index + 1;
        match normalise(line.content) {
            Some(rewritten) if quote_lines.contains(&line_number) => {
                violations.push(Violation {
                    rule_id: ID,
                    message: "put one space after the blockquote marker".to_string(),
                    span: Span {
                        line: line_number,
                        column: 1,
                        length: 1,
                    },
                });
                out.push_str(&rewritten);
            }
            _ => out.push_str(line.content),
        }
        out.push_str(line.terminator);
    }

    (violations, out)
}

fn quote_line_numbers(doc: &Document) -> HashSet<usize> {
    let mut quote = HashSet::new();
    doc.tree().walk(&mut |node| {
        if node.kind == NodeKind::BlockQuote {
            quote.extend(node.span.start..=node.span.end);
        }
    });
    quote
}

/// Rewrite the leading `>` markers to one space each, or `None` if unchanged or
/// the line has no marker content to normalise.
fn normalise(content: &str) -> Option<String> {
    let chars: Vec<char> = content.chars().collect();
    if chars.first() != Some(&'>') {
        return None;
    }

    let mut i = 0;
    let mut depth = 0;
    while chars.get(i) == Some(&'>') {
        depth += 1;
        i += 1;
        if chars.get(i) == Some(&' ') {
            i += 1;
        }
    }

    let rest: String = chars[i..].iter().collect();
    if rest.trim().is_empty() {
        return None;
    }

    let canonical = format!("{}{rest}", "> ".repeat(depth));
    if canonical == content {
        None
    } else {
        Some(canonical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(source: &str) -> String {
        BlockquoteMarker.fix(&Document::new(source)).unwrap()
    }

    fn detect(source: &str) -> Vec<Violation> {
        BlockquoteMarker.detect(&Document::new(source))
    }

    #[test]
    fn adds_a_missing_marker_space() {
        assert_eq!(fix(">foo\n"), "> foo\n");
        assert_eq!(detect(">foo\n").len(), 1);
    }

    #[test]
    fn leaves_a_correct_marker_untouched() {
        assert_eq!(fix("> foo\n"), "> foo\n");
        assert!(detect("> foo\n").is_empty());
    }

    #[test]
    fn spaces_nested_markers() {
        assert_eq!(fix(">>foo\n"), "> > foo\n");
    }

    #[test]
    fn preserves_content_indentation() {
        // A nested list inside the quote keeps its indentation.
        let source = ">   - b\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn leaves_empty_quote_lines_to_the_whitespace_rule() {
        assert_eq!(fix(">\n"), ">\n");
        assert!(detect(">\n").is_empty());
    }

    #[test]
    fn does_not_touch_a_gt_inside_a_code_block() {
        let source = "```\n>foo\n```\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn preserves_crlf_line_endings() {
        assert_eq!(fix(">foo\r\n"), "> foo\r\n");
    }
}
