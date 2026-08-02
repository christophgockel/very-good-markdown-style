use std::collections::HashMap;

use crate::ast::NodeKind;
use crate::document::Document;
use crate::rule::Rule;
use crate::text::split_lines;
use crate::violation::{Span, Violation};

/// Code blocks use backtick fences. Tilde fences are converted to backticks when
/// it is safe, and indented code blocks are flagged (but not converted, since
/// that can change how the surrounding text parses).
pub struct CodeFence;

const ID: &str = "code-fence";

impl Rule for CodeFence {
    fn id(&self) -> &'static str {
        ID
    }

    fn kind(&self) -> crate::rule::RuleKind {
        crate::rule::RuleKind::Both
    }

    fn short_reason(&self) -> &'static str {
        "Use backtick code fences, not tildes or indentation."
    }

    fn rationale(&self) -> &'static str {
        "Backtick fences are the most widely supported form and let you tag a \
         language for highlighting. Tilde fences are converted to backticks when \
         it is safe, meaning the code contains no backtick fence of its own. \
         Indented code blocks are reported but not converted, because turning \
         indentation into a fence can change how nearby text parses."
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
    let mut violations = Vec::new();
    let mut edits: HashMap<usize, String> = HashMap::new();

    doc.tree().walk(&mut |node| match node.kind {
        NodeKind::CodeBlock { fenced: true } => {
            let open = lines[node.span.start - 1].content;
            if !starts_with_fence(open, '~') {
                return;
            }
            // Converting to backticks is unsafe if the code itself contains a
            // backtick fence, which would close the block early.
            let contains_backtick_fence = (node.span.start + 1..=node.span.end)
                .any(|line| line_starts_with_backtick_fence(&lines, line));
            if contains_backtick_fence {
                return;
            }

            violations.push(violation(
                node.span.start,
                "use a backtick code fence (```) instead of tildes",
            ));
            edits.insert(node.span.start, to_backtick_fence(open));
            if let Some(close) = lines.get(node.span.end - 1)
                && starts_with_fence(close.content, '~')
            {
                edits.insert(node.span.end, to_backtick_fence(close.content));
            }
        }
        NodeKind::CodeBlock { fenced: false } => {
            violations.push(violation(
                node.span.start,
                "use a fenced code block (```) instead of indentation",
            ));
        }
        _ => {}
    });

    let mut out = String::with_capacity(doc.source.len());
    for (index, line) in lines.iter().enumerate() {
        match edits.get(&(index + 1)) {
            Some(replacement) => out.push_str(replacement),
            None => out.push_str(line.content),
        }
        out.push_str(line.terminator);
    }

    (violations, out)
}

fn starts_with_fence(line: &str, fence: char) -> bool {
    line.trim_start_matches(' ').starts_with(fence)
}

fn line_starts_with_backtick_fence(lines: &[crate::text::Line<'_>], line: usize) -> bool {
    lines
        .get(line - 1)
        .is_some_and(|line| line.content.trim_start_matches(' ').starts_with("```"))
}

fn to_backtick_fence(line: &str) -> String {
    let indent_len = line.len() - line.trim_start_matches(' ').len();
    let (indent, rest) = line.split_at(indent_len);
    let run = rest.chars().take_while(|c| *c == '~').count();
    format!("{indent}{}{}", "`".repeat(run), &rest[run..])
}

fn violation(line: usize, message: &str) -> Violation {
    Violation {
        rule_id: ID,
        message: message.to_string(),
        span: Span {
            line,
            column: 1,
            length: 3,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(source: &str) -> String {
        CodeFence.fix(&Document::new(source)).unwrap()
    }

    fn detect(source: &str) -> Vec<Violation> {
        CodeFence.detect(&Document::new(source))
    }

    #[test]
    fn converts_a_tilde_fence_to_backticks() {
        assert_eq!(fix("~~~\ncode\n~~~\n"), "```\ncode\n```\n");
        let violations = detect("~~~\ncode\n~~~\n");
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].message,
            "use a backtick code fence (```) instead of tildes"
        );
    }

    #[test]
    fn keeps_the_info_string() {
        assert_eq!(fix("~~~rust\ncode\n~~~\n"), "```rust\ncode\n```\n");
    }

    #[test]
    fn leaves_a_backtick_fence_untouched() {
        let source = "```\ncode\n```\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn does_not_convert_when_the_code_contains_a_backtick_fence() {
        let source = "~~~\n```\ncode\n```\n~~~\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn flags_an_indented_code_block_without_converting_it() {
        let source = "    code\n";
        assert_eq!(fix(source), source);
        let violations = detect(source);
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].message,
            "use a fenced code block (```) instead of indentation"
        );
    }

    #[test]
    fn preserves_crlf_line_endings() {
        assert_eq!(fix("~~~\r\ncode\r\n~~~\r\n"), "```\r\ncode\r\n```\r\n");
    }
}
