#![feature(proc_macro_hygiene, decl_macro, never_type)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;

mod auth;
mod base64;
mod data;
mod filters;
mod flights;
mod locations;
mod models;
mod optionresult;
mod process_igc;
mod schema;
#[cfg(test)] mod test_utils;

use clap::{App, Arg};
use dotenv;
use rocket::request::Request;
use rocket::response::Redirect;
use rocket::{catch, catchers, get, routes};
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use serde::Serialize;

pub(crate) const MAX_UPLOAD_BYTES: u64 = 50 * 1024 * 1024;
pub(crate) const NAME: &str = "flugbuech";
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const DESCRIPTION: &str = "Paragliding flight book.";

// Index

#[derive(Serialize)]
struct IndexContext {
    user: Option<models::User>,
    user_count: i64,
    aircraft_count: i64,
    flight_count: i64,
}

#[get("/")]
fn index(db: data::Database, user: Option<auth::AuthUser>) -> Template {
    let context = IndexContext {
        user: user.map(|u| u.into_inner()),
        user_count: data::get_user_count(&db),
        aircraft_count: data::get_aircraft_count(&db),
        flight_count: data::get_flight_count(&db),
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
    // Load env
    let _ = dotenv::dotenv();

    // Parse args
    let args = App::new(NAME)
        .about(DESCRIPTION)
        .version(VERSION)
        .arg(
            Arg::with_name("migrate")
                .long("migrate")
                .help("Run database migrations before starting"),
        )
        .get_matches();

    // Decide whether migrations should be run
    let migrate = args.is_present("migrate");
    if migrate {
        println!("Running database migrations...");
        data::run_migrations().unwrap();
    }

    rocket::ignite()
        .attach(data::Database::fairing())
        .attach(Template::custom(|engines| {
            engines.tera.register_filter("duration", filters::duration);
            engines
                .tera
                .register_filter("xcontest_icon", filters::xcontest_icon);
        }))
        .register(catchers![service_not_available])
        // Main routes
        .mount(
            "/",
            routes![
                index,
                flights::list,
                flights::list_nologin,
                flights::flight,
                flights::edit_form,
                flights::edit,
                flights::igc_download,
                flights::submit_form,
                flights::submit_form_nologin,
                flights::submit,
                locations::list,
                locations::list_nologin,
                locations::add_form,
                locations::add_form_nologin,
                locations::add,
                locations::edit_form,
                locations::edit,
                process_igc::process_igc,
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
