//! Human-readable, rustc-style rendering of violations.

use std::collections::{HashMap, HashSet};

use crate::rule::Rule;
use crate::text::split_lines;
use crate::violation::Violation;

const RED: &str = "\x1b[31m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// Render violations for one file as a rustc-style report.
///
/// Each violation shows the source line with a caret underline and a one-line
/// `why:`. The fuller rationale is shown once per rule per report, on that
/// rule's first violation.
pub fn render(
    path: &str,
    source: &str,
    violations: &[Violation],
    rules: &[Box<dyn Rule>],
    color: bool,
) -> String {
    let reasons = reason_index(rules);
    let lines = split_lines(source);

    let mut ordered: Vec<&Violation> = violations.iter().collect();
    ordered.sort_by_key(|violation| (violation.span.line, violation.span.column));

    let mut out = String::new();
    let mut explained: HashSet<&str> = HashSet::new();
    for (position, violation) in ordered.iter().enumerate() {
        if position > 0 {
            out.push('\n');
        }
        render_one(
            &mut out,
            path,
            &lines,
            violation,
            reasons.get(violation.rule_id).copied().unwrap_or(("", "")),
            explained.insert(violation.rule_id),
            color,
        );
    }
    out
}

/// The full rationale for one rule, for the `explain` command. `None` when no
/// rule owns the id.
pub fn explain(rule_id: &str, rules: &[Box<dyn Rule>]) -> Option<String> {
    rules
        .iter()
        .find(|rule| rule.id() == rule_id)
        .map(|rule| format!("{rule_id}\n\n{}\n", rule.rationale()))
}

fn reason_index(rules: &[Box<dyn Rule>]) -> HashMap<&str, (&str, &str)> {
    rules
        .iter()
        .map(|rule| (rule.id(), (rule.short_reason(), rule.rationale())))
        .collect()
}

fn render_one(
    out: &mut String,
    path: &str,
    lines: &[crate::text::Line<'_>],
    violation: &Violation,
    reason: (&str, &str),
    show_rationale: bool,
    color: bool,
) {
    let span = &violation.span;
    let gutter = " ".repeat(span.line.to_string().len());
    let (short_reason, rationale) = reason;

    let heading = format!("{}: {}", violation.rule_id, violation.message);
    out.push_str(&paint(&heading, &format!("{BOLD}{RED}"), color));
    out.push('\n');
    out.push_str(&format!(
        "{gutter}--> {path}:{}:{}\n",
        span.line, span.column
    ));
    out.push_str(&format!("{gutter} |\n"));

    let source_line = lines.get(span.line - 1).map_or("", |line| line.content);
    out.push_str(&format!("{} | {source_line}\n", span.line));

    let caret_pad = " ".repeat(span.column.saturating_sub(1));
    let carets = "^".repeat(span.length.max(1));
    out.push_str(&format!(
        "{gutter} | {caret_pad}{}\n",
        paint(&carets, RED, color)
    ));

    if !short_reason.is_empty() {
        out.push_str(&format!("{gutter} = why: {short_reason}\n"));
    }
    if show_rationale && !rationale.is_empty() {
        for (index, line) in rationale.lines().enumerate() {
            let label = if index == 0 {
                " = help: "
            } else {
                "          "
            };
            out.push_str(&format!("{gutter}{label}{}\n", line.trim()));
        }
    }
}

fn paint(text: &str, code: &str, color: bool) -> String {
    if color {
        format!("{code}{text}{RESET}")
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::default_rules;

    fn detect(source: &str) -> Vec<Violation> {
        crate::lint(&Document::new(source), &default_rules())
    }

    #[test]
    fn renders_a_caret_snippet_with_reason() {
        let source = "# Title\n\ntext   \n";
        let report = render("a.md", source, &detect(source), &default_rules(), false);
        // The source line is shown verbatim (with its trailing spaces), and the
        // carets sit under them starting at the flagged column. `line3` keeps the
        // trailing spaces off the end of a physical source line here.
        let line3 = "text   ";
        let why = "Trailing whitespace is invisible but shows up as diff noise.";
        let expected = format!(
            "trailing-whitespace: trailing whitespace\n --> a.md:3:5\n  |\n3 | {line3}\n  |     ^^^\n  = why: {why}\n"
        );
        assert!(report.starts_with(&expected), "got:\n{report}");
    }

    #[test]
    fn shows_the_full_rationale_only_once_per_rule() {
        let source = "text  \n\nmore  \n";
        let report = render("a.md", source, &detect(source), &default_rules(), false);
        assert_eq!(report.matches("= help:").count(), 1);
        assert_eq!(report.matches("= why:").count(), 2);
    }

    #[test]
    fn adds_colour_only_when_asked() {
        let source = "text   \n";
        let plain = render("a.md", source, &detect(source), &default_rules(), false);
        let coloured = render("a.md", source, &detect(source), &default_rules(), true);
        assert!(!plain.contains('\x1b'));
        assert!(coloured.contains('\x1b'));
    }

    #[test]
    fn explain_returns_a_known_rule_rationale() {
        let text = explain("final-newline", &default_rules()).unwrap();
        assert!(text.starts_with("final-newline\n"));
        assert!(text.contains("POSIX"));
    }

    #[test]
    fn explain_is_none_for_an_unknown_rule() {
        assert!(explain("no-such-rule", &default_rules()).is_none());
    }
}
