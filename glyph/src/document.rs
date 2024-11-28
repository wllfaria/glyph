use std::num::NonZeroUsize;
use std::path::PathBuf;

use ropey::Rope;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct DocumentId(NonZeroUsize);

impl Default for DocumentId {
    fn default() -> DocumentId {
        // Safety: 1 is non-zero
        DocumentId(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}

impl DocumentId {
    pub fn next(&self) -> DocumentId {
        // Safety: will always be non-zero and less than usize::max + 1
        DocumentId(unsafe { NonZeroUsize::new_unchecked(self.0.get().saturating_add(1)) })
    }
}

#[derive(Debug)]
pub enum LineEnding {
    Lf,
}

impl AsRef<str> for LineEnding {
    fn as_ref(&self) -> &str {
        match self {
            LineEnding::Lf => "\n",
        }
    }
}

#[derive(Debug)]
pub struct Document {
    pub id: DocumentId,
    path: Option<PathBuf>,
    text: Rope,
}

impl Document {
    pub fn new<S>(path: Option<PathBuf>, text: Option<S>) -> Document
    where
        S: AsRef<str>,
    {
        Document {
            id: DocumentId::default(),
            path,
            text: text.map(|t| Rope::from_str(t.as_ref())).unwrap_or_default(),
        }
    }

    pub fn text(&self) -> &Rope {
        &self.text
    }
}

impl Default for Document {
    fn default() -> Self {
        Document::new(None, Some(LineEnding::Lf))
    }
}
