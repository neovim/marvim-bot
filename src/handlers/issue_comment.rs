use crate::github::EventPayloadCommon;
use octocrab::models::issues::{Issue, Comment};

#[derive(Serialize, Deserialize)]
pub struct IssueCommentEvent {
    #[serde(flatten)]
    pub common: EventPayloadCommon,
    pub comment: Comment,
    pub issue: Issue
}
