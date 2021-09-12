//! Flight views.

use std::collections::HashMap;
use std::io::Cursor;

use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use chrono::{DateTime, Utc};
use rocket::http::hyper::header::{Charset, ContentDisposition, DispositionParam, DispositionType};
use rocket::http::{ContentType, Status};
use rocket::request::{FlashMessage, Form, FromForm};
use rocket::response::{Flash, Redirect, Responder, Response};
use rocket::{get, post, uri};
use rocket_contrib::templates::Template;
use serde::Serialize;

use crate::base64::Base64Data;
use crate::flash::flashes_from_flash_opt;
use crate::models::{Flight, Glider, Location, NewFlight, User};
use crate::optionresult::OptionResult;
use crate::{auth, data};

#[derive(Serialize)]
struct FlightWithDetails<'a> {
    flight: Flight,
    glider: Option<&'a Glider>,
    launch_at: Option<&'a Location>,
    landing_at: Option<&'a Location>,
    duration_seconds: Option<u64>,
    has_igc: bool,
}

#[derive(Serialize)]
struct FlightsContext<'a> {
    user: User,
    flights: Vec<FlightWithDetails<'a>>,
    flashes: Vec<crate::flash::FlashMessage>,
}

#[get("/flights")]
pub(crate) fn list(db: data::Database, user: auth::AuthUser, flash: Option<FlashMessage>) -> Template {
    let user = user.into_inner();

    // Get all flights
    let flights = data::get_flights_for_user(&db, &user);

    // Get all gliders for user
    let glider_map = data::get_gliders_for_user(&db, &user)
        .into_iter()
        .map(|glider| (glider.id, glider))
        .collect::<HashMap<i32, Glider>>();

    // Get all locations used
    let mut location_ids = flights
        .iter()
        .flat_map(|flight| vec![flight.launch_at, flight.landing_at])
        .filter_map(|opt| opt)
        .collect::<Vec<_>>();
    location_ids.sort_unstable();
    location_ids.dedup();
    let location_map = data::get_locations_with_ids(&db, &location_ids)
        .into_iter()
        .map(|location| (location.id, location))
        .collect::<HashMap<i32, Location>>();

    // Get all flight IDs with IGC files
    let flights_with_igc = data::get_flight_ids_with_igc_for_user(&db, &user);

    // Add details to flights
    let flights_with_details = flights
        .into_iter()
        .map(|flight| {
            // Look up glider
            let glider = flight.glider_id.and_then(|id| glider_map.get(&id));

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

            // Look up whether an IGC file exists for this flight
            let has_igc = flights_with_igc.contains(&flight.id);

            FlightWithDetails {
                flight,
                glider,
                launch_at,
                landing_at,
                duration_seconds,
                has_igc,
            }
        })
        .collect::<Vec<_>>();

    // Render template
    let context = FlightsContext {
        user,
        flights: flights_with_details,
        flashes: flashes_from_flash_opt(flash),
    };
    Template::render("flights", &context)
}

#[get("/flights", rank = 2)]
pub(crate) fn list_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

#[derive(Serialize)]
struct FlightContext<'a> {
    user: User,
    flight: Flight,
    glider: Option<&'a Glider>,
    launch_at: Option<&'a Location>,
    landing_at: Option<&'a Location>,
    duration_seconds: Option<u64>,
    has_igc: bool,
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

    // Check whether flight has IGC data
    let has_igc = data::flight_has_igc(&db, &flight);

    // Resolve foreign keys
    let launch_at = flight.launch_at.and_then(|id| data::get_location_by_id(&db, id));
    let landing_at = flight.landing_at.and_then(|id| data::get_location_by_id(&db, id));
    let glider = flight.glider_id.and_then(|id| data::get_glider_with_id(&db, id));

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
        glider: glider.as_ref(),
        launch_at: launch_at.as_ref(),
        landing_at: landing_at.as_ref(),
        duration_seconds,
        has_igc,
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

    // Get IGC data
    match data::get_igc_for_flight(&db, &flight) {
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
            .sized_body(Cursor::new(igc.data))
            .finalize()),
        None => Err(Status::NotFound),
    }
}

#[derive(FromForm, Debug)]
pub(crate) struct FlightForm {
    igc_data: OptionResult<Base64Data>,
    number: OptionResult<i32>,
    glider: Option<i32>,
    launch_site: Option<i32>,
    landing_site: Option<i32>,
    launch_date: OptionResult<NaiveDate>,
    launch_time: OptionResult<NaiveTime>,
    landing_time: OptionResult<NaiveTime>,
    hikeandfly: bool,
    track_distance: OptionResult<f32>,
    xcontest_tracktype: String,
    xcontest_distance: OptionResult<f32>,
    xcontest_url: String,
    comment: String,
    video_url: String,
}

impl FlightForm {
    /// Validate a `FlightForm` submission and convert it to a `NewFlight` model (plus the IGC file data).
    fn into_new_flight(
        self,
        user: &User,
        db: &data::Database,
    ) -> Result<(NewFlight, Option<Vec<u8>>), String> {
        macro_rules! none_if_empty {
            ($val:expr) => {
                if $val.trim().is_empty() {
                    None
                } else {
                    Some($val)
                }
            };
        }

        // IGC data
        let igc = match self.igc_data {
            OptionResult::Ok(bytes) => Some(bytes.0),
            OptionResult::None => None,
            OptionResult::Err(ref e) => return Err(format!("IGC File: {}", e)),
        };

        // Extract basic model data
        let number = match self.number.into_result() {
            Ok(name) => name,
            Err(_) => return Err("Invalid flight number".into()),
        };
        let user_id = user.id;

        // Extract and validate glider
        let glider = match self.glider {
            Some(id) => {
                match data::get_glider_with_id(&db, id) {
                    Some(glider) => {
                        // Validate ownership
                        if glider.user_id != user.id {
                            return Err("Invalid glider".into());
                        }
                        Some(glider)
                    },
                    None => return Err("Invalid glider".into()),
                }
            },
            None => None,
        };

        // Extract date and time
        let mut date_parts = 0;
        let launch_date_naive = match self.launch_date.into_result() {
            Ok(val) => {
                if val.is_some() {
                    date_parts += 1;
                }
                val
            },
            Err(e) => return Err(format!("Launch date: {}", e)),
        };
        let launch_time_naive = match self.launch_time.into_result() {
            Ok(val) => {
                if val.is_some() {
                    date_parts += 1;
                }
                val
            },
            Err(e) => return Err(format!("Launch time: {}", e)),
        };
        let landing_time_naive = match self.landing_time.into_result() {
            Ok(val) => {
                if val.is_some() {
                    date_parts += 1;
                }
                val
            },
            Err(e) => return Err(format!("Landing time: {}", e)),
        };
        if date_parts > 0 && date_parts < 3 {
            return Err("If you specify launch date, launch time or landing time, \
                        then the other two values must be provided as well"
                .into());
        }

        // Extract launch / landing information
        let launch_at = self.launch_site;
        let landing_at = self.landing_site;
        let launch_time = launch_time_naive.map(|time| {
            let ndt = NaiveDateTime::new(launch_date_naive.unwrap(), time);
            DateTime::from_utc(ndt, Utc) // TODO: Timezone
        });
        let landing_time = landing_time_naive.map(|time| {
            let ndt = NaiveDateTime::new(launch_date_naive.unwrap(), time);
            DateTime::from_utc(ndt, Utc) // TODO: Timezone
        });
        let hikeandfly = self.hikeandfly;

        // Extract track information
        let track_distance = match self.track_distance.into_result() {
            Ok(val) => val,
            Err(_) => return Err("Invalid track distance".into()),
        };
        let xcontest_tracktype = none_if_empty!(self.xcontest_tracktype);
        let xcontest_distance = match self.xcontest_distance.into_result() {
            Ok(val) => val,
            Err(_) => return Err("Invalid XContest distance".into()),
        };
        let xcontest_url = none_if_empty!(self.xcontest_url);

        // Extract other information
        let comment = none_if_empty!(self.comment);
        let video_url = none_if_empty!(self.video_url);

        // Create model
        let glider_id = glider.as_ref().map(|a| a.id);
        Ok((
            NewFlight {
                number,
                user_id,
                glider_id,
                launch_at,
                landing_at,
                launch_time,
                landing_time,
                hikeandfly,
                track_distance,
                xcontest_tracktype,
                xcontest_distance,
                xcontest_url,
                comment,
                video_url,
            },
            igc,
        ))
    }
}

#[derive(Serialize)]
struct SubmitContext {
    user: User,
    flight: Option<Flight>,
    max_flight_number: Option<i32>,
    gliders: Vec<Glider>,
    locations: Vec<Location>,
    error_msg: Option<String>,
}

#[get("/flights/add")]
pub(crate) fn submit_form(user: auth::AuthUser, db: data::Database) -> Template {
    let user = user.into_inner();
    let gliders = data::get_gliders_for_user(&db, &user);
    let locations = data::get_locations_for_user(&db, &user);
    let max_flight_number = data::get_max_flight_number_for_user(&db, &user);
    let context = SubmitContext {
        user,
        flight: None,
        max_flight_number,
        gliders,
        locations,
        error_msg: None,
    };
    Template::render("flight_submit", context)
}

#[get("/flights/add", rank = 2)]
pub(crate) fn submit_form_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

#[post("/flights/add", data = "<data>")]
pub(crate) fn submit(
    user: auth::AuthUser,
    db: data::Database,
    data: Option<Form<FlightForm>>,
) -> Result<Redirect, Template> {
    log::debug!("flights::submit");

    let user = user.into_inner();
    let gliders = data::get_gliders_for_user(&db, &user);
    let locations = data::get_locations_for_user(&db, &user);
    let max_flight_number = data::get_max_flight_number_for_user(&db, &user);

    macro_rules! fail {
        ($msg:expr) => {{
            let error_msg = $msg.into();
            log::error!("Could not submit flight for user {}: {}", user.id, error_msg);
            let ctx = SubmitContext {
                user,
                flight: None,
                max_flight_number,
                gliders,
                locations,
                error_msg: Some(error_msg),
            };
            return Err(Template::render("flight_submit", ctx));
        }};
    }

    if let Some(Form(form)) = data {
        let (new_flight, igc) = match form.into_new_flight(&user, &db) {
            Ok(val) => val,
            Err(msg) => fail!(msg),
        };
        // TODO: Error handling
        data::create_flight(&db, &new_flight, igc);
        log::info!("Created flight for user {}", user.id);
        if let Some(glider_id) = new_flight.glider_id {
            data::update_user_last_glider(&db, &user, glider_id);
        }

        Ok(Redirect::to(uri!(list)))
    } else {
        fail!("Invalid form, could not parse data. Note: Only IGC files up to ~2 MiB can be uploaded.");
    }
}

#[get("/flights/<id>/edit")]
pub(crate) fn edit_form(
    user: auth::AuthUser,
    db: data::Database,
    id: i32,
    flash: Option<FlashMessage>,
) -> Result<Template, Status> {
    let user = user.into_inner();

    // Get data
    let gliders = data::get_gliders_for_user(&db, &user);
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
        max_flight_number: None,
        gliders,
        locations,
        error_msg: flash.map(|f: FlashMessage| f.msg().to_owned()),
    };
    Ok(Template::render("flight_submit", context))
}

#[derive(Debug, Responder)]
pub(crate) enum EditResponse {
    Success(Redirect),
    Message(Flash<Redirect>),
    Error(Status),
}

#[post("/flights/<id>/edit", data = "<data>")]
pub(crate) fn edit(
    user: auth::AuthUser,
    db: data::Database,
    id: i32,
    data: Option<Form<FlightForm>>,
) -> EditResponse {
    let user = user.into_inner();

    /// If something fails, redirect back to the edit form with an error message.
    macro_rules! fail {
        ($msg:expr) => {{
            return EditResponse::Message(Flash::error(Redirect::to(uri!(edit_form: id)), $msg));
        }};
    }

    // Get flight
    let mut flight = match data::get_flight_with_id(&db, id) {
        Some(flight) => flight,
        None => return EditResponse::Error(Status::NotFound),
    };

    // Ownership check
    if flight.user_id != user.id {
        return EditResponse::Error(Status::Forbidden);
    }

    // Get `NewFlight` instance from form
    let (new_flight, igc) = if let Some(Form(form)) = data {
        match form.into_new_flight(&user, &db) {
            Ok(val) => val,
            Err(msg) => fail!(msg),
        }
    } else {
        fail!("Invalid form, could not parse data. Note: Only IGC files up to ~2 MiB can be uploaded.");
    };

    // Update existing flight
    flight.number = new_flight.number;
    flight.user_id = new_flight.user_id;
    flight.glider_id = new_flight.glider_id;
    flight.launch_at = new_flight.launch_at;
    flight.landing_at = new_flight.landing_at;
    flight.launch_time = new_flight.launch_time;
    flight.landing_time = new_flight.landing_time;
    flight.hikeandfly = new_flight.hikeandfly;
    flight.track_distance = new_flight.track_distance;
    flight.xcontest_tracktype = new_flight.xcontest_tracktype;
    flight.xcontest_distance = new_flight.xcontest_distance;
    flight.xcontest_url = new_flight.xcontest_url;
    flight.comment = new_flight.comment;
    flight.video_url = new_flight.video_url;

    // Save changes
    // TODO: Error handling
    data::update_flight(&db, &flight);
    if let Some(data) = igc {
        // We don't want to overwrite IGC data with nothing.
        data::update_igc(&db, &flight, &data);
    }

    // Render template
    EditResponse::Success(Redirect::to(uri!(list)))
}

#[derive(Serialize)]
struct DeleteContext {
    user: User,
    flight: Flight,
}

#[get("/flights/<id>/delete")]
pub(crate) fn delete_form(user: auth::AuthUser, db: data::Database, id: i32) -> Result<Template, Status> {
    let user = user.into_inner();

    // Get data
    let flight = match data::get_flight_with_id(&db, id) {
        Some(flight) => flight,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if flight.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Render template
    let context = DeleteContext { user, flight };
    Ok(Template::render("flight_delete", context))
}

#[post("/flights/<id>/delete")]
pub(crate) fn delete(user: auth::AuthUser, db: data::Database, id: i32) -> Result<Flash<Redirect>, Status> {
    let user = user.into_inner();

    // Get data
    let flight = match data::get_flight_with_id(&db, id) {
        Some(flight) => flight,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if flight.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Delete database entry
    let flight_id = flight.id;
    data::delete_flight(&db, flight)
        .map(|()| {
            log::info!("Deleted flight with ID {}", flight_id);
            Flash::success(Redirect::to(uri!(list)), "Flight deleted")
        })
        .or_else(|e| {
            log::error!("Could not delete flight with ID {}: {}", flight_id, e);
            Ok(Flash::error(Redirect::to(uri!(list)), "Could not delete flight"))
        })
}
