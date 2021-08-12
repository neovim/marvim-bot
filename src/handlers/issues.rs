use crate::github::{EventPayloadCommon, GithubEvent};
use octocrab::models::{User, issues::Issue};

#[derive(Serialize, Deserialize)]
pub struct IssuesEvent {
    #[serde(flatten)]
    pub common: EventPayloadCommon,

    pub assignee: Option<User>,
    pub issue: Issue
}

impl GithubEvent for IssuesEvent {
    fn event_name() -> &'static str {
        "issues"
    }
}
