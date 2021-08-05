#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;

use serde::de::DeserializeOwned;
use std::io::Read;
use rocket::Request;
use rocket::data::{Outcome, FromDataSimple};
use rocket::http::{Status, ContentType};
use sha2::Sha256;
use hmac::{Hmac, Mac, NewMac};
use serde_json as json;
use hex;

struct GithubEvent<T> where T: DeserializeOwned {
    name: String,
    payload: T,
}

type HmacSha256 = Hmac<Sha256>;
const SECRET_ENV_VAR: &'static str = "GITHUB_WEBHOOK_SECRET";

impl<T> FromDataSimple for GithubEvent<T> where T: DeserializeOwned {
    type Error = String;

    fn from_data(request: &Request, data: rocket::Data) -> Outcome<Self, Self::Error> {
        // First check that this is json
        if request.content_type() != Some(&ContentType::JSON) {
            return Outcome::Failure((Status::UnprocessableEntity, String::from("Expecting JSON")));
        }

        // Extract the tag
        let target_mac: Vec<u8> = if let Some(tgt) = request.headers().get_one("X-Hub-Signature-256") {
            if let Some(tag_hex) = tgt.strip_prefix("sha256=") {
                hex::decode(tag_hex).unwrap() // TODO: handle error
            } else {
                return Outcome::Failure((Status::Unauthorized, String::from("Malformed signature")));
            }
        } else {
            return Outcome::Failure((Status::Unauthorized, String::from("Missing signature")));
        };

        // Extract event name
        let name = if let Some(n) = request.headers().get_one("X-GitHub-Event") {
            n.to_owned()
        } else {
            return Outcome::Failure((Status::Unauthorized, String::from("Missing event name")));
        };

        // Create mac
        let secret = std::env::var(SECRET_ENV_VAR).unwrap();
        let mut mac = if let Ok(mac) = HmacSha256::new_from_slice(secret.as_bytes()) {
            mac
        } else {
            return Outcome::Failure((Status::InternalServerError, String::from("Unknown error")));
        };

        // Read data
        let mut string = String::new();
        data.open().read_to_string(&mut string).unwrap();

        // Verify MAC
        mac.update(string.as_bytes());
        if let Err(_) = mac.verify(target_mac.as_slice()) {
            return Outcome::Failure((Status::Unauthorized, String::from("Invalid signature")));
        }

        // Now deserialize everything
        if let Ok(payload) = json::from_str(&string) {
            Outcome::Success(GithubEvent { payload, name })
        } else {
            Outcome::Failure((Status::InternalServerError, String::from("Invalid data")))
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Ping {
    zen: String
}

#[post("/handler", data="<event>")]
fn handle_ping(event: GithubEvent<Ping>) {
    println!("{}: {}", event.name, event.payload.zen);
}

#[get("/status")]
fn status() -> &'static str {
    "running"
}

fn main() {
    rocket::ignite().mount("/", routes![handle_ping, status]).launch();
}
