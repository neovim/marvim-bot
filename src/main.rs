#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;

use rocket_contrib::json::Json;

#[derive(Serialize, Deserialize)]
struct Ping {
    zen: String,
    hook_id: i64,
}

#[post("/handler", format="json", data="<ping>")]
fn handle(ping: Json<Ping>) {
    println!("{}", ping.zen);
}

#[get("/status")]
fn status() -> &'static str {
    "running"
}

fn main() {
    rocket::ignite().mount("/", routes![handle, status]).launch();
}
