use serde::de::DeserializeOwned;
use std::io::Read;
use rocket::Request;
use rocket::data::{Outcome, FromDataSimple};
use rocket::http::{Status, ContentType};
use sha2::Sha256;
use hmac::{Hmac, Mac, NewMac};
use serde_json as json;
use hex;
use octocrab::models::User;

// TODO: this could be made easier using macro_rules
#[derive(Serialize, Deserialize)]
pub struct EventPayloadCommon {
    pub sender: User,
    pub action: String
}

pub struct GithubEvent<T> where T: DeserializeOwned {
    pub name: String,
    pub payload: T,
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
        match json::from_str::<T>(&string) {
            Ok(payload) => Outcome::Success(GithubEvent { payload, name }),
            Err(e) => {
                eprintln!("Invalid data: {:?}", e);
                return Outcome::Failure((Status::InternalServerError, String::from("Invalid data")))
            }
        }
    }
}

