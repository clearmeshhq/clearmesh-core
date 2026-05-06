use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrgRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoRole {
    Admin,
    Maintainer,
    Writer,
    Reader,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoAction {
    Read,
    Write,
    Manage,
    Delete,
}

pub fn role_allows_action(role: RepoRole, action: RepoAction) -> bool {
    match role {
        RepoRole::Admin => true,
        RepoRole::Maintainer => matches!(
            action,
            RepoAction::Read | RepoAction::Write | RepoAction::Manage
        ),
        RepoRole::Writer => matches!(action, RepoAction::Read | RepoAction::Write),
        RepoRole::Reader => matches!(action, RepoAction::Read),
    }
}

#[cfg(test)]
mod tests {
    use super::{role_allows_action, RepoAction, RepoRole};

    #[test]
    fn permissions_match_roles() {
        assert!(role_allows_action(RepoRole::Admin, RepoAction::Delete));
        assert!(role_allows_action(RepoRole::Writer, RepoAction::Write));
        assert!(!role_allows_action(RepoRole::Writer, RepoAction::Manage));
        assert!(!role_allows_action(RepoRole::Reader, RepoAction::Write));
    }
}
