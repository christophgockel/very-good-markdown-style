pub mod ast;
pub mod document;
pub mod parser;
pub mod rule;
pub mod rules;
pub mod text;
pub mod violation;

pub use document::Document;
pub use rule::Rule;
pub use violation::{Span, Violation};

/// The built-in, opinionated rule set. Every rule is always on.
pub fn default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(rules::trailing_whitespace::TrailingWhitespace),
        Box::new(rules::final_newline::FinalNewline),
        Box::new(rules::heading_increment::HeadingIncrement),
        Box::new(rules::heading_style::HeadingStyle),
    ]
}

/// Run every rule's detector against a document and collect the violations.
pub fn lint(doc: &Document, rules: &[Box<dyn Rule>]) -> Vec<Violation> {
    rules.iter().flat_map(|rule| rule.detect(doc)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lint_runs_the_default_rules() {
        let doc = Document::new("foo   \n");
        let violations = lint(&doc, &default_rules());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "trailing-whitespace");
    }
}
