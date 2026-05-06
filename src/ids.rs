use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type UserId = Uuid;
pub type OrgId = Uuid;
pub type RepoId = Uuid;
pub type SessionId = Uuid;
pub type CommitId = String;
pub type BranchName = String;
pub type ChunkHash = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Slug(pub String);

impl Slug {
    pub fn new(input: impl AsRef<str>) -> Self {
        Self(slugify(input.as_ref()))
    }
}

pub fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;

    for ch in input.trim().chars().flat_map(|c| c.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }

    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        "untitled".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::slugify;

    #[test]
    fn slugify_normalizes_input() {
        assert_eq!(slugify("  Clear Mesh V2!! "), "clear-mesh-v2");
        assert_eq!(slugify("Models___2026"), "models-2026");
        assert_eq!(slugify("###"), "untitled");
    }
}
