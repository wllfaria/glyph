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
    text: Rope,
    language: LanguageId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LanguageId {
    Rust,
    Markdown,
    Plain,
}

impl LanguageId {
    pub fn from_path(path: Option<&PathBuf>) -> LanguageId {
        match path.and_then(|p| p.extension().and_then(|e| e.to_str())) {
            Some("rs") => LanguageId::Rust,
            Some("md") => LanguageId::Markdown,
            Some(_) => LanguageId::Plain,
            None => LanguageId::Plain,
        }
    }
}

impl Document {
    pub fn new<S>(path: Option<PathBuf>, text: Option<S>) -> Document
    where
        S: AsRef<str>,
    {
        let language = LanguageId::from_path(path.as_ref());

        Document {
            id: DocumentId::default(),
            language,
            text: text.map(|t| Rope::from_str(t.as_ref())).unwrap_or_default(),
        }
    }

    pub fn text(&self) -> &Rope {
        &self.text
    }

    pub fn language(&self) -> LanguageId {
        self.language
    }
}

impl Default for Document {
    fn default() -> Self {
        Document::new(None, Some(LineEnding::Lf))
    }
}
