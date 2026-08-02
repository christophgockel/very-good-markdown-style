use crate::ast::{Node, NodeKind};
use crate::document::Document;
use crate::rule::{Rule, RuleKind};
use crate::text::split_lines;
use crate::violation::{Span, Violation};

/// Continuation lines of a paragraph inside a list item or blockquote must reach
/// the text they continue, by indentation for a list and by a `>` prefix for a
/// quote. A line that falls short is silently folded into the block above it by
/// Markdown's lazy-continuation rule, so it reads as part of a different item
/// than it appears to. Detect-only, because the fix is the author's call.
pub struct ContinuationIndent;

const ID: &str = "continuation-indent";

impl Rule for ContinuationIndent {
    fn id(&self) -> &'static str {
        ID
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Flag
    }

    fn short_reason(&self) -> &'static str {
        "Indent continuation lines to reach the text they continue."
    }

    fn rationale(&self) -> &'static str {
        "When a paragraph continues onto another line inside a list item or \
         blockquote, that line has to reach the text it continues, by indentation \
         in a list or a `>` prefix in a quote. A line that falls short is silently \
         folded into the block above it by Markdown's lazy-continuation rule, so it \
         renders as part of an item it does not appear to belong to. Indent it to \
         line up, or add a blank line to make it a separate block. This is reported \
         but never fixed, because only you know which you meant."
    }

    fn detect(&self, doc: &Document) -> Vec<Violation> {
        let lines = split_lines(&doc.source);
        let mut violations = Vec::new();
        check(doc.tree(), false, &lines, &mut violations);
        violations
    }
}

fn check(
    node: &Node,
    in_container: bool,
    lines: &[crate::text::Line<'_>],
    out: &mut Vec<Violation>,
) {
    match node.kind {
        NodeKind::Paragraph if in_container => {
            for line_number in (node.span.start + 1)..=node.span.end {
                let Some(line) = lines.get(line_number - 1) else {
                    continue;
                };
                let column = content_column(line.content);
                if column < node.start_column {
                    out.push(Violation {
                        rule_id: ID,
                        message: "indent this line to reach the text it continues, or separate it with a blank line".to_string(),
                        span: Span {
                            line: line_number,
                            column,
                            length: 1,
                        },
                    });
                }
            }
        }
        NodeKind::Paragraph => {}
        NodeKind::Item | NodeKind::BlockQuote => {
            for child in &node.children {
                check(child, true, lines, out);
            }
        }
        _ => {
            for child in &node.children {
                check(child, in_container, lines, out);
            }
        }
    }
}

/// The column where a line's text begins, past leading whitespace and blockquote
/// markers.
fn content_column(line: &str) -> usize {
    1 + line
        .chars()
        .take_while(|c| matches!(c, ' ' | '\t' | '>'))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect(source: &str) -> Vec<Violation> {
        ContinuationIndent.detect(&Document::new(source))
    }

    #[test]
    fn flags_an_under_indented_list_continuation() {
        let violations = detect("- item one.\nbadly aligned.\n");
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].span.line, 2);
    }

    #[test]
    fn allows_a_properly_indented_list_continuation() {
        assert!(detect("- item one\n  continues here.\n").is_empty());
    }

    #[test]
    fn flags_a_line_folded_into_a_nested_item() {
        // The trailing line looks like it belongs to the outer item, but it is
        // folded into the nested bullet.
        let violations = detect("- outer.\n  - nested.\n  trailing line.\n");
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].span.line, 3);
    }

    #[test]
    fn flags_a_lazy_blockquote_continuation() {
        let violations = detect("> quote line.\nlazy line.\n");
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].span.line, 2);
    }

    #[test]
    fn allows_a_prefixed_blockquote_continuation() {
        assert!(detect("> quote line.\n> more here.\n").is_empty());
    }

    #[test]
    fn ignores_top_level_paragraphs() {
        assert!(detect("first line\nsecond line.\n").is_empty());
    }
}
