//! Flight views.

use std::{collections::HashMap, io::Cursor, sync::Arc};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{
    naive::{NaiveDate, NaiveDateTime, NaiveTime},
    DateTime, Utc,
};
use rocket::{
    get,
    http::{ContentType, Header, Status},
    post,
    request::Request,
    response::{self, Flash, Redirect, Responder, Response},
    routes,
    serde::{
        json::Json,
        {Deserialize, Serialize},
    },
    uri, Route,
};
use rocket_dyn_templates::Template;

use crate::{
    auth, data,
    models::{Flight, Location, NewFlight, User},
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FlightAddForm {
    /// Flight number
    number: Option<i32>,
    /// Glider ID
    glider: Option<i32>,
    /// Launch site ID
    launch_site: Option<i32>,
    /// Landing site ID
    landing_site: Option<i32>,
    /// Launch date
    launch_date: Option<NaiveDate>,
    /// Launch time
    launch_time: Option<NaiveTime>,
    /// Landing time
    landing_time: Option<NaiveTime>,
    /// Whether this was a hikeandfly tour
    hikeandfly: Option<bool>,
    /// Track distance in km
    track_distance: Option<f32>,
    /// XContest tracktype
    xcontest_tracktype: Option<String>,
    /// XContest distance in km
    xcontest_distance: Option<f32>,
    /// XContest URL
    xcontest_url: Option<String>,
    /// Flight comment
    comment: Option<String>,
    /// Flight video URL
    video_url: Option<String>,
    /// IGC file bytes as URL-safe base64 string
    igc_data: Option<String>,
}

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

impl FlightAddForm {
    /// Validate a `FlightAddForm` submission and convert it to a `NewFlight` model.
    ///
    /// If validation fails, return error message.
    async fn into_new_flight(self, user: &User, database: &data::Database) -> Result<NewFlight, String> {
        // Look up and validate glider
        if let Some(glider_id) = self.glider {
            match database
                .run(move |db| data::get_glider_by_id(db, glider_id))
                .await
            {
                Some(glider) => {
                    // Glider found, validate ownership
                    if glider.user_id != user.id {
                        return Err("Invalid glider".into());
                    }
                }
                None => return Err("Invalid glider".into()),
            }
        }

        // Look up and validate locations
        let user_location_ids = database
            .run({
                let user = user.clone();
                move |db| data::get_locations_for_user(db, &user)
            })
            .await
            .into_iter()
            .map(|location| location.id)
            .collect::<Vec<_>>();
        if let Some(ref location_id) = self.launch_site {
            if !user_location_ids.contains(location_id) {
                return Err("Invalid launch location".into());
            }
        }
        if let Some(ref location_id) = self.landing_site {
            if !user_location_ids.contains(location_id) {
                return Err("Invalid landing location".into());
            }
        }

        // Validate distances (must be valid and non-negative)
        if let Some(distance) = self.track_distance {
            if distance.is_infinite()
                || distance.is_subnormal()
                || distance.is_nan()
                || distance.is_sign_negative()
            {
                return Err(format!("Invalid GPS track distance: {}", distance));
            }
        }
        if let Some(distance) = self.xcontest_distance {
            if distance.is_infinite()
                || distance.is_subnormal()
                || distance.is_nan()
                || distance.is_sign_negative()
            {
                return Err(format!("Invalid XContest scored distance: {}", distance));
            }
        }

        // Validate and combine date and time
        let date_parts = self.launch_date.map(|_| 1).unwrap_or_default()
            + self.launch_time.map(|_| 1).unwrap_or_default()
            + self.landing_time.map(|_| 1).unwrap_or_default();
        if date_parts > 0 && date_parts < 3 {
            return Err("If you specify launch date, launch time or landing time, \
                        then the other two values must be provided as well"
                .into());
        }
        let launch_time = self.launch_time.map(|time| {
            let ndt = NaiveDateTime::new(self.launch_date.unwrap(), time);
            DateTime::from_naive_utc_and_offset(ndt, Utc) // TODO: Timezone?
        });
        let landing_time = self.landing_time.map(|time| {
            let ndt = NaiveDateTime::new(self.launch_date.unwrap(), time);
            DateTime::from_naive_utc_and_offset(ndt, Utc) // TODO: Timezone?
        });

        // Create model
        Ok(NewFlight {
            number: self.number,
            user_id: user.id,
            glider_id: self.glider,
            launch_at: self.launch_site,
            landing_at: self.landing_site,
            launch_time,
            landing_time,
            hikeandfly: self.hikeandfly.unwrap_or_default(),
            track_distance: self.track_distance,
            xcontest_tracktype: self.xcontest_tracktype,
            xcontest_distance: self.xcontest_distance,
            xcontest_url: self.xcontest_url,
            comment: self.comment,
            video_url: self.video_url,
        })
    }
}

#[post("/flights", data = "<data>")]
pub async fn add(
    user: auth::AuthUser,
    database: data::Database,
    data: Json<FlightAddForm>,
) -> Result<Status, ApiError> {
    log::debug!("flights::add");
    let user = user.into_inner();

    // TODO: Test error handling for too-large IGC file

    // Decode IGC data
    let igc_bytes = if let Some(ref base64) = data.igc_data {
        Some(
            URL_SAFE_NO_PAD
                .decode(base64)
                .map_err(|e| ApiError::InvalidData {
                    message: format!("Invalid base64 data for IGC file: {}", e),
                })?,
        )
    } else {
        None
    };

    // Convert flight into `NewFlight`
    let new_flight = data
        .into_inner()
        .into_new_flight(&user, &database)
        .await
        .map_err(|e| ApiError::InvalidData {
            message: format!("Invalid flight data: {}", e),
        })?;

    // Insert flight into database
    database
        .run(move |db| {
            data::create_flight(db, &new_flight, igc_bytes);
            log::info!("Created flight for user {}", user.id);
            if let Some(glider_id) = new_flight.glider_id {
                data::update_user_last_glider(db, &user, glider_id);
            }
        })
        .await;

    Ok(Status::Created)
}

#[post("/flights/add", rank = 2)]
pub fn add_nologin() -> ApiError {
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
    routes![
        list,
        list_nologin,
        get,
        get_nologin,
        add,
        add_nologin,
        igc_download,
    ]
}

// Classic views (TODO remove)

/*

#[derive(Serialize)]
struct SubmitContext {
    user: User,
    flight: Option<Flight>,
    max_flight_number: Option<i32>,
    gliders: Vec<Glider>,
    locations: Vec<Location>,
    error_msg: Option<String>,
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

*/

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

// TODO: API tests for flight submission and editing
