use crate::github::EventPayloadCommon;
use octocrab::models::{User, issues::Issue};

#[derive(Serialize, Deserialize)]
pub struct IssuesEvent {
    #[serde(flatten)]
    pub common: EventPayloadCommon,

    pub assignee: Option<User>,
    pub issue: Issue
}
