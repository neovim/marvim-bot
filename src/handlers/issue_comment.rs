use crate::github::{EventPayloadCommon, GithubEvent};
use octocrab::models::issues::{Issue, Comment};

#[derive(Serialize, Deserialize)]
pub struct IssueCommentEvent {
    #[serde(flatten)]
    pub common: EventPayloadCommon,
    pub comment: Comment,
    pub issue: Issue
}

impl GithubEvent for IssueCommentEvent {
    fn event_name() -> &'static str {
        "issue_comment"
    }
}
