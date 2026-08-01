use std::collections::HashMap;

use crate::ast::NodeKind;
use crate::document::Document;
use crate::rule::Rule;
use crate::text::split_lines;
use crate::violation::{Span, Violation};

/// Unordered lists use `-` as their marker, never `*` or `+`. Because the marker
/// comes from the parsed list, a `*` used for emphasis or a thematic break is
/// never touched.
pub struct ListMarker;

const ID: &str = "list-marker";

impl Rule for ListMarker {
    fn id(&self) -> &'static str {
        ID
    }

    fn short_reason(&self) -> &'static str {
        "Use `-` for unordered list markers."
    }

    fn rationale(&self) -> &'static str {
        "One unordered list marker throughout keeps lists visually consistent. \
         We use `-`: it is the most common choice and never reads as emphasis the \
         way a leading `*` can."
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

    // Marker positions of unordered list items: line -> columns (1-based).
    let mut markers: HashMap<usize, Vec<usize>> = HashMap::new();
    doc.tree().walk(&mut |node| {
        if let NodeKind::List { ordered: false } = node.kind {
            for item in &node.children {
                if item.kind == NodeKind::Item {
                    markers
                        .entry(item.span.start)
                        .or_default()
                        .push(item.start_column);
                }
            }
        }
    });

    let mut violations = Vec::new();
    let mut out = String::with_capacity(doc.source.len());
    for (index, line) in lines.iter().enumerate() {
        let line_number = index + 1;
        match markers.get(&line_number) {
            Some(columns) => {
                let (rewritten, mut line_violations) =
                    replace_markers(line.content, line_number, columns);
                violations.append(&mut line_violations);
                out.push_str(&rewritten);
            }
            None => out.push_str(line.content),
        }
        out.push_str(line.terminator);
    }

    (violations, out)
}

fn replace_markers(content: &str, line: usize, columns: &[usize]) -> (String, Vec<Violation>) {
    let mut chars: Vec<char> = content.chars().collect();
    let mut violations = Vec::new();
    for &column in columns {
        if let Some(marker) = chars.get_mut(column - 1)
            && matches!(*marker, '*' | '+')
        {
            *marker = '-';
            violations.push(Violation {
                rule_id: ID,
                message: "use `-` for the list marker".to_string(),
                span: Span {
                    line,
                    column,
                    length: 1,
                },
            });
        }
    }
    (chars.into_iter().collect(), violations)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(source: &str) -> String {
        ListMarker.fix(&Document::new(source)).unwrap()
    }

    fn detect(source: &str) -> Vec<Violation> {
        ListMarker.detect(&Document::new(source))
    }

    #[test]
    fn converts_a_star_marker() {
        assert_eq!(fix("* foo\n"), "- foo\n");
        assert_eq!(detect("* foo\n").len(), 1);
    }

    #[test]
    fn converts_a_plus_marker() {
        assert_eq!(fix("+ foo\n"), "- foo\n");
    }

    #[test]
    fn leaves_a_dash_marker_untouched() {
        assert_eq!(fix("- foo\n"), "- foo\n");
        assert!(detect("- foo\n").is_empty());
    }

    #[test]
    fn converts_a_nested_marker() {
        assert_eq!(fix("- a\n  * b\n"), "- a\n  - b\n");
    }

    #[test]
    fn leaves_ordered_lists_untouched() {
        assert_eq!(fix("1. foo\n"), "1. foo\n");
        assert!(detect("1. foo\n").is_empty());
    }

    #[test]
    fn does_not_touch_emphasis() {
        let source = "a *b* c\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn preserves_crlf_line_endings() {
        assert_eq!(fix("* foo\r\n"), "- foo\r\n");
    }
}
