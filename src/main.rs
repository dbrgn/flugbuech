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
use serde::Serialize;

#[database("flugbuech")]
struct Database(diesel::PgConnection);

#[derive(Serialize)]
struct IndexData {
    users_with_aircraft: Vec<(models::User, Vec<models::Aircraft>)>,
}

#[get("/")]
fn index(db: Database) -> Template {
    let mut usermap: HashMap<i32, (models::User, Vec<models::Aircraft>)> = HashMap::new();
    for user in data::get_users(&db) {
        usermap.insert(user.id, (user, vec![]));
    }

    for aircraft in data::get_aircraft(&db) {
        usermap.get_mut(&aircraft.user_id).unwrap().1.push(aircraft)
    }

    let context = IndexData {
        users_with_aircraft: usermap.values().cloned().collect::<Vec<_>>()
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
