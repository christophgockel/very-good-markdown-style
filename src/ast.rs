//! The tool's own Markdown representation.
//!
//! Rules operate on these types, never on the parser's. The concrete parser
//! (comrak) is confined to the `parser` module, which maps its tree into this
//! one (see docs/adr/0004). Swapping parsers means rewriting that mapping, not
//! every rule.

/// An inclusive range of 1-based source lines a node covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Document,
    FrontMatter,
    Heading {
        level: u8,
        setext: bool,
    },
    Paragraph,
    CodeBlock {
        fenced: bool,
    },
    List {
        ordered: bool,
    },
    Item,
    BlockQuote,
    ThematicBreak,
    Table,
    HtmlBlock,
    /// A block kind we do not model specifically yet.
    Other,
}

/// One block-level node. Inline content (emphasis, links, code spans) is not
/// modelled here yet; it is added when a rule first needs it.
#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub span: LineSpan,
    /// 1-based column where the node's content starts on its first line. For a
    /// list item's paragraph this is the column after the marker.
    pub start_column: usize,
    pub children: Vec<Node>,
}

impl Node {
    /// Visit this node and every descendant, depth-first, parents before
    /// children.
    pub fn walk<'a>(&'a self, visit: &mut dyn FnMut(&'a Node)) {
        visit(self);
        for child in &self.children {
            child.walk(visit);
        }
    }
}
