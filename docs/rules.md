# Rules

Every rule is always on.
A rule either _fixes_ what it finds or only _flags_ it.
A rule is flag-only when fixing it would mean guessing at your intent, so the tool reports it and leaves the change to you.
This page is generated from the rules themselves, and the reasoning is the same text `markdown-style explain <rule>` prints.


## trailing-whitespace

_Fix._
Trailing whitespace is invisible but shows up as diff noise.

Trailing spaces and tabs are invisible in most editors but appear in diffs and version control, adding noise that hides real changes.
They are stripped everywhere, except a run of two or more spaces before a line with text, which Markdown treats as an intentional hard line break and which is normalised to exactly two spaces.


## hard-tabs

_Fix._
Use spaces, not hard tabs.

Hard tabs render at different widths in different tools, so indentation and alignment drift between editors.
They are expanded to spaces at four-column tab stops everywhere except inside code blocks, where a tab may be part of the code.


## final-newline

_Fix._
Files should end with exactly one newline.

A single trailing newline is the POSIX convention: many tools expect it, and its absence shows up as a 'no newline at end of file' marker in diffs.
Extra blank lines at the end carry no meaning, so the file is trimmed to exactly one final newline.


## heading-increment

_Flag._
Heading levels should increase one at a time.

Skipping a heading level, for example jumping from # straight to ###, breaks the document outline that screen readers and tables of contents rely on.
Increase depth one level at a time.
This is reported but never fixed automatically, because only you know which level a heading was meant to be.


## heading-style

_Fix._
Use ATX headings (# Heading), not setext underlines.

ATX headings state their level explicitly on the same line and work for all six levels, while setext underlines only reach two and put the level on a separate line.
One style throughout keeps headings consistent and easy to scan.


## atx-heading

_Fix._
Headings use one space after the marker and no closing #s.

A single space after the # marker and no trailing run of #s is the plain, canonical ATX form.
Closing hashes and extra spaces are decorative, vary between authors, and add nothing the renderer uses.


## code-fence

_Fix and flag._
Use backtick code fences, not tildes or indentation.

Backtick fences are the most widely supported form and let you tag a language for highlighting.
Tilde fences are converted to backticks when it is safe, meaning the code contains no backtick fence of its own.
Indented code blocks are reported but not converted, because turning indentation into a fence can change how nearby text parses.


## list-marker

_Fix._
Use `-` for unordered list markers.

One unordered list marker throughout keeps lists visually consistent.
We use `-`: it is the most common choice and never reads as emphasis the way a leading `*` can.


## list-marker-space

_Fix._
Use one space after a list marker.

A single space after the marker keeps list items aligned predictably and the source tidy.
Wider gaps vary between authors and add nothing, so they are collapsed to one space.


## ordered-list

_Fix._
Number ordered list items in sequence.

Sequential numbers in the source match what the reader sees rendered, so the Markdown is easy to follow and reorder.
The list keeps whatever number it starts from and counts up from there.


## blockquote-marker

_Fix._
Put one space after each blockquote marker.

A single space after `>` keeps quotes readable in the source and consistent between authors.
Only the marker spacing changes.
Deeper indentation is content and is left as written.


## nested-indent

_Fix._
Indent nested list items to their parent's content.

Aligning a nested list under the first character of its parent's text keeps the outline readable and matches how the list renders.
The indent is the parent's marker width plus one space: two under a bullet, three under `1.`, and so on.


## emphasis

_Fix._
Use _emphasis_ and **strong**.

One marker for each kind of emphasis keeps prose consistent: `_` for emphasis, which stands out from the surrounding text, and `**` for strong, which works even inside a word.
Conversions that would change how the text renders are left alone.


## sentence-per-line

_Fix._
Start each sentence on its own line.

One sentence per line makes repetition and over-long sentences obvious in the source, keeps diffs to the sentences that actually changed, and is why the tool has no line-length rule: a long line is your cue that a sentence is long.
Hard breaks, inline code, and links are preserved, and a sentence is never split where it would start a line with a block marker.


## block-spacing

_Fix._
Blank lines around blocks and headings are kept consistent.

Consistent spacing makes structure scannable: at most one blank line between blocks, two before a heading that follows text so sections stand out, one before a heading that directly follows another and one after any heading, and no blank lines at the very top of the file.


## single-h1

_Flag._
Use a single top-level heading per document.

A document should have exactly one # heading, its title.
Several top-level headings usually mean the file is really two documents, or that a heading should sit a level deeper.
This is reported but never fixed, because only you know which it is.
