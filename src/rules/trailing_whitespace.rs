use crate::document::Document;
use crate::rule::Rule;
use crate::violation::{Span, Violation};

/// Trailing whitespace is stripped, except a two-or-more space run before a
/// non-empty line, which is a Markdown hard line break and is kept (normalised
/// to exactly two spaces). Line endings are preserved as written.
pub struct TrailingWhitespace;

const ID: &str = "trailing-whitespace";

impl Rule for TrailingWhitespace {
    fn id(&self) -> &'static str {
        ID
    }

    fn detect(&self, doc: &Document) -> Vec<Violation> {
        analyze(&doc.source).0
    }

    fn fix(&self, doc: &Document) -> Option<String> {
        Some(analyze(&doc.source).1)
    }
}

struct Line<'a> {
    content: &'a str,
    terminator: &'a str,
}

fn split_lines(source: &str) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    let mut rest = source;
    loop {
        match rest.find('\n') {
            Some(idx) => {
                let raw = &rest[..idx];
                let (content, terminator) = match raw.strip_suffix('\r') {
                    Some(without_cr) => (without_cr, "\r\n"),
                    None => (raw, "\n"),
                };
                lines.push(Line {
                    content,
                    terminator,
                });
                rest = &rest[idx + 1..];
                if rest.is_empty() {
                    break;
                }
            }
            None => {
                lines.push(Line {
                    content: rest,
                    terminator: "",
                });
                break;
            }
        }
    }
    lines
}

/// Detect and fix in one pass so the two halves can never disagree.
fn analyze(source: &str) -> (Vec<Violation>, String) {
    let lines = split_lines(source);
    let mut violations = Vec::new();
    let mut out = String::with_capacity(source.len());

    for (i, line) in lines.iter().enumerate() {
        let stripped = line.content.trim_end_matches([' ', '\t']);
        let trailing = &line.content[stripped.len()..];

        let next_blank = match lines.get(i + 1) {
            Some(next) => next.content.trim().is_empty(),
            None => true,
        };
        let hard_break = !stripped.is_empty()
            && !next_blank
            && trailing.len() >= 2
            && trailing.bytes().all(|b| b == b' ');

        let content = if trailing.is_empty() {
            line.content.to_string()
        } else if hard_break {
            if trailing.len() != 2 {
                violations.push(violation(
                    i,
                    stripped,
                    trailing,
                    "hard line break has more than two trailing spaces",
                ));
            }
            format!("{stripped}  ")
        } else {
            violations.push(violation(i, stripped, trailing, "trailing whitespace"));
            stripped.to_string()
        };

        out.push_str(&content);
        out.push_str(line.terminator);
    }

    (violations, out)
}

fn violation(index: usize, stripped: &str, trailing: &str, message: &str) -> Violation {
    Violation {
        rule_id: ID,
        message: message.to_string(),
        span: Span {
            line: index + 1,
            column: stripped.chars().count() + 1,
            length: trailing.chars().count(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(source: &str) -> String {
        TrailingWhitespace.fix(&Document::new(source)).unwrap()
    }

    fn detect(source: &str) -> Vec<Violation> {
        TrailingWhitespace.detect(&Document::new(source))
    }

    #[test]
    fn strips_simple_trailing_whitespace() {
        assert_eq!(fix("foo   \n"), "foo\n");
    }

    #[test]
    fn reports_simple_trailing_whitespace_with_span() {
        let violations = detect("foo   \n");
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "trailing-whitespace");
        assert_eq!(
            violations[0].span,
            Span {
                line: 1,
                column: 4,
                length: 3
            }
        );
    }

    #[test]
    fn keeps_two_space_hard_break_before_text() {
        assert_eq!(fix("foo  \nbar\n"), "foo  \nbar\n");
        assert!(detect("foo  \nbar\n").is_empty());
    }

    #[test]
    fn normalises_over_long_hard_break_to_two_spaces() {
        assert_eq!(fix("foo    \nbar\n"), "foo  \nbar\n");
        let violations = detect("foo    \nbar\n");
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].message,
            "hard line break has more than two trailing spaces"
        );
    }

    #[test]
    fn strips_trailing_spaces_before_a_blank_line() {
        assert_eq!(fix("foo  \n\nbar\n"), "foo\n\nbar\n");
    }

    #[test]
    fn strips_single_trailing_space_before_text() {
        assert_eq!(fix("foo \nbar\n"), "foo\nbar\n");
    }

    #[test]
    fn strips_trailing_tab_even_before_text() {
        assert_eq!(fix("foo\t\nbar\n"), "foo\nbar\n");
    }

    #[test]
    fn does_not_treat_a_whitespace_only_line_as_a_hard_break() {
        assert_eq!(fix("   \nbar\n"), "\nbar\n");
    }

    #[test]
    fn preserves_crlf_line_endings() {
        assert_eq!(fix("foo \r\nbar\r\n"), "foo\r\nbar\r\n");
    }

    #[test]
    fn strips_trailing_whitespace_on_a_final_line_without_a_newline() {
        assert_eq!(fix("foo   "), "foo");
    }

    #[test]
    fn leaves_clean_source_untouched() {
        let clean = "foo\nbar\n";
        assert_eq!(fix(clean), clean);
        assert!(detect(clean).is_empty());
    }
}
