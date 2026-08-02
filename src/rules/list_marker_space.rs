use std::collections::HashMap;

use crate::ast::NodeKind;
use crate::document::Document;
use crate::rule::Rule;
use crate::text::split_lines;
use crate::violation::{Span, Violation};

/// Exactly one space follows a list marker. `-   text` and `1.   text` become
/// `- text` and `1. text`. Over-wide gaps of five or more spaces are left alone,
/// since those mark indented content inside the item.
pub struct ListMarkerSpace;

const ID: &str = "list-marker-space";

impl Rule for ListMarkerSpace {
    fn id(&self) -> &'static str {
        ID
    }

    fn short_reason(&self) -> &'static str {
        "Use one space after a list marker."
    }

    fn rationale(&self) -> &'static str {
        "A single space after the marker keeps list items aligned predictably and \
         the source tidy. Wider gaps vary between authors and add nothing, so they \
         are collapsed to one space."
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

    // Item first line -> content column (1-based). The content column lives on
    // the item's first child, not the item itself (whose column is the marker).
    let mut items: HashMap<usize, usize> = HashMap::new();
    doc.tree().walk(&mut |node| {
        if node.kind == NodeKind::Item
            && let Some(child) = node.children.first()
        {
            items.insert(node.span.start, child.start_column);
        }
    });

    let mut violations = Vec::new();
    let mut out = String::with_capacity(doc.source.len());
    for (index, line) in lines.iter().enumerate() {
        let line_number = index + 1;
        match items.get(&line_number) {
            Some(&content_column) => match normalise(line.content, content_column) {
                Some((rewritten, column)) => {
                    violations.push(Violation {
                        rule_id: ID,
                        message: "use a single space after the list marker".to_string(),
                        span: Span {
                            line: line_number,
                            column,
                            length: 1,
                        },
                    });
                    out.push_str(&rewritten);
                }
                None => out.push_str(line.content),
            },
            None => out.push_str(line.content),
        }
        out.push_str(line.terminator);
    }

    (violations, out)
}

/// Return the normalised line and the marker-end column, or `None` if there is
/// nothing to change.
fn normalise(content: &str, content_column: usize) -> Option<(String, usize)> {
    let chars: Vec<char> = content.chars().collect();
    let indent = chars.iter().take_while(|c| **c == ' ').count();
    let mut end = indent;

    if chars.get(end)?.is_ascii_digit() {
        while chars.get(end).is_some_and(char::is_ascii_digit) {
            end += 1;
        }
        if !matches!(chars.get(end), Some('.') | Some(')')) {
            return None;
        }
        end += 1;
    } else if matches!(chars.get(end), Some('-' | '*' | '+')) {
        end += 1;
    } else {
        return None;
    }

    let content_start = content_column.checked_sub(1)?;
    let gap = content_start.checked_sub(end)?;
    // One space is already correct; five or more marks indented content.
    if !(2..=4).contains(&gap) || content_start >= chars.len() {
        return None;
    }

    let mut rewritten: String = chars[..end].iter().collect();
    rewritten.push(' ');
    rewritten.extend(&chars[content_start..]);
    Some((rewritten, end + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(source: &str) -> String {
        ListMarkerSpace.fix(&Document::new(source)).unwrap()
    }

    fn detect(source: &str) -> Vec<Violation> {
        ListMarkerSpace.detect(&Document::new(source))
    }

    #[test]
    fn collapses_spaces_after_a_bullet() {
        assert_eq!(fix("-   foo\n"), "- foo\n");
        assert_eq!(detect("-   foo\n").len(), 1);
    }

    #[test]
    fn collapses_spaces_after_an_ordered_marker() {
        assert_eq!(fix("1.   foo\n"), "1. foo\n");
    }

    #[test]
    fn leaves_a_single_space_untouched() {
        assert_eq!(fix("- foo\n"), "- foo\n");
        assert!(detect("- foo\n").is_empty());
        assert_eq!(fix("1. foo\n"), "1. foo\n");
    }

    #[test]
    fn collapses_a_nested_marker() {
        assert_eq!(fix("- a\n  -   b\n"), "- a\n  - b\n");
    }

    #[test]
    fn preserves_crlf_line_endings() {
        assert_eq!(fix("-   foo\r\n"), "- foo\r\n");
    }
}
