# markdown-style

Project-specific guidance for this repo.


## Checks

Every commit is green.
Before committing, `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` all pass.


## Testing

Never put real content into tests.
When reproducing a reported case, build a synthetic, made-up fixture that exercises the same structure, not the actual prose or data from a real document.
Real content in tests invites privacy and data-policy problems and couples tests to text that can change or shouldn't be redistributed.


## Generated files

`docs/rules.md` is generated from the rule registry, so don't hand-edit it.
Regenerate it with `markdown-style rules --markdown > docs/rules.md`.
The `tests/docs.rs` test fails if it is stale.


## Dogfooding

All Markdown in the repo must pass the tool, and CI enforces this with `markdown-style lint .`.
After changing any Markdown, run `markdown-style format .` so it stays conformant.


## Changelog

Every user-facing change (a new, changed, or removed rule, or a change to CLI behaviour) gets an entry under `[Unreleased]` in `CHANGELOG.md`, using the Keep a Changelog headings (`Added`, `Changed`, `Fixed`, and so on).


## Design

Vocabulary lives in `CONTEXT.md` and architectural decisions in `docs/adr/`.
Each rule is one definition with a detector and an optional fixer (see ADR 0001).
To add a rule, implement `Rule` (`id`, `short_reason`, `rationale`, `kind`, `detect`, and an optional `fix`), register it in `default_rules`, add tests with synthetic fixtures, and regenerate `docs/rules.md`.
