#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate json;

mod github;
mod handlers;

use std::fs::File;
use std::io::BufReader;
use serde_yaml::from_reader as yaml_de;
use github::SignedPayload;
use handlers::ping::Ping;
use handlers::issues::IssuesEvent;
use handlers::issue_comment::IssueCommentEvent;
use octocrab;

#[derive(Deserialize)]
struct Config {
    webhook_secret: String,
    access_token: String
}

#[post("/handler", data="<event>")]
async fn handle_issue_comment(event: SignedPayload<IssueCommentEvent>) -> () {
    if let Some(ref body) = event.payload.comment.body {
        println!("{} commented \"{}\" on issue {}",
                 event.payload.common.sender.login,
                 body,
                 event.payload.issue.number);
        for line in body.lines() {
            if line.starts_with("ping") {
                println!("LOOKS LIKE A PING");
                let issue_comment_url = event.payload.issue.comments_url.clone();
                if let Ok(resp) =
                    octocrab::instance()._post(issue_comment_url, Some("{\"body\":\"pong\"}")).await {
                    println!("{:?}", resp);
                };
                println!("COMMENTED");
            } else {
                println!("MEH");
            }
        }
    }
}

#[post("/handler", data="<event>", rank=2)]
fn handle_issues(event: SignedPayload<IssuesEvent>) {
    println!("issue {} {} by {} ",
             event.payload.issue.number,
             event.payload.common.action,
             event.payload.common.sender.login);
}

#[post("/handler", data="<event>", rank=3)]
fn handle_ping(event: SignedPayload<Ping>) {
    println!("{}", event.payload.zen);
}

#[get("/status")]
fn status() -> &'static str {
    "running"
}

lazy_static! {
    static ref CONFIG: Config = {
        let file = BufReader::new(File::open("config.yml").expect("Unable to find config file"));
        yaml_de(file).expect("Invalid config file format")
    };
}

#[launch]
async fn rocket() -> _ {
    let token_copy = String::from(&CONFIG.access_token);
    let octocrab_builder = octocrab::Octocrab::builder().personal_token(token_copy);
    octocrab::initialise(octocrab_builder).expect("Unable to connect to github");
    rocket::build()
        .mount("/",
               routes![handle_issue_comment, handle_issues, handle_ping, status])
}
