# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]

### Added

- `lint`, `format`, and `explain` commands, plus `rules` to list the rule set.
- Sentence-per-line formatting as the core feature: prose in paragraphs, blockquotes, and list items is rewritten so every sentence starts on its own line, with no line-length rule.
- An opinionated, always-on set of sixteen rules covering whitespace and file hygiene, heading style and hierarchy, list markers, numbering and indentation, blockquote and emphasis markers, code fences, and blank-line spacing.
- rustc-style diagnostics with a source snippet, a caret at the exact location, and a per-rule `why:` explanation, with long lines windowed so the caret stays aligned.
- `explain <rule>` prints a rule's full reasoning.
- CommonMark and GitHub Flavored Markdown support, with YAML frontmatter preserved.
- Files, directories (walked for Markdown, respecting `.gitignore`), and stdin (`-`) as inputs.
- Exit codes `0` for clean, `1` for violations, and `2` for errors, with fail-fast on the first operational error.
- Idempotent formatting, so running `format` twice makes no further changes.
- `rules --markdown` generates the rule catalogue in `docs/rules.md`, kept current by a test.
