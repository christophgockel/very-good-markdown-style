# Rules

Every rule is always on.
A rule either _fixes_ what it finds or only _flags_ it.
A rule is flag-only when fixing it would mean guessing at your intent, so the tool reports it and leaves the change to you.

The reasoning below is the same text `markdown-style explain <rule>` prints.
If the two ever drift, trust `explain`, since it comes straight from the code.


## Whitespace and file hygiene

### trailing-whitespace

_Fix._ Strip trailing spaces and tabs from the end of a line.

Trailing whitespace is invisible in most editors but appears in diffs and version control, adding noise that hides real changes.
It is stripped everywhere, except a run of two or more spaces before a line with text, which Markdown treats as an intentional hard line break and which is normalised to exactly two spaces.


### hard-tabs

_Fix._ Expand hard tabs to spaces at four-column tab stops.

Hard tabs render at different widths in different tools, so indentation and alignment drift between editors.
They are expanded everywhere except inside code blocks, where a tab may be part of the code.


### final-newline

_Fix._ End the file with exactly one newline and no trailing blank lines.

A single trailing newline is the POSIX convention.
Many tools expect it, and its absence shows up as a "no newline at end of file" marker in diffs.
Extra blank lines at the end carry no meaning, so the file is trimmed to exactly one final newline.
The file's existing line-ending style is preserved, so a CRLF file stays CRLF.


### block-spacing

_Fix._ Normalise the blank lines between top-level blocks.

Consistent spacing makes structure scannable.
There is at most one blank line between blocks, two before a heading that follows text so sections stand out, one before a heading that directly follows another, and one after any heading.
The file never begins with blank lines.
A gap that holds something without a block of its own, like a link reference definition, is left exactly as written so nothing is lost.


## Headings

### heading-increment

_Flag._ Heading levels should increase one step at a time.

Skipping a heading level, for example jumping from `#` straight to `###`, breaks the document outline that screen readers and tables of contents rely on.
This is reported but never fixed automatically, because only you know which level a heading was meant to be.


### heading-style

_Fix._ Use ATX headings (`# Heading`), not setext underlines.

ATX headings state their level explicitly on the same line and work for all six levels, while setext underlines only reach two and put the level on a separate line.
One style throughout keeps headings consistent and easy to scan.


### atx-heading

_Fix._ Use one space after the `#` marker and no closing hashes.

A single space after the marker and no trailing run of `#`s is the plain, canonical ATX form.
Closing hashes and extra spaces are decorative, vary between authors, and add nothing the renderer uses.
The heading text is read from the source, so inline formatting is preserved.


### single-h1

_Flag._ A document should have a single top-level (`#`) heading.

That heading is the document's title.
Several top-level headings usually mean the file is really two documents, or that a heading should sit a level deeper.
This is reported but never fixed, because only you know which it is.


## Code

### code-fence

_Fix and flag._ Use backtick code fences, not tildes or indentation.

Backtick fences are the most widely supported form and let you tag a language for highlighting.
Tilde fences are converted to backticks when it is safe, meaning the code contains no backtick fence of its own.
Indented code blocks are flagged but not converted, because turning indentation into a fence can change how nearby text parses.


## Lists

### list-marker

_Fix._ Use `-` as the marker for unordered lists.

One unordered marker throughout keeps lists visually consistent.
We use `-` because it is the most common choice and never reads as emphasis the way a leading `*` can.
Because the marker comes from the parsed list, a `*` used for emphasis or a thematic break is never touched.


### list-marker-space

_Fix._ Use exactly one space after a list marker.

A single space after the marker keeps list items aligned predictably and the source tidy.
Wider gaps vary between authors and add nothing, so they are collapsed to one space.
A gap of five or more spaces is left alone, since that marks indented content inside the item.


### ordered-list

_Fix._ Number ordered list items in sequence.

Sequential numbers in the source match what the reader sees rendered, so the Markdown is easy to follow and reorder.
The list keeps whatever number it starts from and counts up from there, and the delimiter (`.` or `)`) is preserved.


### nested-indent

_Fix._ Indent nested list items to line up with their parent's content.

Aligning a nested list under the first character of its parent's text keeps the outline readable and matches how the list renders.
The indent is the parent's marker width plus one space, so two under a bullet, three under `1.`, and so on.
Lists inside blockquotes are left alone.


## Inline and blocks

### blockquote-marker

_Fix._ Put one space after each blockquote `>` marker.

A single space after `>` keeps quotes readable in the source and consistent between authors.
Only the marker spacing changes.
Deeper indentation is content and is left as written.


### emphasis

_Fix._ Use `_emphasis_` and `**strong**`.

One marker for each kind of emphasis keeps prose consistent.
We use `_` for emphasis, which stands out from the surrounding text, and `**` for strong, which works even inside a word.
Conversions that would change how the text renders are left alone, so nothing spanning multiple lines is touched, nothing intraword where `_` would not emphasise, and nothing whose content already holds the other marker.


## The core rule

### sentence-per-line

_Fix._ Start each sentence on its own line.

This is the reason the tool exists.
One sentence per line makes repetition and over-long sentences obvious in the source, and keeps diffs to the sentences that actually changed.
It is also why the tool has no line-length rule, because a long line is your cue that a sentence is long.

A paragraph's soft-wrapped lines are joined into logical text and re-split at sentence boundaries.
It applies to prose everywhere it lives, so paragraphs, blockquotes, and list items.
Hard breaks are barriers we never join across, inline code and links are protected so a boundary inside them cannot split them across lines, and a sentence is never split where the new line would start with a block marker.

Sentence boundaries are found with a fast, deliberately conservative heuristic: a small set of rules with a short list of abbreviations.
When it is unsure, it leaves the text joined rather than split it wrongly.
