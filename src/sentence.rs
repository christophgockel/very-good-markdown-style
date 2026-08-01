//! Heuristic sentence splitting for the sentence-per-line rule.
//!
//! Works on a single logical line of plain text (no newlines). A boundary is a
//! run of `.`/`!`/`?` (with any closing quotes) followed by whitespace and the
//! start of a new sentence. A curated abbreviation list and single-letter
//! initials suppress false boundaries. It is deliberately conservative: when in
//! doubt it does not split, so it never runs together with per-run churn.

/// Common abbreviations that end in a period but do not end a sentence. Stored
/// lowercase, with the trailing period.
const ABBREVIATIONS: &[&str] = &[
    "e.g.", "i.e.", "etc.", "vs.", "cf.", "al.", "esp.", "approx.", "dr.", "mr.", "mrs.", "ms.",
    "prof.", "sr.", "jr.", "st.", "no.", "vol.", "fig.", "pp.", "inc.", "ltd.", "co.", "u.s.",
    "u.k.", "a.m.", "p.m.",
];

/// Split `text` into sentences, each trimmed of surrounding whitespace.
pub fn split_sentences(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut sentences = Vec::new();
    let mut start = 0;
    let mut index = 0;

    while index < chars.len() {
        if !is_terminator(chars[index]) {
            index += 1;
            continue;
        }

        let mut run_end = index + 1;
        while run_end < chars.len() && is_terminator(chars[run_end]) {
            run_end += 1;
        }
        let mut after = run_end;
        while after < chars.len() && is_closing(chars[after]) {
            after += 1;
        }

        if is_boundary(&chars, index, run_end, after) {
            push_sentence(&mut sentences, &chars[start..after]);
            let next = after
                + chars[after..]
                    .iter()
                    .take_while(|c| c.is_whitespace())
                    .count();
            start = next;
            index = next;
        } else {
            index = run_end;
        }
    }

    if start < chars.len() {
        push_sentence(&mut sentences, &chars[start..]);
    }
    if sentences.is_empty() {
        sentences.push(text.trim().to_string());
    }
    sentences
}

/// A boundary needs whitespace then the start of a new sentence after the
/// punctuation, and the word being ended must not be an abbreviation or initial.
fn is_boundary(chars: &[char], term_start: usize, run_end: usize, after: usize) -> bool {
    let Some(&whitespace) = chars.get(after) else {
        return false;
    };
    if !whitespace.is_whitespace() {
        return false;
    }
    let next = after
        + chars[after..]
            .iter()
            .take_while(|c| c.is_whitespace())
            .count();
    match chars.get(next) {
        Some(&start) if starts_sentence(start) => {}
        _ => return false,
    }

    let mut word_start = term_start;
    while word_start > 0 && !chars[word_start - 1].is_whitespace() {
        word_start -= 1;
    }
    let token: String = chars[word_start..run_end].iter().collect();
    !is_abbreviation(&token)
}

fn is_abbreviation(token: &str) -> bool {
    let lower = token.to_lowercase();
    if ABBREVIATIONS.contains(&lower.as_str()) {
        return true;
    }
    // A single letter followed by a period is an initial, like "J." in a name.
    let core = token.trim_end_matches(['.', '!', '?']);
    core.chars().count() == 1 && core.chars().all(char::is_alphabetic)
}

fn push_sentence(sentences: &mut Vec<String>, chars: &[char]) {
    let sentence: String = chars.iter().collect();
    let trimmed = sentence.trim();
    if !trimmed.is_empty() {
        sentences.push(trimmed.to_string());
    }
}

fn is_terminator(c: char) -> bool {
    matches!(c, '.' | '!' | '?')
}

fn is_closing(c: char) -> bool {
    matches!(c, '"' | '\'' | ')' | ']' | '\u{201D}' | '\u{2019}')
}

fn starts_sentence(c: char) -> bool {
    c.is_uppercase()
        || c.is_ascii_digit()
        || matches!(c, '"' | '\'' | '(' | '[' | '\u{201C}' | '\u{2018}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_sentence_stays_whole() {
        assert_eq!(split_sentences("One sentence."), vec!["One sentence."]);
    }

    #[test]
    fn splits_on_sentence_boundaries() {
        assert_eq!(
            split_sentences("One. Two. Three."),
            vec!["One.", "Two.", "Three."]
        );
    }

    #[test]
    fn splits_on_all_terminators() {
        assert_eq!(
            split_sentences("One! Two? Three."),
            vec!["One!", "Two?", "Three."]
        );
    }

    #[test]
    fn does_not_split_on_a_lowercase_abbreviation() {
        assert_eq!(
            split_sentences("See e.g. this one."),
            vec!["See e.g. this one."]
        );
    }

    #[test]
    fn does_not_split_an_abbreviation_before_a_capital() {
        assert_eq!(
            split_sentences("Ask Dr. Smith now."),
            vec!["Ask Dr. Smith now."]
        );
    }

    #[test]
    fn does_not_split_initials() {
        assert_eq!(
            split_sentences("J. R. R. Tolkien wrote it."),
            vec!["J. R. R. Tolkien wrote it."]
        );
    }

    #[test]
    fn does_not_split_a_decimal_number() {
        assert_eq!(
            split_sentences("Pi is 3.14 today."),
            vec!["Pi is 3.14 today."]
        );
    }

    #[test]
    fn keeps_a_closing_quote_with_the_sentence() {
        assert_eq!(
            split_sentences("He said \"Go.\" Then left."),
            vec!["He said \"Go.\"", "Then left."]
        );
    }

    #[test]
    fn collapses_extra_spaces_between_sentences() {
        assert_eq!(split_sentences("One.   Two."), vec!["One.", "Two."]);
    }

    #[test]
    fn does_not_split_before_a_lowercase_word() {
        assert_eq!(
            split_sentences("Version 1.0 works. and more"),
            vec!["Version 1.0 works. and more"]
        );
    }
}
