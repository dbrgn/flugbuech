#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;

mod data;
mod models;
mod schema;

use std::collections::HashMap;

use rocket::{get, routes, catchers, catch};
use rocket::request::Request;
use rocket_contrib::database;
use rocket_contrib::templates::Template;

#[database("flugbuech")]
struct Database(diesel::PgConnection);

#[get("/")]
fn index(db: Database) -> Template {
    let mut map = HashMap::new();
    map.insert("user_count", data::get_users(&db).len());
    map.insert("aircraft_count", data::get_aircraft(&db).len());
    Template::render("index", &map)
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
