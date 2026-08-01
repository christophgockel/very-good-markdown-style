use crate::ast::NodeKind;
use crate::document::Document;
use crate::rule::Rule;
use crate::text::split_lines;
use crate::violation::{Span, Violation};

/// Headings use ATX syntax (`# Heading`), never setext underlines
/// (`Heading\n=====`). ATX shows the level explicitly on the line itself and
/// works for every level, whereas setext only reaches two.
pub struct HeadingStyle;

const ID: &str = "heading-style";

impl Rule for HeadingStyle {
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

/// Detect and fix in one pass so the two halves can never disagree.
fn convert(doc: &Document) -> (Vec<Violation>, String) {
    let lines = split_lines(&doc.source);

    // Setext headings in document order: (first line, underline line, level).
    let mut setext: Vec<(usize, usize, u8)> = Vec::new();
    doc.tree().walk(&mut |node| {
        if let NodeKind::Heading {
            level,
            setext: true,
        } = node.kind
        {
            setext.push((node.span.start, node.span.end, level));
        }
    });

    let mut violations = Vec::new();
    let mut out = String::with_capacity(doc.source.len());
    let mut i = 0;
    while i < lines.len() {
        let line_number = i + 1;
        match setext.iter().find(|(start, ..)| *start == line_number) {
            Some(&(start, end, level)) => {
                // Everything up to the underline is the heading text; ATX is a
                // single line, so multi-line setext content joins with spaces.
                let content = (start..end)
                    .map(|line| lines[line - 1].content.trim())
                    .collect::<Vec<_>>()
                    .join(" ");
                violations.push(Violation {
                    rule_id: ID,
                    message: "use an ATX heading (`# Heading`) instead of a setext heading"
                        .to_string(),
                    span: Span {
                        line: start,
                        column: 1,
                        length: content.chars().count(),
                    },
                });
                out.push_str(&"#".repeat(level as usize));
                out.push(' ');
                out.push_str(&content);
                out.push_str(lines[end - 1].terminator);
                i = end;
            }
            None => {
                out.push_str(lines[i].content);
                out.push_str(lines[i].terminator);
                i += 1;
            }
        }
    }

    (violations, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(source: &str) -> String {
        HeadingStyle.fix(&Document::new(source)).unwrap()
    }

    fn detect(source: &str) -> Vec<Violation> {
        HeadingStyle.detect(&Document::new(source))
    }

    #[test]
    fn converts_a_level_one_setext_heading() {
        assert_eq!(fix("Title\n=====\n"), "# Title\n");
        let violations = detect("Title\n=====\n");
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].message,
            "use an ATX heading (`# Heading`) instead of a setext heading"
        );
        assert_eq!(violations[0].span.line, 1);
    }

    #[test]
    fn converts_a_level_two_setext_heading() {
        assert_eq!(fix("Sub\n---\n"), "## Sub\n");
    }

    #[test]
    fn leaves_atx_headings_untouched() {
        assert_eq!(fix("# Title\n"), "# Title\n");
        assert!(detect("# Title\n").is_empty());
    }

    #[test]
    fn joins_multi_line_setext_content() {
        assert_eq!(fix("One\nTwo\n===\n"), "# One Two\n");
    }

    #[test]
    fn preserves_surrounding_content() {
        assert_eq!(
            fix("# A\n\nTitle\n===\n\nBody\n"),
            "# A\n\n# Title\n\nBody\n"
        );
    }

    #[test]
    fn ignores_setext_like_lines_in_a_code_block() {
        let source = "```\nTitle\n=====\n```\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn preserves_crlf_line_endings() {
        assert_eq!(fix("Title\r\n=====\r\n"), "# Title\r\n");
    }
}
