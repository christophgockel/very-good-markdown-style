# Sentence-per-line is the tool's core rule

The defining rule is that prose has one sentence per physical line. `format` re-linearises each paragraph: it joins the existing line breaks into logical text and re-splits at sentence boundaries.
This makes repetition and over-long sentences visible in the source, which is also why the tool deliberately has no line-length rule.

This overturns the usual "a formatter never reflows prose" assumption, so it is recorded here.
Key choices and their reasons:

- **Fixer, not just a flag.** `format` actively rewrites prose.
  Users must run it under version control; a wrong split shows up as a visible diff.
- **Heuristic sentence detection with a curated abbreviation list**, not a Unicode segmenter.
  It is fast, tiny, deterministic (so `format` is idempotent), and tunable. `icu_segmenter` was rejected as heavyweight, bad for startup, and still not abbreviation-aware.
- **Hard breaks are barriers.** A paragraph is segmented at author hard breaks (2+ trailing spaces) first, and we never join across one.
- **Scope is all prose**: top-level paragraphs, list-item paragraphs (continuation indented to the marker's content column), and blockquotes.
  Code, tables, headings, frontmatter, link definitions, and raw HTML are left verbatim.
- **Never inject characters into prose.** If a break would put a block marker (`- `, `1. `, `> `, `#`, ...) at the start of a line and thus change parsing, we leave that one boundary un-split rather than escape it with a backslash.
