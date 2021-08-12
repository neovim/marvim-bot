use serde::de::DeserializeOwned;
use rocket::Request;
use rocket::tokio::io::AsyncReadExt;
use rocket::data::{Outcome, FromData, ToByteUnit};
use rocket::http::{Status, ContentType};
use sha2::Sha256;
use hmac::{Hmac, Mac, NewMac};
use serde_json as json;
use hex;
use octocrab::models::User;
use crate::CONFIG;

// TODO: this could be made easier using macro_rules
#[derive(Serialize, Deserialize)]
pub struct EventPayloadCommon {
    pub sender: User,
    pub action: String
}

pub struct SignedPayload<T> where T: DeserializeOwned + GithubEvent {
    pub payload: T,
}

pub trait GithubEvent {
    fn event_name() -> &'static str;
}

type HmacSha256 = Hmac<Sha256>;

#[rocket::async_trait]
impl<'r, T> FromData<'r> for SignedPayload<T> where T: DeserializeOwned + GithubEvent {
    type Error = String;

    async fn from_data(request: &'r Request<'_>, data: rocket::Data<'r>) -> Outcome<'r, Self> {
        // First check that this is json
        if request.content_type() != Some(&ContentType::JSON) {
            return Outcome::Failure((Status::UnprocessableEntity, String::from("Expecting JSON")));
        }

        // Extract event name
        if let Some(n) = request.headers().get_one("X-GitHub-Event") {
            if <T as GithubEvent>::event_name() != n {
                return Outcome::Forward(data);
            }
        } else {
            return Outcome::Failure((Status::Unauthorized, String::from("Missing event name")));
        };

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

        // Create mac
        let mut mac = if let Ok(mac) = HmacSha256::new_from_slice(CONFIG.webhook_secret.as_bytes()) {
            mac
        } else {
            return Outcome::Failure((Status::InternalServerError, String::from("Unknown error")));
        };

        // Read data
        let string: String = match data.open(512.megabytes()).into_string().await {
            Ok(str) => {
                if !str.is_complete() {
                    return Outcome::Failure((Status::InsufficientStorage, String::from("Request too big")));
                }
                str.into_inner()
            },
            Err(_) => {
                return Outcome::Failure((Status::UnprocessableEntity, String::from("Impossible to get data")));
            }
        };

        // Verify MAC
        mac.update(string.as_bytes());
        if let Err(_) = mac.verify(target_mac.as_slice()) {
            return Outcome::Failure((Status::Unauthorized, String::from("Invalid signature")));
        }

        // Now deserialize everything
        match json::from_str::<T>(&string) {
            Ok(payload) => Outcome::Success(SignedPayload { payload }),
            Err(_) => {
                return Outcome::Failure((Status::InternalServerError, String::from("Invalid data")))
            }
        }
    }
}
