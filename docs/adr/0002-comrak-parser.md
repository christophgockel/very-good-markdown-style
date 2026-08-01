# Use comrak as the Markdown parser

We parse with comrak, which supports CommonMark, the GFM extensions (tables, task lists, strikethrough, autolinks), and YAML frontmatter out of the box, and exposes a typed AST with source positions.

We chose it over pulldown-cmark (an event stream that would need frontmatter handling and more reconstruction work for structural rules) and markdown-rs (capable but a smaller ecosystem). We do not use comrak's Markdown renderer: fixes are applied as targeted edits to the original source, not by reprinting the tree. Swapping parsers later would touch every structural rule, which is why this is recorded.
