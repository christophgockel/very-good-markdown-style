//! The one place comrak is used. Everything else works against `crate::ast`.

use crate::ast::{LineSpan, Node, NodeKind};
use comrak::nodes::{AstNode, ListType, NodeValue};
use comrak::{Arena, Options, parse_document};

/// Parse Markdown into the tool's internal representation.
///
/// CommonMark plus the GFM extensions and YAML frontmatter, matching the
/// dialect the tool targets.
pub fn parse(source: &str) -> Node {
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.extension.front_matter_delimiter = Some("---".to_string());

    let root = parse_document(&arena, source, &options);
    map(root).expect("the document root is always a block")
}

fn map<'a>(node: &'a AstNode<'a>) -> Option<Node> {
    let data = node.data.borrow();
    let kind = match &data.value {
        NodeValue::Document => NodeKind::Document,
        NodeValue::FrontMatter(_) => NodeKind::FrontMatter,
        NodeValue::Heading(heading) => NodeKind::Heading {
            level: heading.level,
            setext: heading.setext,
        },
        NodeValue::Paragraph => NodeKind::Paragraph,
        NodeValue::CodeBlock(code) => NodeKind::CodeBlock {
            fenced: code.fenced,
        },
        NodeValue::List(list) => NodeKind::List {
            ordered: matches!(list.list_type, ListType::Ordered),
        },
        NodeValue::Item(_) => NodeKind::Item,
        NodeValue::BlockQuote => NodeKind::BlockQuote,
        NodeValue::ThematicBreak => NodeKind::ThematicBreak,
        NodeValue::Table(_) => NodeKind::Table,
        NodeValue::HtmlBlock(_) => NodeKind::HtmlBlock,
        value if value.block() => NodeKind::Other,
        // Inline nodes carry no block structure we model yet.
        _ => return None,
    };

    let sourcepos = data.sourcepos;
    let span = LineSpan {
        start: sourcepos.start.line,
        end: sourcepos.end.line,
    };
    let children = node.children().filter_map(map).collect();

    Some(Node {
        kind,
        span,
        children,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<NodeKind> {
        let mut kinds = Vec::new();
        parse(source).walk(&mut |node| kinds.push(node.kind.clone()));
        kinds
    }

    #[test]
    fn maps_headings_with_level_and_style() {
        let tree = parse("# Title\n\nText\n");
        let mut headings = Vec::new();
        tree.walk(&mut |node| {
            if let NodeKind::Heading { level, setext } = node.kind {
                headings.push((level, setext, node.span.start));
            }
        });
        assert_eq!(headings, vec![(1, false, 1)]);
    }

    #[test]
    fn recognises_setext_headings() {
        let tree = parse("Title\n=====\n");
        let mut setext = false;
        tree.walk(&mut |node| {
            if let NodeKind::Heading { setext: s, .. } = node.kind {
                setext = s;
            }
        });
        assert!(setext);
    }

    #[test]
    fn maps_fenced_code_blocks() {
        assert!(kinds("```\ncode\n```\n").contains(&NodeKind::CodeBlock { fenced: true }));
    }

    #[test]
    fn maps_frontmatter() {
        let source = "---\ntitle: x\n---\n\n# H\n";
        assert!(kinds(source).contains(&NodeKind::FrontMatter));
    }

    #[test]
    fn does_not_emit_nodes_for_inline_content() {
        // A single paragraph of styled text is one block node, not one per span.
        let paragraphs = kinds("This is *emphasised* and `code`.\n")
            .iter()
            .filter(|kind| **kind == NodeKind::Paragraph)
            .count();
        assert_eq!(paragraphs, 1);
    }
}
