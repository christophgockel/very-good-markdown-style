# markdown-style

An opinionated command-line tool that lints and formats Markdown files. It reports style problems with helpful explanations and can fix them in place.

## Language

**Rule**:
A single, named style expectation about Markdown (for example "headings use ATX syntax"). Every rule knows how to *detect* breaches and may also know how to *fix* them.
_Avoid_: Check, policy

**Violation**:
A specific place in a document where a rule is breached. Carries a source location and a human explanation.
_Avoid_: Error, warning, issue, offense

**Detector**:
The part of a rule that finds violations in a document.

**Fixer**:
The part of a rule that rewrites the source to remove a violation. Not every rule has one.
_Avoid_: Autofix, corrector

**Lint**:
Run every rule's detector against a document and report the violations. Never changes the file.

**Format**:
Apply every fixable rule's fixer to a document, rewriting the source in place for files or to stdout when reading stdin. Never changes the file's existing line-ending style.
_Avoid_: Fix, prettify, reformat

**Document**:
One Markdown file, held as its original source text plus whatever parsed structure the rules need.

**Sentence-per-line**:
The tool's central rule: within prose, every sentence begins on its own physical line. It makes repetition and over-long sentences visible in the source, which is why the tool has no line-length rule. Format rewrites paragraphs to satisfy it.
_Avoid_: ventilated prose, semantic line breaks (a looser style that also breaks mid-sentence at clauses)

**Prose**:
The running text the sentence-per-line rule may rewrite: paragraphs, and text inside list items and blockquotes. Excludes anything left verbatim (code, tables, headings, frontmatter, link definitions).
_Avoid_: body, content, text
