//! Flight views.

use std::collections::HashMap;
use std::io::Cursor;

use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use chrono::{DateTime, Utc};
use rocket::http::hyper::header::{Charset, ContentDisposition, DispositionParam, DispositionType};
use rocket::http::{ContentType, Status};
use rocket::request::{Form, FromForm};
use rocket::response::{Redirect, Response};
use rocket::{get, post};
use rocket_contrib::templates::Template;
use serde::Serialize;

use crate::base64::Base64Data;
use crate::models::{Aircraft, Flight, Location, NewFlight, User};
use crate::optionresult::OptionResult;
use crate::{auth, data};

#[derive(Serialize)]
struct FlightWithDetails<'a> {
    flight: Flight,
    aircraft: Option<&'a Aircraft>,
    launch_at: Option<&'a Location>,
    landing_at: Option<&'a Location>,
    duration_seconds: Option<u64>,
}

#[derive(Serialize)]
struct FlightsContext<'a> {
    user: User,
    flights: Vec<FlightWithDetails<'a>>,
}

#[get("/flights")]
pub(crate) fn flights(db: data::Database, user: auth::AuthUser) -> Template {
    let user = user.into_inner();

    // Get all flights
    let flights = data::get_flights_for_user(&db, &user);

    // Get all aircraft for user
    let aircraft_map = data::get_aircraft_for_user(&db, &user)
        .into_iter()
        .map(|aircraft| (aircraft.id, aircraft))
        .collect::<HashMap<i32, Aircraft>>();

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
        .collect::<HashMap<i32, Location>>();

    // Add details to flights
    let flights_with_details = flights
        .into_iter()
        .map(|flight| {
            // Look up aircraft
            let aircraft = flight.aircraft_id.and_then(|id| aircraft_map.get(&id));

            // Look up launch and landing
            let launch_at = flight.launch_at.and_then(|id| location_map.get(&id));
            let landing_at = flight.landing_at.and_then(|id| location_map.get(&id));

            // Calculate duration
            let duration_seconds = match (flight.launch_time, flight.landing_time) {
                (Some(launch), Some(landing)) => {
                    let duration = (landing - launch).num_seconds();
                    if duration < 0 {
                        None
                    } else {
                        Some(duration as u64)
                    }
                },
                _ => None,
            };
            FlightWithDetails {
                flight,
                aircraft,
                launch_at,
                landing_at,
                duration_seconds,
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
pub(crate) fn flights_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

#[derive(Serialize)]
struct FlightContext<'a> {
    user: User,
    flight: Flight,
    aircraft: Option<&'a Aircraft>,
    launch_at: Option<&'a Location>,
    landing_at: Option<&'a Location>,
    duration_seconds: Option<u64>,
}

#[get("/flights/<id>")]
pub(crate) fn flight(id: i32, db: data::Database, user: auth::AuthUser) -> Result<Template, Status> {
    let user = user.into_inner();

    // Get flight
    let flight = match data::get_flight_with_id(&db, id) {
        Some(flight) => flight,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if flight.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Resolve foreign keys
    let launch_at = flight
        .launch_at
        .and_then(|id| data::get_location_with_id(&db, id));
    let landing_at = flight
        .landing_at
        .and_then(|id| data::get_location_with_id(&db, id));
    let aircraft = flight
        .aircraft_id
        .and_then(|id| data::get_aircraft_with_id(&db, id));

    // Calculate duration
    let duration_seconds = match (flight.launch_time, flight.landing_time) {
        (Some(launch), Some(landing)) => {
            let duration = (landing - launch).num_seconds();
            if duration < 0 {
                None
            } else {
                Some(duration as u64)
            }
        },
        _ => None,
    };

    // Render template
    let context = FlightContext {
        user,
        flight,
        aircraft: aircraft.as_ref(),
        launch_at: launch_at.as_ref(),
        landing_at: landing_at.as_ref(),
        duration_seconds,
    };
    Ok(Template::render("flight", &context))
}

#[get("/flights/<id>/igc")]
pub(crate) fn igc_download(
    user: auth::AuthUser,
    db: data::Database,
    id: i32,
) -> Result<Response<'static>, Status> {
    let user = user.into_inner();

    // Get flight
    let flight = match data::get_flight_with_id(&db, id) {
        Some(flight) => flight,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if flight.user_id != user.id {
        return Err(Status::Forbidden);
    }

    match flight.igc {
        Some(igc) => Ok(Response::build()
            .header(ContentType::new("application", "octet-stream"))
            .header(ContentDisposition {
                disposition: DispositionType::Attachment,
                parameters: vec![DispositionParam::Filename(
                    Charset::Us_Ascii,
                    None,
                    format!("flight{}.igc", flight.id).into_bytes(),
                )],
            })
            .sized_body(Cursor::new(igc))
            .finalize()),
        None => Err(Status::NotFound),
    }
}

#[derive(FromForm, Debug)]
pub(crate) struct SubmitForm {
    igc_data: OptionResult<Base64Data>,
    number: OptionResult<i32>,
    aircraft: Option<i32>,
    launch_site: Option<i32>,
    landing_site: Option<i32>,
    launch_date: OptionResult<NaiveDate>,
    launch_time: OptionResult<NaiveTime>,
    landing_time: OptionResult<NaiveTime>,
    track_distance: OptionResult<f32>,
    xcontest_tracktype: String,
    xcontest_distance: OptionResult<f32>,
    xcontest_url: String,
    comment: String,
    video_url: String,
}

#[derive(Serialize)]
struct SubmitContext {
    user: User,
    flight: Option<Flight>,
    aircraft_list: Vec<Aircraft>,
    locations: Vec<Location>,
    error_msg: Option<String>,
}

#[get("/flights/add")]
pub(crate) fn submit_form(user: auth::AuthUser, db: data::Database) -> Template {
    let user = user.into_inner();
    let aircraft_list = data::get_aircraft_for_user(&db, &user);
    let locations = data::get_locations_for_user(&db, &user);
    let context = SubmitContext {
        user,
        flight: None,
        aircraft_list,
        locations,
        error_msg: None,
    };
    Template::render("submit", context)
}

#[get("/flights/add", rank = 2)]
pub(crate) fn submit_form_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

#[post("/flights/add", data = "<data>")]
pub(crate) fn submit(
    user: auth::AuthUser,
    db: data::Database,
    data: Option<Form<SubmitForm>>,
) -> Result<Redirect, Template> {
    let user = user.into_inner();
    let aircraft_list = data::get_aircraft_for_user(&db, &user);
    let locations = data::get_locations_for_user(&db, &user);

    macro_rules! fail {
        ($msg:expr) => {{
            let error_msg = Some($msg.into());
            let ctx = SubmitContext {
                user,
                flight: None,
                aircraft_list,
                locations,
                error_msg,
            };
            return Err(Template::render("submit", ctx));
        }};
    }

    macro_rules! none_if_empty {
        ($val:expr) => {
            if $val.trim().is_empty() {
                None
            } else {
                Some($val)
            }
        };
    }

    if let Some(Form(SubmitForm {
        igc_data: form_igc_data,
        number: form_number,
        aircraft: form_aircraft,
        launch_site: form_launch_site,
        landing_site: form_landing_site,
        launch_date: form_launch_date,
        launch_time: form_launch_time,
        landing_time: form_landing_time,
        track_distance: form_track_distance,
        xcontest_tracktype: form_xcontest_tracktype,
        xcontest_distance: form_xcontest_distance,
        xcontest_url: form_xcontest_url,
        comment: form_comment,
        video_url: form_video_url,
    })) = data
    {
        // IGC data
        let igc = match form_igc_data {
            OptionResult::Ok(bytes) => Some(bytes.0),
            OptionResult::None => None,
            OptionResult::Err(ref e) => fail!(format!("IGC File: {}", e)),
        };

        // Extract basic model data
        let number = match form_number.into_result() {
            Ok(name) => name,
            Err(_) => fail!("Invalid flight number"),
        };
        let user_id = user.id;

        // Extract and validate aircraft
        let aircraft = match form_aircraft {
            Some(id) => {
                match data::get_aircraft_with_id(&db, id) {
                    Some(aircraft) => {
                        // Validate ownership
                        if aircraft.user_id != user.id {
                            fail!("Invalid aircraft")
                        }
                        Some(aircraft)
                    },
                    None => fail!("Invalid aircraft"),
                }
            },
            None => None,
        };

        // Extract date and time
        let mut date_parts = 0;
        let launch_date_naive = match form_launch_date.into_result() {
            Ok(val) => {
                date_parts += 1;
                val
            },
            Err(e) => fail!(format!("Launch date: {}", e)),
        };
        let launch_time_naive = match form_launch_time.into_result() {
            Ok(val) => {
                date_parts += 1;
                val
            },
            Err(e) => fail!(format!("Launch time: {}", e)),
        };
        let landing_time_naive = match form_landing_time.into_result() {
            Ok(val) => {
                date_parts += 1;
                val
            },
            Err(e) => fail!(format!("Landing time: {}", e)),
        };
        if date_parts < 0 && date_parts < 3 {
            fail!("If you specify launch date, launch time or landing time, then the other two values must be provided as well");
        }

        // Extract launch / landing information
        let launch_at = form_launch_site;
        let landing_at = form_landing_site;
        let launch_time = launch_time_naive.map(|time| {
            let ndt = NaiveDateTime::new(launch_date_naive.unwrap(), time);
            DateTime::from_utc(ndt, Utc) // TODO: Timezone
        });
        let landing_time = landing_time_naive.map(|time| {
            let ndt = NaiveDateTime::new(launch_date_naive.unwrap(), time);
            DateTime::from_utc(ndt, Utc) // TODO: Timezone
        });

        // Extract track information
        let track_distance = match form_track_distance.into_result() {
            Ok(val) => val,
            Err(_) => fail!("Invalid track distance"),
        };
        let xcontest_tracktype = none_if_empty!(form_xcontest_tracktype);
        let xcontest_distance = match form_xcontest_distance.into_result() {
            Ok(val) => val,
            Err(_) => fail!("Invalid XContest distance"),
        };
        let xcontest_url = none_if_empty!(form_xcontest_url);

        // Extract other information
        let comment = none_if_empty!(form_comment);
        let video_url = none_if_empty!(form_video_url);

        // Create model
        let aircraft_id = aircraft.as_ref().map(|a| a.id);
        let flight = NewFlight {
            number,
            user_id,
            aircraft_id,
            launch_at,
            landing_at,
            launch_time,
            landing_time,
            track_distance,
            xcontest_tracktype,
            xcontest_distance,
            xcontest_url,
            comment,
            video_url,
            igc,
        };
        // TODO: Error handling
        data::create_flight(&db, flight);
        if let Some(aircraft) = aircraft {
            data::update_user_last_aircraft(&db, &user, &aircraft);
        }

        Ok(Redirect::to("/flights/"))
    } else {
        fail!("Invalid form, could not parse data. Note: Only IGC files up to ~2 MiB can be uploaded.");
    }
}

#[get("/flights/<id>/edit")]
pub(crate) fn edit_form(user: auth::AuthUser, db: data::Database, id: i32) -> Result<Template, Status> {
    let user = user.into_inner();

    // Get data
    let aircraft_list = data::get_aircraft_for_user(&db, &user);
    let locations = data::get_locations_for_user(&db, &user);
    let flight = match data::get_flight_with_id(&db, id) {
        Some(flight) => flight,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if flight.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Render template
    let context = SubmitContext {
        user,
        flight: Some(flight),
        aircraft_list,
        locations,
        error_msg: None,
    };
    Ok(Template::render("submit", context))
}
