/// A location within a document, pointing at the run of source a violation covers.
///
/// Lines and columns are 1-based; `length` is the number of characters the span
/// covers on that line. Columns count characters, not bytes, so carets line up
/// under multi-byte text when rendered later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

/// A single place where a rule is breached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub rule_id: &'static str,
    pub message: String,
    pub span: Span,
}
