use std::collections::HashMap;

use crate::ast::NodeKind;
use crate::document::Document;
use crate::rule::Rule;
use crate::sentence::split_sentences;
use crate::text::{Line, split_lines};
use crate::violation::{Span, Violation};

/// The tool's core rule: within prose, every sentence begins on its own line.
///
/// A paragraph's soft-wrapped lines are joined into logical text and re-split at
/// sentence boundaries. Author hard breaks (two trailing spaces) are barriers we
/// never join across. Inline code, links, and autolinks are protected so a
/// boundary inside them cannot split them across lines. A boundary that would put
/// a block marker at the start of a line is not taken, so nothing is corrupted.
///
/// Scope for now is top-level paragraphs; list items and blockquotes follow.
pub struct SentencePerLine;

const ID: &str = "sentence-per-line";
const PH_OPEN: char = '\u{E000}';
const PH_CLOSE: char = '\u{E001}';

impl Rule for SentencePerLine {
    fn id(&self) -> &'static str {
        ID
    }

    fn short_reason(&self) -> &'static str {
        "Start each sentence on its own line."
    }

    fn rationale(&self) -> &'static str {
        "One sentence per line makes repetition and over-long sentences obvious \
         in the source, keeps diffs to the sentences that actually changed, and is \
         why the tool has no line-length rule: a long line is your cue that a \
         sentence is long. Hard breaks, inline code, and links are preserved, and \
         a sentence is never split where it would start a line with a block marker."
    }

    fn detect(&self, doc: &Document) -> Vec<Violation> {
        rewrite(doc).0
    }

    fn fix(&self, doc: &Document) -> Option<String> {
        Some(rewrite(doc).1)
    }
}

/// Detect and fix in one pass so the two halves can never disagree.
fn rewrite(doc: &Document) -> (Vec<Violation>, String) {
    let lines = split_lines(&doc.source);
    let newline = if doc.source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };

    // Top-level paragraphs: start line -> (end line, reformatted block).
    let mut replacements: HashMap<usize, (usize, String)> = HashMap::new();
    let mut violations = Vec::new();
    for node in &doc.tree().children {
        if node.kind != NodeKind::Paragraph {
            continue;
        }
        let (start, end) = (node.span.start, node.span.end);
        let slice = &lines[start - 1..end];
        let reformatted = reformat(slice, newline);
        // Compare line structure only; pure trailing-whitespace differences
        // belong to the trailing-whitespace rule, not this one.
        if trailing_normalised(&reformatted) != trailing_normalised(&original(slice)) {
            violations.push(Violation {
                rule_id: ID,
                message: "put each sentence on its own line".to_string(),
                span: Span {
                    line: start,
                    column: 1,
                    length: 1,
                },
            });
        }
        replacements.insert(start, (end, reformatted));
    }

    let mut out = String::with_capacity(doc.source.len());
    let mut line_number = 1;
    while line_number <= lines.len() {
        if let Some((end, replacement)) = replacements.get(&line_number) {
            out.push_str(replacement);
            line_number = end + 1;
        } else {
            let line = &lines[line_number - 1];
            out.push_str(line.content);
            out.push_str(line.terminator);
            line_number += 1;
        }
    }

    (violations, out)
}

fn original(slice: &[Line<'_>]) -> String {
    slice
        .iter()
        .map(|line| format!("{}{}", line.content, line.terminator))
        .collect()
}

fn trailing_normalised(text: &str) -> String {
    split_lines(text)
        .iter()
        .map(|line| format!("{}{}", line.content.trim_end(), line.terminator))
        .collect()
}

fn reformat(slice: &[Line<'_>], newline: &str) -> String {
    let last_terminator = slice[slice.len() - 1].terminator;
    let segments = segment_on_hard_breaks(slice);

    let mut out_lines: Vec<String> = Vec::new();
    let segment_count = segments.len();
    for (segment_index, segment) in segments.iter().enumerate() {
        let joined = segment
            .iter()
            .map(|content| content.trim())
            .filter(|content| !content.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        let (masked, protected) = mask(&joined);
        let sentences: Vec<String> = split_sentences(&masked)
            .into_iter()
            .map(|sentence| restore(&sentence, &protected))
            .collect();
        let sentences = merge_block_marker_starts(sentences);

        let is_last_segment = segment_index + 1 == segment_count;
        let last_sentence = sentences.len().saturating_sub(1);
        for (sentence_index, mut sentence) in sentences.into_iter().enumerate() {
            if !is_last_segment && sentence_index == last_sentence {
                sentence.push_str("  ");
            }
            out_lines.push(sentence);
        }
    }

    let mut out = String::new();
    let last = out_lines.len().saturating_sub(1);
    for (index, line) in out_lines.into_iter().enumerate() {
        out.push_str(&line);
        out.push_str(if index == last {
            last_terminator
        } else {
            newline
        });
    }
    out
}

/// Split a paragraph's lines into segments at author hard breaks (two or more
/// trailing spaces), which we never join across.
fn segment_on_hard_breaks<'a>(slice: &'a [Line<'a>]) -> Vec<Vec<&'a str>> {
    let mut segments: Vec<Vec<&str>> = vec![Vec::new()];
    for (index, line) in slice.iter().enumerate() {
        segments.last_mut().unwrap().push(line.content);
        let is_last = index + 1 == slice.len();
        if !is_last && ends_with_hard_break(line.content) {
            segments.push(Vec::new());
        }
    }
    segments
}

fn ends_with_hard_break(content: &str) -> bool {
    let trimmed = content.trim_end_matches(' ');
    content.len() - trimmed.len() >= 2
}

/// Merge back any sentence that would begin a line with a block marker, so the
/// re-emitted text cannot be reparsed as a list, heading, quote, or rule.
fn merge_block_marker_starts(sentences: Vec<String>) -> Vec<String> {
    let mut merged: Vec<String> = Vec::new();
    for sentence in sentences {
        match merged.last_mut() {
            Some(previous) if starts_with_block_marker(&sentence) => {
                previous.push(' ');
                previous.push_str(&sentence);
            }
            _ => merged.push(sentence),
        }
    }
    merged
}

fn starts_with_block_marker(sentence: &str) -> bool {
    sentence.starts_with('#')
        || sentence.starts_with('>')
        || starts_with_bullet(sentence)
        || starts_with_ordered(sentence)
        || is_rule_line(sentence)
}

fn starts_with_bullet(sentence: &str) -> bool {
    let mut chars = sentence.chars();
    match chars.next() {
        Some('-' | '*' | '+') => matches!(chars.next(), Some(' ') | None),
        _ => false,
    }
}

fn starts_with_ordered(sentence: &str) -> bool {
    let bytes = sentence.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 || i >= bytes.len() || (bytes[i] != b'.' && bytes[i] != b')') {
        return false;
    }
    i + 1 == bytes.len() || bytes[i + 1] == b' '
}

fn is_rule_line(sentence: &str) -> bool {
    let trimmed = sentence.trim();
    trimmed.chars().count() >= 3 && trimmed.chars().all(|c| matches!(c, '-' | '=' | '*' | '_'))
}

/// Replace inline code, links, images, and autolinks with placeholders that
/// contain no sentence punctuation, so the splitter cannot break inside them.
fn mask(text: &str) -> (String, Vec<String>) {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut protected: Vec<String> = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c == '`' {
            let run = run_length(&chars, i, '`');
            if let Some(close) = closing_backticks(&chars, i + run, run) {
                protect(&mut out, &mut protected, &chars[i..close + run]);
                i = close + run;
                continue;
            }
        } else if c == '<' {
            if let Some(gt) = chars[i + 1..]
                .iter()
                .position(|&c| c == '>')
                .map(|p| i + 1 + p)
                && !chars[i + 1..gt].iter().any(|c| c.is_whitespace())
            {
                protect(&mut out, &mut protected, &chars[i..gt + 1]);
                i = gt + 1;
                continue;
            }
        } else if c == '[' || (c == '!' && chars.get(i + 1) == Some(&'[')) {
            let bracket = if c == '!' { i + 1 } else { i };
            if let Some(end) = link_end(&chars, bracket) {
                protect(&mut out, &mut protected, &chars[i..end]);
                i = end;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }

    (out, protected)
}

fn link_end(chars: &[char], bracket: usize) -> Option<usize> {
    let close = matching(chars, bracket, '[', ']')?;
    let mut end = close + 1;
    match chars.get(end) {
        Some('(') => {
            if let Some(paren) = matching(chars, end, '(', ')') {
                end = paren + 1;
            }
        }
        Some('[') => {
            if let Some(reference) = matching(chars, end, '[', ']') {
                end = reference + 1;
            }
        }
        _ => {}
    }
    Some(end)
}

fn matching(chars: &[char], open: usize, open_char: char, close_char: char) -> Option<usize> {
    let mut depth = 0;
    for (offset, &c) in chars[open..].iter().enumerate() {
        if c == open_char {
            depth += 1;
        } else if c == close_char {
            depth -= 1;
            if depth == 0 {
                return Some(open + offset);
            }
        }
    }
    None
}

fn run_length(chars: &[char], from: usize, c: char) -> usize {
    chars[from..].iter().take_while(|&&x| x == c).count()
}

fn closing_backticks(chars: &[char], from: usize, run: usize) -> Option<usize> {
    let mut i = from;
    while i < chars.len() {
        if chars[i] == '`' {
            let length = run_length(chars, i, '`');
            if length == run {
                return Some(i);
            }
            i += length;
        } else {
            i += 1;
        }
    }
    None
}

fn protect(out: &mut String, protected: &mut Vec<String>, span: &[char]) {
    out.push(PH_OPEN);
    out.push_str(&protected.len().to_string());
    out.push(PH_CLOSE);
    protected.push(span.iter().collect());
}

fn restore(sentence: &str, protected: &[String]) -> String {
    let mut restored = sentence.to_string();
    for (index, original) in protected.iter().enumerate() {
        let placeholder = format!("{PH_OPEN}{index}{PH_CLOSE}");
        restored = restored.replace(&placeholder, original);
    }
    restored
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(source: &str) -> String {
        SentencePerLine.fix(&Document::new(source)).unwrap()
    }

    fn detect(source: &str) -> Vec<Violation> {
        SentencePerLine.detect(&Document::new(source))
    }

    #[test]
    fn leaves_a_single_sentence_paragraph_alone() {
        let source = "Hello world.\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn splits_a_multi_sentence_line() {
        assert_eq!(fix("One. Two.\n"), "One.\nTwo.\n");
        assert_eq!(detect("One. Two.\n").len(), 1);
    }

    #[test]
    fn joins_soft_wrapped_lines_of_one_sentence() {
        assert_eq!(fix("One two\nthree four.\n"), "One two three four.\n");
    }

    #[test]
    fn keeps_a_hard_break_as_a_barrier() {
        // The hard break after the first line is preserved; the second segment
        // still splits into one sentence per line.
        assert_eq!(fix("first  \nfoo. Bar\n"), "first  \nfoo.\nBar\n");
    }

    #[test]
    fn protects_an_inline_code_span() {
        assert_eq!(
            fix("Call `a. b` now. Then stop.\n"),
            "Call `a. b` now.\nThen stop.\n"
        );
    }

    #[test]
    fn protects_a_link() {
        assert_eq!(
            fix("See [a. b](http://x). Next.\n"),
            "See [a. b](http://x).\nNext.\n"
        );
    }

    #[test]
    fn does_not_break_before_a_block_marker() {
        let source = "See below. - not a list.\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn leaves_code_blocks_untouched() {
        let source = "```\nx. y. z\n```\n";
        assert_eq!(fix(source), source);
        assert!(detect(source).is_empty());
    }

    #[test]
    fn preserves_crlf_line_endings() {
        assert_eq!(fix("One. Two.\r\n"), "One.\r\nTwo.\r\n");
    }
}
