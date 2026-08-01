use std::cell::OnceCell;

use crate::ast::Node;

/// One Markdown file, held as its original source text.
///
/// The source is always the source of truth, so fixes apply to the original
/// bytes. The parsed tree is derived lazily and cached, so the many rules that
/// need structure parse the document only once.
#[derive(Debug, Clone)]
pub struct Document {
    pub source: String,
    tree: OnceCell<Node>,
}

impl Document {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            tree: OnceCell::new(),
        }
    }

    /// The parsed representation, parsed on first use and cached.
    pub fn tree(&self) -> &Node {
        self.tree.get_or_init(|| crate::parser::parse(&self.source))
    }
}
