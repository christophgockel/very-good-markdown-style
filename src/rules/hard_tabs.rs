use std::collections::HashSet;

use crate::ast::NodeKind;
use crate::document::Document;
use crate::rule::Rule;
use crate::text::split_lines;
use crate::violation::{Span, Violation};

const TAB_WIDTH: usize = 4;

/// Hard tabs are expanded to spaces, to four-column tab stops, everywhere except
/// inside code blocks, where whitespace can be significant.
pub struct HardTabs;

const ID: &str = "hard-tabs";

impl Rule for HardTabs {
    fn id(&self) -> &'static str {
        ID
    }

    fn short_reason(&self) -> &'static str {
        "Use spaces, not hard tabs."
    }

    fn rationale(&self) -> &'static str {
        "Hard tabs render at different widths in different tools, so indentation \
         and alignment drift between editors. They are expanded to spaces at \
         four-column tab stops everywhere except inside code blocks, where a tab \
         may be part of the code."
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
    let code_lines = code_line_numbers(doc);

    let mut violations = Vec::new();
    let mut out = String::with_capacity(doc.source.len());
    for (index, line) in lines.iter().enumerate() {
        let line_number = index + 1;
        if !code_lines.contains(&line_number)
            && let Some(column) = line.content.chars().position(|c| c == '\t')
        {
            violations.push(Violation {
                rule_id: ID,
                message: "hard tab; use spaces".to_string(),
                span: Span {
                    line: line_number,
                    column: column + 1,
                    length: 1,
                },
            });
            out.push_str(&expand_tabs(line.content));
        } else {
            out.push_str(line.content);
        }
        out.push_str(line.terminator);
    }

    (violations, out)
}

fn code_line_numbers(doc: &Document) -> HashSet<usize> {
    let mut code = HashSet::new();
    doc.tree().walk(&mut |node| {
        if let NodeKind::CodeBlock { .. } = node.kind {
            code.extend(node.span.start..=node.span.end);
        }
    });
    code
}

fn expand_tabs(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut column = 0;
    for c in content.chars() {
        if c == '\t' {
            let spaces = TAB_WIDTH - (column % TAB_WIDTH);
            out.extend(std::iter::repeat_n(' ', spaces));
            column += spaces;
        } else {
            out.push(c);
            column += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(source: &str) -> String {
        HardTabs.fix(&Document::new(source)).unwrap()
    }

    fn detect(source: &str) -> Vec<Violation> {
        HardTabs.detect(&Document::new(source))
    }

    #[test]
    fn expands_a_tab_to_the_next_tab_stop() {
        assert_eq!(fix("a\tb\n"), "a   b\n");
        let violations = detect("a\tb\n");
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].span.column, 2);
    }

    #[test]
    fn expands_a_leading_tab_to_four_spaces() {
        // A lone tab-indented line is an indented code block, so use a tab after
        // text on a normal paragraph line instead.
        assert_eq!(fix("text\n\tmore\n"), "text\n    more\n");
    }

    #[test]
    fn preserves_tabs_inside_a_fenced_code_block() {
        let source = "```\n\tcode\n```\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn leaves_tab_free_text_untouched() {
        let source = "no tabs here\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn preserves_crlf_line_endings() {
        assert_eq!(fix("a\tb\r\n"), "a   b\r\n");
    }
}
