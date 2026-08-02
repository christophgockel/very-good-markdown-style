use std::collections::HashMap;

use crate::ast::{Node, NodeKind};
use crate::document::Document;
use crate::rule::Rule;
use crate::text::{Line, split_lines};
use crate::violation::{Span, Violation};

/// Nested list items are indented to line up with their parent item's content:
/// the parent's marker width plus one space. A list nested four spaces under a
/// `- ` parent is pulled back to two. Lists inside blockquotes are left alone.
pub struct NestedIndent;

const ID: &str = "nested-indent";

impl Rule for NestedIndent {
    fn id(&self) -> &'static str {
        ID
    }

    fn short_reason(&self) -> &'static str {
        "Indent nested list items to their parent's content."
    }

    fn rationale(&self) -> &'static str {
        "Aligning a nested list under the first character of its parent's text \
         keeps the outline readable and matches how the list renders. The indent \
         is the parent's marker width plus one space: two under a bullet, three \
         under `1.`, and so on."
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

    // Line number -> columns to add (negative removes leading spaces).
    let mut deltas: HashMap<usize, isize> = HashMap::new();
    let mut violations = Vec::new();
    drive(doc.tree(), &lines, &mut deltas, &mut violations);

    let mut out = String::with_capacity(doc.source.len());
    for (index, line) in lines.iter().enumerate() {
        let shifted = match deltas.get(&(index + 1)) {
            Some(&delta) if delta != 0 && !line.content.trim().is_empty() => {
                reindent(line.content, delta)
            }
            _ => line.content.to_string(),
        };
        out.push_str(&shifted);
        out.push_str(line.terminator);
    }

    (violations, out)
}

/// Find top-level lists to lay out. Lists inside blockquotes are skipped.
fn drive(
    node: &Node,
    lines: &[Line<'_>],
    deltas: &mut HashMap<usize, isize>,
    violations: &mut Vec<Violation>,
) {
    match node.kind {
        NodeKind::BlockQuote => {}
        NodeKind::List { .. } => layout(node, 0, lines, deltas, violations),
        _ => {
            for child in &node.children {
                drive(child, lines, deltas, violations);
            }
        }
    }
}

/// Lay out one list whose items should sit at `desired` indentation, recursing
/// into nested lists with the corrected parent content column.
fn layout(
    list: &Node,
    desired: usize,
    lines: &[Line<'_>],
    deltas: &mut HashMap<usize, isize>,
    violations: &mut Vec<Violation>,
) {
    for item in list.children.iter().filter(|c| c.kind == NodeKind::Item) {
        let marker_line = &lines[item.span.start - 1].content;
        let actual = leading_spaces(marker_line);
        let width = marker_width(marker_line, actual);
        let delta = desired as isize - actual as isize;
        let content_desired = desired + width + 1;

        let nested: Vec<(usize, usize)> = item
            .children
            .iter()
            .filter(|c| matches!(c.kind, NodeKind::List { .. }))
            .map(|list| (list.span.start, list.span.end))
            .collect();

        for line in item.span.start..=item.span.end {
            if !nested
                .iter()
                .any(|(start, end)| (*start..=*end).contains(&line))
            {
                *deltas.entry(line).or_default() += delta;
            }
        }

        if delta != 0 {
            violations.push(Violation {
                rule_id: ID,
                message: "align this list item with its parent's content".to_string(),
                span: Span {
                    line: item.span.start,
                    column: 1,
                    length: 1,
                },
            });
        }

        for child in item
            .children
            .iter()
            .filter(|c| matches!(c.kind, NodeKind::List { .. }))
        {
            layout(child, content_desired, lines, deltas, violations);
        }
    }
}

fn leading_spaces(content: &str) -> usize {
    content.chars().take_while(|c| *c == ' ').count()
}

fn marker_width(content: &str, indent: usize) -> usize {
    let chars: Vec<char> = content.chars().collect();
    let mut i = indent;
    if chars.get(i).is_some_and(char::is_ascii_digit) {
        while chars.get(i).is_some_and(char::is_ascii_digit) {
            i += 1;
        }
        i += 1; // the '.' or ')' delimiter
        i - indent
    } else {
        1
    }
}

fn reindent(content: &str, delta: isize) -> String {
    if delta > 0 {
        let mut out = " ".repeat(delta as usize);
        out.push_str(content);
        out
    } else {
        let remove = leading_spaces(content).min((-delta) as usize);
        content[remove..].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(source: &str) -> String {
        NestedIndent.fix(&Document::new(source)).unwrap()
    }

    fn detect(source: &str) -> Vec<Violation> {
        NestedIndent.detect(&Document::new(source))
    }

    #[test]
    fn pulls_an_over_indented_nested_list_back() {
        assert_eq!(fix("- a\n    - b\n"), "- a\n  - b\n");
        assert_eq!(detect("- a\n    - b\n").len(), 1);
    }

    #[test]
    fn leaves_correct_nesting_untouched() {
        let source = "- a\n  - b\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn aligns_under_an_ordered_parent() {
        assert_eq!(fix("1. a\n    - b\n"), "1. a\n   - b\n");
    }

    #[test]
    fn handles_three_levels() {
        assert_eq!(fix("- a\n  - b\n      - c\n"), "- a\n  - b\n    - c\n");
    }

    #[test]
    fn shifts_continuation_lines_with_the_item() {
        // The nested item's continuation line moves with its marker.
        assert_eq!(fix("- a\n    - b\n      cont\n"), "- a\n  - b\n    cont\n");
    }

    #[test]
    fn preserves_crlf_line_endings() {
        assert_eq!(fix("- a\r\n    - b\r\n"), "- a\r\n  - b\r\n");
    }
}
