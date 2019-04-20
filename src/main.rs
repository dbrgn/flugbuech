#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;

mod data;
mod models;
mod schema;

use rocket::{get, routes, catchers, catch};
use rocket::request::Request;
use rocket_contrib::database;

#[database("flugbuech")]
struct Database(diesel::PgConnection);

#[get("/")]
fn index(db: Database) -> String {
    let user_count = data::get_users(&db).len();
    let aircraft_count= data::get_aircraft(&db).len();
    format!(
        "Hello, sky!\n\nDatabase contains {} users with {} aircraft.",
        user_count,
        aircraft_count,
    )
}

#[catch(503)]
fn service_not_available(_req: &Request) -> &'static str {
    "Service is not available. (Is the database up?)"
}

fn main() {
    rocket::ignite()
        .attach(Database::fairing())
        .register(catchers![service_not_available])
        .mount("/", routes![index])
        .launch();
}
