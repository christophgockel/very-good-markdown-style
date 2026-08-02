use std::fs;

/// The rule catalog is generated from the registry, so a change to a rule's
/// reasoning or set must be reflected in the committed doc. Regenerate with
/// `markdown-style rules --markdown > docs/rules.md`.
#[test]
fn rules_doc_is_up_to_date() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/docs/rules.md");
    let committed = fs::read_to_string(path).unwrap();
    let generated = markdown_style::report::rules_catalog();

    assert_eq!(
        committed, generated,
        "docs/rules.md is stale; regenerate with `markdown-style rules --markdown > docs/rules.md`"
    );
}
