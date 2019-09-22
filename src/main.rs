#![feature(proc_macro_hygiene, decl_macro, never_type)]

#[macro_use] extern crate diesel;

mod auth;
mod data;
mod filters;
mod flights;
mod locations;
mod models;
mod process_igc;
mod schema;
mod submit;
#[cfg(test)] mod test_utils;

use rocket::request::Request;
use rocket::response::Redirect;
use rocket::{catch, catchers, get, routes};
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use serde::Serialize;
use std::collections::HashMap;

pub(crate) const MAX_UPLOAD_BYTES: u64 = 50 * 1024 * 1024;

// Index

#[derive(Serialize)]
struct IndexContext {
    user: Option<models::User>,
    users_with_aircraft: Vec<(models::User, Vec<models::Aircraft>)>,
}

#[get("/")]
fn index(db: data::Database, user: Option<auth::AuthUser>) -> Template {
    let mut usermap: HashMap<i32, (models::User, Vec<models::Aircraft>)> = HashMap::new();
    for user in data::get_users(&db) {
        usermap.insert(user.id, (user, vec![]));
    }

    for aircraft in data::get_aircraft(&db) {
        usermap.get_mut(&aircraft.user_id).unwrap().1.push(aircraft)
    }

    let context = IndexContext {
        user: user.map(|u| u.into_inner()),
        users_with_aircraft: usermap.values().cloned().collect::<Vec<_>>(),
    };
    Template::render("index", &context)
}

// Profile

#[derive(Serialize)]
struct ProfileContext {
    user: models::User,
    aircraft_list: Vec<models::Aircraft>,
}

#[get("/profile")]
fn profile(user: auth::AuthUser, db: data::Database) -> Template {
    let user = user.into_inner();
    let aircraft_list = data::get_aircraft_for_user(&db, &user);
    let context = ProfileContext { user, aircraft_list };
    Template::render("profile", context)
}

#[get("/profile", rank = 2)]
fn profile_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

// Handle missing DB

#[catch(503)]
fn service_not_available(_req: &Request) -> &'static str {
    "Service is not available. (Is the database up?)"
}

// Main

fn main() {
    rocket::ignite()
        .attach(data::Database::fairing())
        .attach(Template::custom(|engines| {
            engines.tera.register_filter("duration", filters::duration);
        }))
        .register(catchers![service_not_available])
        // Main routes
        .mount(
            "/",
            routes![
                index,
                flights::flights,
                flights::flights_nologin,
                flights::flight,
                locations::list,
                locations::list_nologin,
                locations::add_form,
                locations::add_form_nologin,
                locations::add,
                locations::edit_form,
                locations::edit,
                process_igc::process_igc,
                submit::submit_form,
                submit::submit_form_nologin,
                submit::submit,
            ],
        )
        // Profile
        .mount("/", routes![profile, profile_nologin,])
        // Auth routes
        .mount("/", auth::get_routes())
        // Static files
        .mount(
            "/static",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .launch();
}
