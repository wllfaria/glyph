use std::fmt::Display;
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

impl From<DocumentId> for usize {
    fn from(value: DocumentId) -> Self {
        value.0.into()
    }
}

impl DocumentId {
    pub fn new(document: usize) -> Option<DocumentId> {
        Some(DocumentId(NonZeroUsize::new(document)?))
    }

    pub fn next(&self) -> DocumentId {
        // Safety: will always be non-zero and less than usize::max + 1
        DocumentId(unsafe { NonZeroUsize::new_unchecked(self.0.get().saturating_add(1)) })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LineBreak {
    Lf,
}

impl Display for LineBreak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineBreak::Lf => f.write_str("\n"),
        }
    }
}

impl AsRef<str> for LineBreak {
    fn as_ref(&self) -> &str {
        match self {
            LineBreak::Lf => "\n",
        }
    }
}

#[derive(Debug)]
pub struct DocumentMeta {
    path: Option<PathBuf>,
}

impl DocumentMeta {
    pub fn new(path: Option<PathBuf>) -> DocumentMeta {
        DocumentMeta { path }
    }

    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }
}

#[derive(Debug)]
pub struct Document {
    pub id: DocumentId,
    text: Rope,
    language: LanguageId,
    metadata: DocumentMeta,
    pub line_break: LineBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LanguageId {
    Rust,
    Markdown,
    C,
    Cpp,
    Zig,
    Ocaml,
    Plain,
    Lua,
}

impl LanguageId {
    pub fn from_path(path: Option<&PathBuf>) -> LanguageId {
        match path.and_then(|p| p.extension().and_then(|e| e.to_str())) {
            Some("rs") => LanguageId::Rust,
            Some("md") => LanguageId::Markdown,
            Some("lua") => LanguageId::Lua,
            Some("ml") => LanguageId::Ocaml,
            Some("zig") => LanguageId::Zig,
            Some("c") => LanguageId::C,
            Some("cpp") => LanguageId::Cpp,
            Some("cc") => LanguageId::Cpp,
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
            line_break: LineBreak::Lf,
            metadata: DocumentMeta::new(path),
        }
    }

    pub fn text(&self) -> &Rope {
        &self.text
    }

    pub fn text_mut(&mut self) -> &mut Rope {
        &mut self.text
    }

    pub fn language(&self) -> LanguageId {
        self.language
    }

    pub fn metadata(&self) -> &DocumentMeta {
        &self.metadata
    }
}

impl Default for Document {
    fn default() -> Self {
        Document::new(None, Some(LineBreak::Lf))
    }
}
