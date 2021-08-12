use crate::github::GithubEvent;

#[derive(Deserialize, Serialize)]
pub struct Ping {
    pub zen: String
}

impl GithubEvent for Ping {
    fn event_name() -> &'static str {
        "ping"
    }
}
