/// A single source line: its content without the terminator, and the terminator
/// itself (`"\n"`, `"\r\n"`, or `""` for a final line with no trailing newline).
pub struct Line<'a> {
    pub content: &'a str,
    pub terminator: &'a str,
}

/// Split source into lines, preserving each line's terminator so the original
/// text, including its line-ending style, can be reconstructed exactly.
pub fn split_lines(source: &str) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    let mut rest = source;
    loop {
        match rest.find('\n') {
            Some(idx) => {
                let raw = &rest[..idx];
                let (content, terminator) = match raw.strip_suffix('\r') {
                    Some(without_cr) => (without_cr, "\r\n"),
                    None => (raw, "\n"),
                };
                lines.push(Line {
                    content,
                    terminator,
                });
                rest = &rest[idx + 1..];
                if rest.is_empty() {
                    break;
                }
            }
            None => {
                lines.push(Line {
                    content: rest,
                    terminator: "",
                });
                break;
            }
        }
    }
    lines
}
