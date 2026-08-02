use crate::ast::NodeKind;
use crate::document::Document;
use crate::rule::Rule;
use crate::violation::{Span, Violation};

/// A document has a single top-level (`#`) heading: its title. Detect-only,
/// since the right level for an extra top-level heading is the author's call.
pub struct SingleH1;

const ID: &str = "single-h1";

impl Rule for SingleH1 {
    fn id(&self) -> &'static str {
        ID
    }

    fn kind(&self) -> crate::rule::RuleKind {
        crate::rule::RuleKind::Flag
    }

    fn short_reason(&self) -> &'static str {
        "Use a single top-level heading per document."
    }

    fn rationale(&self) -> &'static str {
        "A document should have exactly one # heading, its title. Several \
         top-level headings usually mean the file is really two documents, or \
         that a heading should sit a level deeper. This is reported but never \
         fixed, because only you know which it is."
    }

    fn detect(&self, doc: &Document) -> Vec<Violation> {
        let mut h1_lines = Vec::new();
        doc.tree().walk(&mut |node| {
            if let NodeKind::Heading { level: 1, .. } = node.kind {
                h1_lines.push(node.span.start);
            }
        });

        h1_lines
            .into_iter()
            .skip(1)
            .map(|line| Violation {
                rule_id: ID,
                message: "a document should have a single top-level heading (#)".to_string(),
                span: Span {
                    line,
                    column: 1,
                    length: 1,
                },
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect(source: &str) -> Vec<Violation> {
        SingleH1.detect(&Document::new(source))
    }

    #[test]
    fn allows_a_single_h1() {
        assert!(detect("# Title\n\n## Section\n").is_empty());
    }

    #[test]
    fn flags_each_extra_h1() {
        let violations = detect("# One\n\n# Two\n\n# Three\n");
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].span.line, 3);
        assert_eq!(violations[1].span.line, 5);
    }

    #[test]
    fn allows_a_document_with_no_h1() {
        assert!(detect("## Section\n\n## Another\n").is_empty());
    }

    #[test]
    fn ignores_h1_looking_lines_in_code_blocks() {
        assert!(detect("# Title\n\n```\n# not a heading\n```\n").is_empty());
    }
}
