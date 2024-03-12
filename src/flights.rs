//! Flight views.

use std::{collections::HashMap, io::Cursor, sync::Arc};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{
    naive::{NaiveDate, NaiveDateTime, NaiveTime},
    DateTime, Utc,
};
use rocket::{
    delete, get,
    http::{ContentType, Header, Status},
    post,
    request::Request,
    response::{self, Responder, Response},
    routes,
    serde::json::Json,
    Route,
};
use serde::{Deserialize, Serialize};

use crate::{
    auth, data,
    models::{Location, NewFlight, User},
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
    /// The name of the glider used
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
    /// The ID of the glider used
    #[serde(skip_serializing_if = "Option::is_none")]
    glider_id: Option<i32>,
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
pub struct FlightAddUpdateForm {
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
        glider_id: flight.glider_id,
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

impl FlightAddUpdateForm {
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
    data: Json<FlightAddUpdateForm>,
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

    // Convert request data into `NewFlight`
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

#[post("/flights/<id>", data = "<data>")]
pub async fn edit(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
    data: Json<FlightAddUpdateForm>,
) -> Result<Status, ApiError> {
    log::debug!("flights::add");
    let user = user.into_inner();

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
    log::info!("flight igc {:?}", igc_bytes.is_some());

    // Get flight
    let mut flight = database
        .run(move |db| data::get_flight_with_id(db, id))
        .await
        .ok_or(ApiError::NotFound)?;

    // Ownership check
    if flight.user_id != user.id {
        return Err(ApiError::NotFound);
    }

    // TODO: Test error handling for too-large IGC file

    // Convert request data into `NewFlight`
    let new_flight = data
        .into_inner()
        .into_new_flight(&user, &database)
        .await
        .map_err(|e| ApiError::InvalidData {
            message: format!("Invalid flight data: {}", e),
        })?;

    // Update existing flight
    flight.number = new_flight.number;
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
            // Note: Only add IGC data if flight doesn't have IGC data yet. Never modify IGC.
            let has_igc = data::flight_has_igc(db, &flight);
            if !has_igc {
                if let Some(data) = igc_bytes {
                    data::update_igc(db, &flight, &data);
                }
            }
        })
        .await;

    Ok(Status::NoContent)
}

#[post("/flights/<id>", rank = 3)]
#[allow(unused_variables)]
pub fn edit_nologin(id: i32) -> ApiError {
    ApiError::MissingAuthentication
}

/// Delete a flight.
///
/// - Return "HTTP 204 No Content" if flight was deleted.
/// - Return "HTTP 404 Not Found" if flight was not found.
/// - Return "HTTP 403 Forbidden" if flight does not belong to user.
#[delete("/flights/<id>")]
pub async fn delete(user: auth::AuthUser, database: data::Database, id: i32) -> Result<Status, Status> {
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
            Status::NoContent
        })
        .map_err(|e| {
            log::error!("Could not delete flight with ID {}: {}", flight_id, e);
            Status::InternalServerError
        })
}

#[delete("/flights/<id>", rank = 2)]
#[allow(unused_variables)]
pub fn delete_nologin(id: i32) -> ApiError {
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
        edit,
        edit_nologin,
        delete,
        delete_nologin,
        igc_download,
    ]
}

// TODO: API tests for flight submission and editing

#[cfg(test)]
mod tests {
    use rocket::{self, local::blocking::Client};

    use crate::{
        models::NewFlight,
        templates,
        test_utils::{make_test_config, DbTestContext},
        Config,
    };

    use super::*;

    /// Create a new test client. Cookie tracking is disabled.
    fn make_client() -> Client {
        let app = rocket::custom(make_test_config())
            .attach(data::Database::fairing())
            .attach(templates::fairing(&Config::default()))
            .mount("/", api_routes());
        Client::untracked(app).expect("valid rocket instance")
    }

    #[test]
    fn delete_flight() {
        let ctx = DbTestContext::new();
        let client = make_client();

        macro_rules! delete_flight {
            ($id:expr, $cookie:expr) => {
                client
                    .delete(format!("/flights/{}", $id))
                    .private_cookie($cookie)
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }

        // Add flights
        let flight1 = data::create_flight(
            &mut *ctx.force_get_conn(),
            &NewFlight {
                number: Some(1),
                user_id: ctx.testuser1.user.id,
                ..Default::default()
            },
            None,
        );
        let flight2 = data::create_flight(
            &mut *ctx.force_get_conn(),
            &NewFlight {
                number: Some(2),
                user_id: ctx.testuser2.user.id,
                ..Default::default()
            },
            None,
        );

        // Delete flight 1 from user 1: OK
        let resp = delete_flight!(flight1.id, ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::NoContent);

        // Delete flight 2 from user 1: Forbidden
        let resp = delete_flight!(flight2.id, ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::Forbidden);

        // Delete flight 2 from user 2: OK
        let resp = delete_flight!(flight2.id, ctx.auth_cookie_user2());
        assert_eq!(resp.status(), Status::NoContent);

        // Delete non-existing flight: Not found
        let resp = delete_flight!(flight2.id, ctx.auth_cookie_user2());
        assert_eq!(resp.status(), Status::NotFound);
    }
}
