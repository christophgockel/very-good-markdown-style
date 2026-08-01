use crate::ast::NodeKind;
use crate::document::Document;
use crate::rule::Rule;
use crate::violation::{Span, Violation};

/// Heading levels increase one step at a time. Jumping from `#` straight to
/// `###` skips a level and breaks the document outline. Detect-only: the intended
/// level is the author's to decide, so we never rewrite it.
pub struct HeadingIncrement;

const ID: &str = "heading-increment";

impl Rule for HeadingIncrement {
    fn id(&self) -> &'static str {
        ID
    }

    fn detect(&self, doc: &Document) -> Vec<Violation> {
        let mut headings = Vec::new();
        doc.tree().walk(&mut |node| {
            if let NodeKind::Heading { level, .. } = node.kind {
                headings.push((level, node.span.start));
            }
        });

        let mut violations = Vec::new();
        let mut previous: Option<u8> = None;
        for (level, line) in headings {
            if let Some(previous) = previous
                && level > previous + 1
            {
                violations.push(Violation {
                    rule_id: ID,
                    message: format!(
                        "heading level jumps from {previous} to {level}; increase one level at a time"
                    ),
                    span: Span {
                        line,
                        column: 1,
                        length: level as usize,
                    },
                });
            }
            previous = Some(level);
        }
        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect(source: &str) -> Vec<Violation> {
        HeadingIncrement.detect(&Document::new(source))
    }

    #[test]
    fn flags_a_skipped_level() {
        let violations = detect("# One\n\n### Three\n");
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].message,
            "heading level jumps from 1 to 3; increase one level at a time"
        );
        assert_eq!(violations[0].span.line, 3);
    }

    #[test]
    fn allows_stepping_up_one_level() {
        assert!(detect("# One\n\n## Two\n\n### Three\n").is_empty());
    }

    #[test]
    fn allows_jumping_back_to_a_shallower_level() {
        assert!(detect("# One\n\n## Two\n\n### Three\n\n# Another\n").is_empty());
    }

    #[test]
    fn does_not_require_the_first_heading_to_be_h1() {
        assert!(detect("## Starts deep\n\n### Deeper\n").is_empty());
    }

    #[test]
    fn ignores_hashes_inside_code_blocks() {
        let source = "# Real\n\n```\n### not a heading\n```\n";
        assert!(detect(source).is_empty());
    }
}
