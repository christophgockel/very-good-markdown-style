use crate::document::Document;
use crate::violation::Violation;

/// Whether a rule fixes what it finds, only flags it, or both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    /// Every violation this rule reports can be fixed automatically.
    Fix,
    /// Detect-only: reported but never rewritten, because fixing would guess.
    Flag,
    /// Some violations are fixed and some are only flagged.
    Both,
}

impl RuleKind {
    pub fn label(self) -> &'static str {
        match self {
            RuleKind::Fix => "fix",
            RuleKind::Flag => "flag",
            RuleKind::Both => "fix+flag",
        }
    }
}

/// A single, named style expectation about Markdown.
///
/// Every rule can *detect* breaches. A rule may also *fix* them: `fix` returns
/// the rewritten source, or `None` when the rule is detect-only. This is the
/// unified rule model (see docs/adr/0001) — `lint` runs detectors, `format`
/// applies fixers, and both come from one definition per rule.
///
/// Every rule also explains itself: a one-line `short_reason` shown with each
/// violation, and a fuller `rationale` shown once per run and by `explain`.
pub trait Rule {
    fn id(&self) -> &'static str;

    /// One line, shown with every violation: why this rule exists.
    fn short_reason(&self) -> &'static str;

    /// The full reasoning, shown once per run and by the `explain` command.
    fn rationale(&self) -> &'static str;

    /// Whether the rule fixes, flags, or both. Defaults to `Fix`, since most
    /// rules are fully fixable; detect-only and mixed rules override it.
    fn kind(&self) -> RuleKind {
        RuleKind::Fix
    }

    fn detect(&self, doc: &Document) -> Vec<Violation>;

    fn fix(&self, _doc: &Document) -> Option<String> {
        None
    }
}
