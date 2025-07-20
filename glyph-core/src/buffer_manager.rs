use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::error::Result;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum BufferKind {
    Scratch,
    Regular,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum FileStatus {
    New,
    Existing,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Buffer {
    pub id: BufferId,
    is_dirty: bool,
    content: String,
    file_status: FileStatus,
    buffer_kind: BufferKind,
    path: Option<PathBuf>,
    absolute_path: Option<PathBuf>,
}

impl Buffer {
    pub fn new(
        id: BufferId,
        content: String,
        path: Option<PathBuf>,
        absolute_path: Option<PathBuf>,
        file_status: FileStatus,
        buffer_kind: BufferKind,
    ) -> Self {
        Self {
            id,
            path,
            content,
            file_status,
            buffer_kind,
            absolute_path,
            is_dirty: false,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BufferId(u64);

impl BufferId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<u64> for BufferId {
    fn from(id: u64) -> Self {
        Self::new(id)
    }
}

impl From<u32> for BufferId {
    fn from(id: u32) -> Self {
        Self::new(id as u64)
    }
}

impl From<u16> for BufferId {
    fn from(id: u16) -> Self {
        Self::new(id as u64)
    }
}

impl From<u8> for BufferId {
    fn from(id: u8) -> Self {
        Self::new(id as u64)
    }
}

#[derive(Debug)]
pub struct BufferManager {
    next_buffer_id: BufferId,
    pub(crate) buffers: BTreeMap<BufferId, Buffer>,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            buffers: BTreeMap::new(),
            next_buffer_id: BufferId(0),
        }
    }

    pub fn load_buffer(&mut self, path_str: &str) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let path = PathBuf::from(path_str);
        let file_status = if !path.exists() { FileStatus::New } else { FileStatus::Existing };
        let absolute_path = cwd.join(&path);
        let content = std::fs::read_to_string(&path).unwrap_or_default();

        let id = self.next_buffer_id;
        self.next_buffer_id = self.next_buffer_id.next();

        let buffer = Buffer::new(
            id,
            content,
            Some(path),
            Some(absolute_path),
            file_status,
            BufferKind::Regular,
        );
        self.buffers.insert(id, buffer);

        Ok(())
    }

    pub fn load_startup_buffer(&mut self) -> Result<()> {
        let content = r#"
        Welcome to Glyph!
        "#
        .to_string();

        let id = BufferId::new(0);
        let buffer = Buffer::new(
            id,
            content,
            None,
            None,
            FileStatus::New,
            BufferKind::Scratch,
        );
        self.buffers.insert(id, buffer);

        Ok(())
    }

    pub fn get(&self, id: BufferId) -> Option<&Buffer> {
        self.buffers.get(&id)
    }
}
