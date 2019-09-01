#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;

mod auth;
mod data;
mod models;
mod schema;
#[cfg(test)] mod test_utils;

use std::collections::HashMap;
use std::io::{self, Read, BufRead, BufReader};

use igc::records::{Record, HRecord};
use igc::util::{Time, RawPosition};
use rocket::{get, post, routes, catchers, catch};
use rocket::data::Data;
use rocket::request::Request;
use rocket::response::Redirect;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use serde::Serialize;


const MAX_UPLOAD_BYTES: u64 = 50 * 1024 * 1024;


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
        users_with_aircraft: usermap.values().cloned().collect::<Vec<_>>()
    };
    Template::render("index", &context)
}


// Profile

#[derive(Serialize)]
struct ProfileContext {
    user: models::User,
    aircraft: Vec<models::Aircraft>,
}

#[get("/profile")]
fn profile(user: auth::AuthUser, db: data::Database) -> Template {
    let user = user.into_inner();
    let aircraft = data::get_aircraft_for_user(&db, &user);
    let context = ProfileContext { user, aircraft };
    Template::render("profile", context)
}

#[get("/profile", rank = 2)]
fn profile_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

// Submit

#[derive(Serialize)]
struct SubmitContext {
    user: models::User,
}

#[get("/submit")]
fn submit(user: auth::AuthUser) -> Template {
    let context = SubmitContext {
        user: user.into_inner(),
    };
    Template::render("submit", context)
}

#[get("/submit", rank = 2)]
fn submit_nologin() -> Redirect {
    Redirect::to("/auth/login")
}


// Process IGC

#[derive(Debug, PartialEq)]
struct LaunchLandingInfo {
    pos: RawPosition,
    alt: i16,
    time: Time,
}

impl LaunchLandingInfo {
    fn seconds_since_midnight(&self) -> u32 {
        u32::from(self.time.hours) * 24 * 60 +
        u32::from(self.time.minutes) * 60 +
        u32::from(self.time.seconds)
    }
}

#[derive(Default, Debug, PartialEq)]
struct FlightInfo {
    /// Name of the pilot, as configured in the flight instrument.
    pilot: Option<String>,
    /// Name of the glider, as configured in the flight instrument.
    glidertype: Option<String>,
    /// Name of the launch site, as configured in the flight instrument.
    site: Option<String>,
    /// Lauch infos.
    launch: Option<LaunchLandingInfo>,
    /// Landing infos.
    landing: Option<LaunchLandingInfo>,
}

impl FlightInfo {
    fn duration(&self) -> Option<u32> {
        if let (Some(launch), Some(landing)) = (&self.launch, &self.landing) {
            let launch_seconds = launch.seconds_since_midnight();
            let landing_seconds = landing.seconds_since_midnight();
            if landing_seconds > launch_seconds {
                Some(landing_seconds - launch_seconds)
            } else {
                Some(86400 - launch_seconds + landing_seconds)
            }
        } else {
            None
        }
    }
}

#[post("/submit/process_igc", format = "application/octet-stream", data = "<data>")]
fn process_igc(data: Data) -> io::Result<String> {
    let reader = data.open().take(MAX_UPLOAD_BYTES);
    let buf_reader = BufReader::new(reader);
    let lines = buf_reader.lines().collect::<Result<Vec<String>, io::Error>>()?;
    let mut info = FlightInfo::default();
    for line in &lines {
        match Record::parse_line(&line) {
            Ok(Record::H(h @ HRecord { mnemonic: "PLT", .. })) => {
                info.pilot = Some(h.data.trim().into());
            }
            Ok(Record::H(h @ HRecord { mnemonic: "GTY", .. })) => {
                info.glidertype = Some(h.data.trim().into());
            }
            Ok(Record::H(h @ HRecord { mnemonic: "SIT", .. })) => {
                info.site = Some(h.data.trim().into());
            }
            Ok(Record::B(b)) => {
                println!("{}: {} (GPS) / {} (Baro)", b.timestamp, b.gps_alt, b.pressure_alt);
                if info.launch.is_none() {
                    info.launch = Some(LaunchLandingInfo {
                        pos: b.pos,
                        alt: b.gps_alt,
                        time: b.timestamp,
                    });
                } else {
                    info.landing = Some(LaunchLandingInfo {
                        pos: b.pos,
                        alt: b.gps_alt,
                        time: b.timestamp,
                    });
                }
            }
            Ok(_rec) => {},
            Err(e) => return Ok(format!("Error parsing lines: {:?}", e)),
        }
    }
    println!("Info: {:#?}", info);
    println!("Flight duration: {:?} seconds", info.duration());
    Ok("OK".into())
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
        .mount("/", routes![index, submit, submit_nologin, process_igc])
        // Profile
        .mount("/", routes![profile, profile_nologin])
        // Auth routes
        .mount("/", auth::get_routes())
        // Static files
        .mount("/static", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")))
        .launch();
}
