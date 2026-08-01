/// One Markdown file, held as its original source text.
///
/// Rules that need parsed structure derive it from `source`; the source itself
/// is always the source of truth, so fixes can be applied to the original bytes.
#[derive(Debug, Clone)]
pub struct Document {
    pub source: String,
}

impl Document {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }
}
