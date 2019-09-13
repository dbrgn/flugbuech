#![feature(proc_macro_hygiene, decl_macro, never_type)]

#[macro_use] extern crate diesel;

mod auth;
mod data;
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

// Flights

#[derive(Serialize)]
struct FlightWithDetails<'a> {
    flight: models::Flight,
    aircraft: Option<&'a models::Aircraft>,
    launch_at: Option<&'a models::Location>,
    landing_at: Option<&'a models::Location>,
}

#[derive(Serialize)]
struct FlightsContext<'a> {
    user: models::User,
    flights: Vec<FlightWithDetails<'a>>,
}

#[get("/flights")]
fn flights(db: data::Database, user: auth::AuthUser) -> Template {
    let user = user.into_inner();

    // Get all flights
    let flights = data::get_flights_for_user(&db, &user);

    // Get all aircraft for user
    let aircraft_map = data::get_aircraft_for_user(&db, &user)
        .into_iter()
        .map(|aircraft| (aircraft.id, aircraft))
        .collect::<HashMap<i32, models::Aircraft>>();

    // Get all locations used
    let mut location_ids = flights
        .iter()
        .flat_map(|flight| vec![flight.launch_at, flight.landing_at])
        .filter_map(|opt| opt)
        .collect::<Vec<_>>();
    location_ids.sort();
    location_ids.dedup();
    let location_map = data::get_locations_with_ids(&db, &location_ids)
        .into_iter()
        .map(|location| (location.id, location))
        .collect::<HashMap<i32, models::Location>>();

    // Add details to flights
    let flights_with_details = flights
        .into_iter()
        .map(|flight| {
            // Look up aircraft
            let aircraft = flight.aircraft_id.and_then(|id| aircraft_map.get(&id));

            // Look up launch and landing
            let launch_at = flight.launch_at.and_then(|id| location_map.get(&id));
            let landing_at = flight.landing_at.and_then(|id| location_map.get(&id));

            FlightWithDetails {
                flight,
                aircraft,
                launch_at,
                landing_at,
            }
        })
        .collect::<Vec<_>>();

    // Render template
    let context = FlightsContext {
        user,
        flights: flights_with_details,
    };
    Template::render("flights", &context)
}

#[get("/flights", rank = 2)]
fn flights_nologin() -> Redirect {
    Redirect::to("/auth/login")
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
        .attach(Template::fairing())
        .register(catchers![service_not_available])
        // Main routes
        .mount(
            "/",
            routes![
                index,
                flights,
                flights_nologin,
                locations::list,
                locations::list_nologin,
                locations::add_form,
                locations::add_form_nologin,
                locations::add,
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
