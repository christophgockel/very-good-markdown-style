use crate::document::Document;
use crate::violation::Violation;

/// A single, named style expectation about Markdown.
///
/// Every rule can *detect* breaches. A rule may also *fix* them: `fix` returns
/// the rewritten source, or `None` when the rule is detect-only. This is the
/// unified rule model (see docs/adr/0001) — `lint` runs detectors, `format`
/// applies fixers, and both come from one definition per rule.
pub trait Rule {
    fn id(&self) -> &'static str;

    fn detect(&self, doc: &Document) -> Vec<Violation>;

    fn fix(&self, _doc: &Document) -> Option<String> {
        None
    }
}
