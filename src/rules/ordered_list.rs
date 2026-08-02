use std::collections::HashMap;

use crate::ast::{Node, NodeKind};
use crate::document::Document;
use crate::rule::Rule;
use crate::text::split_lines;
use crate::violation::{Span, Violation};

/// Ordered list items are numbered sequentially: `1. 2. 3.`. The list's starting
/// number is kept, so a list that begins at another number keeps counting from
/// there. The delimiter (`.` or `)`) is preserved.
pub struct OrderedList;

const ID: &str = "ordered-list";

impl Rule for OrderedList {
    fn id(&self) -> &'static str {
        ID
    }

    fn short_reason(&self) -> &'static str {
        "Number ordered list items in sequence."
    }

    fn rationale(&self) -> &'static str {
        "Sequential numbers in the source match what the reader sees rendered, so \
         the Markdown is easy to follow and reorder. The list keeps whatever number \
         it starts from and counts up from there."
    }

    fn detect(&self, doc: &Document) -> Vec<Violation> {
        rewrite(doc).0
    }

    fn fix(&self, doc: &Document) -> Option<String> {
        Some(rewrite(doc).1)
    }
}

struct Renumber {
    digit_start: usize,
    old_len: usize,
    number: String,
}

/// Detect and fix in one pass so the two halves can never disagree.
fn rewrite(doc: &Document) -> (Vec<Violation>, String) {
    let lines = split_lines(&doc.source);

    let mut edits: HashMap<usize, Renumber> = HashMap::new();
    collect(doc.tree(), &lines, &mut edits);

    let mut violations = Vec::new();
    let mut out = String::with_capacity(doc.source.len());
    for (index, line) in lines.iter().enumerate() {
        match edits.get(&(index + 1)) {
            Some(edit) => {
                violations.push(Violation {
                    rule_id: ID,
                    message: "number ordered list items in sequence".to_string(),
                    span: Span {
                        line: index + 1,
                        column: edit.digit_start + 1,
                        length: edit.old_len,
                    },
                });
                let chars: Vec<char> = line.content.chars().collect();
                out.extend(&chars[..edit.digit_start]);
                out.push_str(&edit.number);
                out.extend(&chars[edit.digit_start + edit.old_len..]);
            }
            None => out.push_str(line.content),
        }
        out.push_str(line.terminator);
    }

    (violations, out)
}

fn collect(node: &Node, lines: &[crate::text::Line<'_>], edits: &mut HashMap<usize, Renumber>) {
    if let NodeKind::List { ordered: true } = node.kind {
        let items: Vec<&Node> = node
            .children
            .iter()
            .filter(|child| child.kind == NodeKind::Item)
            .collect();

        if let Some(start) = items
            .first()
            .and_then(|item| number_at(lines, item).map(|(value, _)| value))
        {
            for (offset, item) in items.iter().enumerate() {
                if let Some((value, len)) = number_at(lines, item) {
                    let target = start + offset;
                    if value != target {
                        edits.insert(
                            item.span.start,
                            Renumber {
                                digit_start: item.start_column - 1,
                                old_len: len,
                                number: target.to_string(),
                            },
                        );
                    }
                }
            }
        }
    }

    for child in &node.children {
        collect(child, lines, edits);
    }
}

/// Parse the item's marker number from its first line: its value and digit count.
fn number_at(lines: &[crate::text::Line<'_>], item: &Node) -> Option<(usize, usize)> {
    let chars: Vec<char> = lines.get(item.span.start - 1)?.content.chars().collect();
    let start = item.start_column - 1;
    let digits: String = chars[start..]
        .iter()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        return None;
    }
    Some((digits.parse().ok()?, digits.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(source: &str) -> String {
        OrderedList.fix(&Document::new(source)).unwrap()
    }

    fn detect(source: &str) -> Vec<Violation> {
        OrderedList.detect(&Document::new(source))
    }

    #[test]
    fn renumbers_a_lazy_list() {
        assert_eq!(fix("1. a\n1. b\n1. c\n"), "1. a\n2. b\n3. c\n");
        assert_eq!(detect("1. a\n1. b\n1. c\n").len(), 2);
    }

    #[test]
    fn leaves_a_correctly_numbered_list() {
        let source = "1. a\n2. b\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn keeps_the_starting_number() {
        assert_eq!(fix("3. a\n3. b\n"), "3. a\n4. b\n");
    }

    #[test]
    fn preserves_the_delimiter() {
        assert_eq!(fix("1) a\n1) b\n"), "1) a\n2) b\n");
    }

    #[test]
    fn numbers_nested_lists_independently() {
        assert_eq!(
            fix("1. a\n   1. x\n   1. y\n1. b\n"),
            "1. a\n   1. x\n   2. y\n2. b\n"
        );
    }

    #[test]
    fn leaves_unordered_lists_untouched() {
        let source = "- a\n- b\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn preserves_crlf_line_endings() {
        assert_eq!(fix("1. a\r\n1. b\r\n"), "1. a\r\n2. b\r\n");
    }
}
