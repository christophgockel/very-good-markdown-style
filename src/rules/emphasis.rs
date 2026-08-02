use std::collections::HashMap;

use crate::ast::{Node, NodeKind};
use crate::document::Document;
use crate::rule::Rule;
use crate::text::split_lines;
use crate::violation::{Span, Violation};

/// Emphasis uses `_text_` and strong uses `**text**`. `*text*` becomes `_text_`
/// and `__text__` becomes `**text**`.
///
/// Conversions that could change meaning are skipped: nothing spanning multiple
/// lines, nothing intraword (where `_` would not emphasise), and nothing whose
/// content already holds the other marker.
pub struct Emphasis;

const ID: &str = "emphasis";

impl Rule for Emphasis {
    fn id(&self) -> &'static str {
        ID
    }

    fn short_reason(&self) -> &'static str {
        "Use _emphasis_ and **strong**."
    }

    fn rationale(&self) -> &'static str {
        "One marker for each kind of emphasis keeps prose consistent: `_` for \
         emphasis, which stands out from the surrounding text, and `**` for \
         strong, which works even inside a word. Conversions that would change how \
         the text renders are left alone."
    }

    fn detect(&self, doc: &Document) -> Vec<Violation> {
        rewrite(doc).0
    }

    fn fix(&self, doc: &Document) -> Option<String> {
        Some(rewrite(doc).1)
    }
}

struct Edit {
    index: usize,
    replacement: Vec<char>,
}

/// Detect and fix in one pass so the two halves can never disagree.
fn rewrite(doc: &Document) -> (Vec<Violation>, String) {
    let lines = split_lines(&doc.source);

    let mut edits: HashMap<usize, Vec<Edit>> = HashMap::new();
    let mut violations = Vec::new();
    doc.tree()
        .walk(&mut |node| consider(node, &lines, &mut edits, &mut violations));

    let mut out = String::with_capacity(doc.source.len());
    for (index, line) in lines.iter().enumerate() {
        match edits.get(&(index + 1)) {
            Some(line_edits) => out.push_str(&apply(line.content, line_edits)),
            None => out.push_str(line.content),
        }
        out.push_str(line.terminator);
    }

    (violations, out)
}

fn consider(
    node: &Node,
    lines: &[crate::text::Line<'_>],
    edits: &mut HashMap<usize, Vec<Edit>>,
    violations: &mut Vec<Violation>,
) {
    // Only single-line spans are safe to rewrite by column.
    if node.span.start != node.span.end {
        return;
    }
    let (marker_len, from, to) = match node.kind {
        NodeKind::Emphasis => (1, '*', '_'),
        NodeKind::Strong => (2, '_', '*'),
        _ => return,
    };

    let chars: Vec<char> = lines[node.span.start - 1].content.chars().collect();
    let open = node.start_column - 1;
    let close = node.end_column - marker_len;

    if chars.get(open) != Some(&from) {
        return; // already the preferred marker
    }
    // Content between the markers must not contain the other marker, which would
    // create an ambiguous run once converted.
    let inner = &chars[open + marker_len..close];
    if inner.contains(&to) {
        return;
    }
    // Emphasis with `_` needs non-word characters on both sides.
    if node.kind == NodeKind::Emphasis
        && (is_word(chars.get(open.wrapping_sub(1))) || is_word(chars.get(close + marker_len)))
    {
        return;
    }

    let replacement = vec![to; marker_len];
    edits.entry(node.span.start).or_default().push(Edit {
        index: open,
        replacement: replacement.clone(),
    });
    edits.entry(node.span.start).or_default().push(Edit {
        index: close,
        replacement,
    });
    violations.push(Violation {
        rule_id: ID,
        message: format!(
            "use {} for {}",
            if to == '_' { "`_`" } else { "`**`" },
            if to == '_' {
                "emphasis"
            } else {
                "strong emphasis"
            }
        ),
        span: Span {
            line: node.span.start,
            column: node.start_column,
            length: marker_len,
        },
    });
}

fn is_word(c: Option<&char>) -> bool {
    c.is_some_and(|c| c.is_alphanumeric())
}

fn apply(content: &str, edits: &[Edit]) -> String {
    let mut chars: Vec<char> = content.chars().collect();
    for edit in edits {
        for (offset, &c) in edit.replacement.iter().enumerate() {
            chars[edit.index + offset] = c;
        }
    }
    chars.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(source: &str) -> String {
        Emphasis.fix(&Document::new(source)).unwrap()
    }

    fn detect(source: &str) -> Vec<Violation> {
        Emphasis.detect(&Document::new(source))
    }

    #[test]
    fn converts_star_emphasis_to_underscore() {
        assert_eq!(fix("a *word* here\n"), "a _word_ here\n");
        assert_eq!(detect("a *word* here\n").len(), 1);
    }

    #[test]
    fn converts_underscore_strong_to_stars() {
        assert_eq!(fix("a __word__ here\n"), "a **word** here\n");
    }

    #[test]
    fn leaves_preferred_markers_untouched() {
        let source = "a _word_ and **strong** here\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn does_not_convert_intraword_emphasis() {
        // `_` would not emphasise inside a word, so `*` must stay.
        let source = "foo*bar*baz\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn leaves_literal_intraword_underscores_alone() {
        // `__` does not emphasise inside a word, so it is literal, not strong.
        let source = "foo__bar__baz\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn preserves_crlf_line_endings() {
        assert_eq!(fix("a *word*\r\n"), "a _word_\r\n");
    }
}
