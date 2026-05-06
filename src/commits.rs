use crate::crypto::hash_bytes;
use crate::models::{Commit, FileEntry};
use serde::Serialize;

#[derive(Serialize)]
struct CommitForHash<'a> {
    message: &'a str,
    author_email: &'a str,
    created_at: String,
    parent_ids: &'a [String],
    files: &'a [FileEntry],
}

pub fn finalize_commit(mut commit: Commit) -> Commit {
    commit.files.sort_by(|a, b| a.path.cmp(&b.path));
    for file in &mut commit.files {
        file.chunks.sort_by_key(|chunk| chunk.index);
    }
    commit.parent_ids.sort();

    let stable = CommitForHash {
        message: &commit.message,
        author_email: &commit.author_email,
        created_at: commit
            .created_at
            .to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        parent_ids: &commit.parent_ids,
        files: &commit.files,
    };
    let bytes = serde_json::to_vec(&stable).expect("commit serialization should not fail");
    commit.id = hash_bytes(&bytes);
    commit
}

#[cfg(test)]
mod tests {
    use super::finalize_commit;
    use crate::models::{ChunkRef, Commit, FileEntry};
    use chrono::{TimeZone, Utc};

    fn sample_commit() -> Commit {
        Commit::new(
            "initial",
            "dev@example.com",
            Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap(),
            vec!["parent-b".into(), "parent-a".into()],
            vec![FileEntry {
                path: "data.txt".into(),
                size: 5,
                mode: 0o100644,
                chunks: vec![ChunkRef {
                    hash: "abc".into(),
                    size: 5,
                    index: 0,
                }],
                chunk_size: 1024 * 1024,
            }],
        )
    }

    #[test]
    fn commit_id_is_deterministic() {
        let a = finalize_commit(sample_commit());
        let b = finalize_commit(sample_commit());
        assert_eq!(a.id, b.id);
        assert_eq!(a.id.len(), 64);
    }
}
