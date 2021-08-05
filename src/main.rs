#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;

mod github;
mod handlers;

use github::GithubEvent;
use handlers::ping::Ping;
use handlers::issues::IssuesEvent;
use handlers::issue_comment::IssueCommentEvent;

#[post("/handler", data="<event>")]
fn handle_issue_comment(event: GithubEvent<IssueCommentEvent>) {
    if let Some(body) = event.payload.comment.body {
        println!("{}: {} commented \"{}\" on issue {}",
                 event.name,
                 event.payload.common.sender.login,
                 body,
                 event.payload.issue.number);
    }
}

#[post("/handler", data="<event>", rank=2)]
fn handle_issues(event: GithubEvent<IssuesEvent>) {
    println!("{}: issue {} {} by {} ",
             event.name,
             event.payload.issue.number,
             event.payload.common.action,
             event.payload.common.sender.login);
}

#[post("/handler", data="<event>", rank=3)]
fn handle_ping(event: GithubEvent<Ping>) {
    println!("{}: {}", event.name, event.payload.zen);
}

#[get("/status")]
fn status() -> &'static str {
    "running"
}

fn main() {
    rocket::ignite().mount("/", routes![handle_issue_comment, handle_issues, handle_ping, status]).launch();
}
