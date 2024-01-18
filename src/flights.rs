//! Flight views.

use std::{collections::HashMap, io::Cursor, sync::Arc};

use chrono::{
    naive::{NaiveDate, NaiveDateTime, NaiveTime},
    DateTime, Utc,
};
use rocket::{
    form::{error::ErrorKind, Errors, Form, FromForm},
    get,
    http::{ContentType, Header, Status},
    post,
    request::{FlashMessage, Request},
    response::{self, Flash, Redirect, Responder, Response},
    routes,
    serde::json::Json,
    uri, Route,
};
use rocket_dyn_templates::Template;
use serde::Serialize;

use crate::{
    auth,
    base64::Base64Data,
    data,
    models::{Flight, Glider, Location, NewFlight, User},
    optionresult::OptionResult,
    responders::ApiError,
};

// API types

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiFlightLocation {
    id: i32,
    name: String,
    country_code: String,
    elevation: i32,
}

impl From<Location> for ApiFlightLocation {
    fn from(location: Location) -> Self {
        ApiFlightLocation {
            id: location.id,
            name: location.name,
            country_code: location.country,
            elevation: location.elevation,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiFlightListItem {
    /// Flight ID
    id: i32,
    /// The user-defined flight number
    #[serde(skip_serializing_if = "Option::is_none")]
    number: Option<i32>,
    /// The glider used
    #[serde(skip_serializing_if = "Option::is_none")]
    glider_name: Option<String>,
    /// Inlined launch location
    #[serde(skip_serializing_if = "Option::is_none")]
    launch_at: Option<i32>,
    /// Inlined landing location
    #[serde(skip_serializing_if = "Option::is_none")]
    landing_at: Option<i32>,
    /// Time of launch
    #[serde(skip_serializing_if = "Option::is_none")]
    launch_time: Option<DateTime<Utc>>,
    /// Time of landing
    #[serde(skip_serializing_if = "Option::is_none")]
    landing_time: Option<DateTime<Utc>>,
    /// Flight duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_seconds: Option<u64>,
    /// GPS track length
    #[serde(skip_serializing_if = "Option::is_none")]
    track_distance: Option<f32>,
    /// XContest tracktype (free_flight, flat_triangle or fai_triangle)
    #[serde(skip_serializing_if = "Option::is_none")]
    xcontest_tracktype: Option<String>,
    /// XContest distance
    #[serde(skip_serializing_if = "Option::is_none")]
    xcontest_distance: Option<f32>,
    /// XContest URL
    #[serde(skip_serializing_if = "Option::is_none")]
    xcontest_url: Option<String>,
    /// Comment your flight
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    /// Link to a video of your flight
    #[serde(skip_serializing_if = "Option::is_none")]
    video_url: Option<String>,
    /// Whether you hiked up to launch
    hikeandfly: bool,
    /// Whether an IGC file is present for this flight
    has_igc: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiFlights {
    flights: Vec<ApiFlightListItem>,
    locations: HashMap<i32, ApiFlightLocation>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiFlight {
    /// Flight ID
    id: i32,
    /// The user-defined flight number
    #[serde(skip_serializing_if = "Option::is_none")]
    number: Option<i32>,
    /// The glider used
    #[serde(skip_serializing_if = "Option::is_none")]
    glider_name: Option<String>,
    /// Launch location
    #[serde(skip_serializing_if = "Option::is_none")]
    launch_at: Option<ApiFlightLocation>,
    /// Landing location
    #[serde(skip_serializing_if = "Option::is_none")]
    landing_at: Option<ApiFlightLocation>,
    /// Time of launch
    #[serde(skip_serializing_if = "Option::is_none")]
    launch_time: Option<DateTime<Utc>>,
    /// Time of landing
    #[serde(skip_serializing_if = "Option::is_none")]
    landing_time: Option<DateTime<Utc>>,
    /// Flight duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_seconds: Option<u64>,
    /// GPS track length
    #[serde(skip_serializing_if = "Option::is_none")]
    track_distance: Option<f32>,
    /// XContest tracktype (free_flight, flat_triangle or fai_triangle)
    #[serde(skip_serializing_if = "Option::is_none")]
    xcontest_tracktype: Option<String>,
    /// XContest distance
    #[serde(skip_serializing_if = "Option::is_none")]
    xcontest_distance: Option<f32>,
    /// XContest URL
    #[serde(skip_serializing_if = "Option::is_none")]
    xcontest_url: Option<String>,
    /// Comment your flight
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    /// Link to a video of your flight
    #[serde(skip_serializing_if = "Option::is_none")]
    video_url: Option<String>,
    /// Whether you hiked up to launch
    hikeandfly: bool,
    /// Whether an IGC file is present for this flight
    has_igc: bool,
}

// Forms

// ...

// API endpoints

#[get("/flights")]
pub async fn list(database: data::Database, user: auth::AuthUser) -> Json<ApiFlights> {
    let user = Arc::new(user.into_inner());

    // Get all flights for user
    let flights = database
        .run({
            let user = user.clone();
            move |db| data::get_flights_for_user(db, &user)
        })
        .await;

    // Get all gliders for user
    let glider_name_map = database
        .run({
            let user = user.clone();
            move |db| data::get_gliders_for_user(db, &user)
        })
        .await
        .into_iter()
        .map(|glider| (glider.id, glider.to_string()))
        .collect::<HashMap<i32, String>>();

    // Get all locations used
    let mut location_ids = flights
        .iter()
        .flat_map(|flight| vec![flight.launch_at, flight.landing_at])
        .flatten()
        .collect::<Vec<_>>();
    location_ids.sort_unstable();
    location_ids.dedup();
    let location_map = database
        .run(move |db| data::get_locations_with_ids(db, &location_ids))
        .await
        .into_iter()
        .map(|location| (location.id, location.into()))
        .collect::<HashMap<i32, ApiFlightLocation>>();

    // Get all flight IDs with IGC files
    let flight_ids_with_igc = database
        .run({
            let user = user.clone();
            move |db| data::get_flight_ids_with_igc_for_user(db, &user)
        })
        .await;

    // Add details to flights
    let api_flights = flights
        .into_iter()
        .map(|flight| {
            // Look up glider name
            let glider_name = flight.glider_id.and_then(|id| glider_name_map.get(&id).cloned());

            // Calculate duration
            let duration_seconds = match (flight.launch_time, flight.landing_time) {
                (Some(launch), Some(landing)) => {
                    let duration = (landing - launch).num_seconds();
                    if duration < 0 {
                        None
                    } else {
                        Some(duration as u64)
                    }
                }
                _ => None,
            };

            // Look up whether an IGC file exists for this flight
            let has_igc = flight_ids_with_igc.contains(&flight.id);

            ApiFlightListItem {
                id: flight.id,
                number: flight.number,
                glider_name,
                launch_at: flight.launch_at,
                landing_at: flight.landing_at,
                launch_time: flight.launch_time,
                landing_time: flight.landing_time,
                duration_seconds,
                track_distance: flight.track_distance,
                xcontest_tracktype: flight.xcontest_tracktype,
                xcontest_distance: flight.xcontest_distance,
                xcontest_url: flight.xcontest_url,
                comment: flight.comment,
                video_url: flight.video_url,
                hikeandfly: flight.hikeandfly,
                has_igc,
            }
        })
        .collect::<Vec<_>>();

    // Render template
    Json(ApiFlights {
        locations: location_map,
        flights: api_flights,
    })
}

#[get("/flights", rank = 2)]
pub fn list_nologin() -> ApiError {
    ApiError::MissingAuthentication
}

#[get("/flights/<id>")]
pub async fn get(id: i32, database: data::Database, user: auth::AuthUser) -> Result<Json<ApiFlight>, Status> {
    let user = user.into_inner();

    // Get flight
    let flight = match database.run(move |db| data::get_flight_with_id(db, id)).await {
        Some(flight) => Arc::new(flight),
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if flight.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Check whether flight has IGC data
    let has_igc = database
        .run({
            let flight = flight.clone();
            move |db| data::flight_has_igc(db, &flight)
        })
        .await;

    // Resolve foreign keys
    let launch_at = database
        .run({
            let flight = flight.clone();
            move |db| flight.launch_at.and_then(|id| data::get_location_by_id(db, id))
        })
        .await
        .map(|location| location.into());
    let landing_at = database
        .run({
            let flight = flight.clone();
            move |db| flight.landing_at.and_then(|id| data::get_location_by_id(db, id))
        })
        .await
        .map(|location| location.into());
    let glider_name = database
        .run({
            let flight = flight.clone();
            move |db| flight.glider_id.and_then(|id| data::get_glider_by_id(db, id))
        })
        .await
        .map(|glider| glider.to_string());

    // Calculate duration
    let duration_seconds = match (flight.launch_time, flight.landing_time) {
        (Some(launch), Some(landing)) => {
            let duration = (landing - launch).num_seconds();
            if duration < 0 {
                None
            } else {
                Some(duration as u64)
            }
        }
        _ => None,
    };

    // Render template
    let flight = Arc::try_unwrap(flight).expect("cannot unwrap flight Arc");
    Ok(Json(ApiFlight {
        id: flight.id,
        number: flight.number,
        glider_name,
        launch_at,
        landing_at,
        launch_time: flight.launch_time,
        landing_time: flight.landing_time,
        duration_seconds,
        track_distance: flight.track_distance,
        xcontest_tracktype: flight.xcontest_tracktype,
        xcontest_distance: flight.xcontest_distance,
        xcontest_url: flight.xcontest_url,
        comment: flight.comment,
        video_url: flight.video_url,
        hikeandfly: flight.hikeandfly,
        has_igc,
    }))
}

#[get("/flights/<id>", rank = 2)]
#[allow(unused_variables)]
pub fn get_nologin(id: i32) -> ApiError {
    ApiError::MissingAuthentication
}

// IGC Download

/// Responder that returns the `data` bytes with the `content_type` header as a
/// file download (disposition=attachment).
pub struct FileAttachment {
    data: Vec<u8>,
    content_type: ContentType,
    filename: String,
}

impl FileAttachment {
    /// Note: Ensure that the `filename` does not need any escaping / encoding!
    ///       There will be no encoding done by this function.
    pub fn new(data: Vec<u8>, content_type: ContentType, filename: String) -> Self {
        Self {
            data,
            content_type,
            filename,
        }
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for FileAttachment {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(self.content_type)
            .header(Header::new(
                "content-disposition",
                format!("attachment; filename=\"{}\"", self.filename),
            ))
            .sized_body(self.data.len(), Cursor::new(self.data))
            .ok()
    }
}

#[get("/flights/<id>/igc")]
pub async fn igc_download(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
) -> Result<FileAttachment, Status> {
    let user = user.into_inner();

    // Get flight
    let flight = match database.run(move |db| data::get_flight_with_id(db, id)).await {
        Some(flight) => flight,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if flight.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Get IGC data
    let flight_id = flight.id;
    match database
        .run(move |db| data::get_igc_for_flight(db, &flight))
        .await
    {
        Some(igc) => Ok(FileAttachment::new(
            igc.data,
            ContentType::new("application", "octet-stream"),
            format!("flight{}.igc", flight_id),
        )),
        None => Err(Status::NotFound),
    }
}

/// Return vec of all API routes.
pub fn api_routes() -> Vec<Route> {
    routes![list, list_nologin, get, get_nologin, igc_download]
}

// Classic views (TODO remove)

#[derive(FromForm, Debug)]
pub struct FlightForm {
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
    async fn into_new_flight(
        self,
        user: &User,
        database: &data::Database,
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
                match database.run(move |db| data::get_glider_by_id(db, id)).await {
                    Some(glider) => {
                        // Validate ownership
                        if glider.user_id != user.id {
                            return Err("Invalid glider".into());
                        }
                        Some(glider)
                    }
                    None => return Err("Invalid glider".into()),
                }
            }
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
            }
            Err(e) => return Err(format!("Launch date: {}", e)),
        };
        let launch_time_naive = match self.launch_time.into_result() {
            Ok(val) => {
                if val.is_some() {
                    date_parts += 1;
                }
                val
            }
            Err(e) => return Err(format!("Launch time: {}", e)),
        };
        let landing_time_naive = match self.landing_time.into_result() {
            Ok(val) => {
                if val.is_some() {
                    date_parts += 1;
                }
                val
            }
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
            DateTime::from_naive_utc_and_offset(ndt, Utc) // TODO: Timezone
        });
        let landing_time = landing_time_naive.map(|time| {
            let ndt = NaiveDateTime::new(launch_date_naive.unwrap(), time);
            DateTime::from_naive_utc_and_offset(ndt, Utc) // TODO: Timezone
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
pub async fn submit_form(user: auth::AuthUser, database: data::Database) -> Template {
    let user = user.into_inner();

    // Query database
    let (gliders, locations, max_flight_number) = database
        .run({
            let user = user.clone();
            move |db| {
                (
                    data::get_gliders_for_user(db, &user),
                    data::get_locations_for_user(db, &user),
                    data::get_max_flight_number_for_user(db, &user),
                )
            }
        })
        .await;

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
pub fn submit_form_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

#[post("/flights/add", data = "<data>")]
pub async fn submit(
    user: auth::AuthUser,
    database: data::Database,
    data: Result<Form<FlightForm>, Errors<'_>>,
) -> Result<Redirect, Template> {
    log::debug!("flights::submit");
    let user = user.into_inner();

    let (gliders, locations, max_flight_number) = database
        .run({
            let user = user.clone();
            move |db| {
                (
                    data::get_gliders_for_user(db, &user),
                    data::get_locations_for_user(db, &user),
                    data::get_max_flight_number_for_user(db, &user),
                )
            }
        })
        .await;

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

    match data {
        Ok(form) => {
            // Unwrap `Form<T>`
            let form = form.into_inner();

            // Convert flight into `NewFlight`
            let (new_flight, igc) = match form.into_new_flight(&user, &database).await {
                Ok(val) => val,
                Err(msg) => fail!(msg),
            };

            // TODO: Error handling

            // Insert flight into database
            database
                .run(move |db| {
                    data::create_flight(db, &new_flight, igc);
                    log::info!("Created flight for user {}", user.id);
                    if let Some(glider_id) = new_flight.glider_id {
                        data::update_user_last_glider(db, &user, glider_id);
                    }
                })
                .await;

            Ok(Redirect::to(uri!(list)))
        }
        Err(errors) => {
            if let Some(error) = errors.first() {
                if let ErrorKind::InvalidLength {
                    max: Some(max_bytes), ..
                } = error.kind
                {
                    fail!(format!(
                        "The size of the form submission is larger than {} KiB, form cannot be submitted.",
                        max_bytes / 1024
                    ));
                }
            }
            eprintln!("Error: Could not parse form: {:?}", errors);
            fail!("Invalid form, could not parse data. Please contact the admin.");
        }
    }
}

#[get("/flights/<id>/edit")]
pub async fn edit_form(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
    flash: Option<FlashMessage<'_>>,
) -> Result<Template, Status> {
    let user = user.into_inner();

    // Get data
    let (gliders, locations, flight_opt) = database
        .run({
            let user = user.clone();
            move |db| {
                (
                    data::get_gliders_for_user(db, &user),
                    data::get_locations_for_user(db, &user),
                    data::get_flight_with_id(db, id),
                )
            }
        })
        .await;
    let flight = match flight_opt {
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
        error_msg: flash.map(|f: FlashMessage| f.message().to_owned()),
    };
    Ok(Template::render("flight_submit", context))
}

#[derive(Debug, Responder)]
pub enum EditResponse {
    Success(Redirect),
    Message(Flash<Redirect>),
    Error(Status),
}

#[post("/flights/<id>/edit", data = "<data>")]
pub async fn edit(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
    data: Result<Form<FlightForm>, Errors<'_>>,
) -> EditResponse {
    let user = user.into_inner();

    /// If something fails, redirect back to the edit form with an error message.
    macro_rules! fail {
        ($msg:expr) => {{
            return EditResponse::Message(Flash::error(Redirect::to(uri!(edit_form(id))), $msg));
        }};
    }

    // Get flight
    let mut flight = match database.run(move |db| data::get_flight_with_id(db, id)).await {
        Some(flight) => flight,
        None => return EditResponse::Error(Status::NotFound),
    };

    // Ownership check
    if flight.user_id != user.id {
        return EditResponse::Error(Status::Forbidden);
    }

    // Get `NewFlight` instance from form
    let (new_flight, igc) = match data {
        Ok(form) => match form.into_inner().into_new_flight(&user, &database).await {
            Ok(val) => val,
            Err(msg) => fail!(msg),
        },
        Err(errors) => {
            if let Some(error) = errors.first() {
                if let ErrorKind::InvalidLength {
                    max: Some(max_bytes), ..
                } = error.kind
                {
                    fail!(format!(
                        "The size of the form submission is larger than {} KiB, form cannot be submitted.",
                        max_bytes / 1024
                    ));
                }
            }
            eprintln!("Error: Could not parse form: {:?}", errors);
            fail!("Invalid form, could not parse data. Please contact the admin.");
        }
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
    database
        .run(move |db| {
            data::update_flight(db, &flight);
            if let Some(data) = igc {
                // We don't want to overwrite IGC data with nothing.
                data::update_igc(db, &flight, &data);
            }
        })
        .await;

    // Render template
    EditResponse::Success(Redirect::to(uri!(list)))
}

#[derive(Serialize)]
struct DeleteContext {
    user: User,
    flight: Flight,
}

#[get("/flights/<id>/delete")]
pub async fn delete_form(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
) -> Result<Template, Status> {
    let user = user.into_inner();

    // Get data
    let flight = match database.run(move |db| data::get_flight_with_id(db, id)).await {
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
pub async fn delete(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
) -> Result<Flash<Redirect>, Status> {
    let user = user.into_inner();

    // Get data
    let flight = match database.run(move |db| data::get_flight_with_id(db, id)).await {
        Some(flight) => flight,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if flight.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Delete database entry
    let flight_id = flight.id;
    database
        .run(move |db| data::delete_flight(db, flight))
        .await
        .map(|()| {
            log::info!("Deleted flight with ID {}", flight_id);
            Flash::success(Redirect::to(uri!(list)), "Flight deleted")
        })
        .or_else(|e| {
            log::error!("Could not delete flight with ID {}: {}", flight_id, e);
            Ok(Flash::error(Redirect::to(uri!(list)), "Could not delete flight"))
        })
}
