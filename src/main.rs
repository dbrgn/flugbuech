#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;

mod data;
mod models;
mod schema;

use rocket::{get, routes, catchers, catch};
use rocket::request::Request;
use rocket_contrib::database;
use rocket_contrib::templates::Template;
use serde::Serialize;

#[database("flugbuech")]
struct Database(diesel::PgConnection);

#[derive(Serialize)]
struct IndexData {
    users: Vec<models::User>,
    aircraft: Vec<models::Aircraft>,
}

#[get("/")]
fn index(db: Database) -> Template {
    let context = IndexData {
        users: data::get_users(&db),
        aircraft: data::get_aircraft(&db),
    };
    Template::render("index", &context)
}

#[catch(503)]
fn service_not_available(_req: &Request) -> &'static str {
    "Service is not available. (Is the database up?)"
}

fn main() {
    rocket::ignite()
        .attach(Database::fairing())
        .attach(Template::fairing())
        .register(catchers![service_not_available])
        .mount("/", routes![index])
        .launch();
}
