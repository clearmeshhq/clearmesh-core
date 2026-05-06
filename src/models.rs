use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkRef {
    pub hash: String,
    pub size: u64,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub mode: u32,
    pub chunks: Vec<ChunkRef>,
    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,
}

fn default_chunk_size() -> usize {
    1024 * 1024 // 1 MiB for backward compatibility
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub author_email: String,
    pub created_at: DateTime<Utc>,
    pub parent_ids: Vec<String>,
    pub files: Vec<FileEntry>,
}

impl Commit {
    pub fn new(
        message: impl Into<String>,
        author_email: impl Into<String>,
        created_at: DateTime<Utc>,
        parent_ids: Vec<String>,
        files: Vec<FileEntry>,
    ) -> Self {
        Self {
            id: String::new(),
            message: message.into(),
            author_email: author_email.into(),
            created_at,
            parent_ids,
            files,
        }
    }
}
