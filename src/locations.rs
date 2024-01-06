//! Location views.

use std::convert::TryFrom;

use diesel_geography::types::GeogPoint;
use rocket::{delete, get, http::Status, post, routes, serde::json::Json, Route};
use serde::{Deserialize, Serialize};

use crate::{
    auth, data,
    models::{LocationWithCount, NewLocation, User},
    responders::ApiError,
};

// API types

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiCoordinates {
    lon: f64,
    lat: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiLocation {
    id: i32,
    name: String,
    country_code: String,
    elevation: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    coordinates: Option<ApiCoordinates>,
    flight_count: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiLocations {
    locations: Vec<ApiLocation>,
}

// Forms

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LocationCreateUpdateForm {
    name: String,
    country_code: String,
    elevation: i32,
    coordinates: Option<ApiCoordinates>,
}

// Contexts

#[derive(Serialize)]
struct LocationContext {
    user: User,
    location: LocationWithCount,
}

// API endpoints

#[get("/locations")]
pub async fn list(database: data::Database, user: auth::AuthUser) -> Json<ApiLocations> {
    let user = user.into_inner();

    // Get all locations for user
    let locations = database
        .run(move |db| data::get_all_locations_with_stats_for_user(db, &user))
        .await
        .into_iter()
        .map(|location| ApiLocation {
            id: location.id,
            name: location.name,
            country_code: location.country,
            elevation: location.elevation,
            coordinates: location.geog.map(|geog| ApiCoordinates {
                lon: geog.x,
                lat: geog.y,
            }),
            flight_count: u64::try_from(location.count.max(0)).unwrap(),
        })
        .collect();

    // Render template
    Json(ApiLocations { locations })
}

#[get("/locations", rank = 2)]
pub fn list_nologin() -> ApiError {
    ApiError::MissingAuthentication
}

#[get("/locations/<id>")]
pub async fn get(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
) -> Result<Json<ApiLocation>, Status> {
    let user = user.into_inner();

    // Get data
    let location = match database
        .run(move |db| data::get_location_with_flight_count_by_id(db, id))
        .await
    {
        Some(location) => location,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if location.user_id != user.id {
        return Err(Status::Forbidden);
    }

    Ok(Json(ApiLocation {
        id: location.id,
        name: location.name,
        country_code: location.country,
        elevation: location.elevation,
        coordinates: location.geog.map(|geog| ApiCoordinates {
            lon: geog.x,
            lat: geog.y,
        }),
        flight_count: u64::try_from(location.count.max(0)).unwrap(),
    }))
}

#[get("/locations/<id>", rank = 2)]
#[allow(unused_variables)]
pub fn get_nologin(id: i32) -> ApiError {
    ApiError::MissingAuthentication
}

#[post("/locations", data = "<data>")]
pub async fn add(
    user: auth::AuthUser,
    database: data::Database,
    data: Json<LocationCreateUpdateForm>,
) -> Status {
    log::debug!("locations::add");
    let user = user.into_inner();

    // Unwrap form data
    let LocationCreateUpdateForm {
        name,
        country_code,
        elevation,
        coordinates,
    } = data.into_inner();

    // Create model
    let location = NewLocation {
        name,
        country: country_code,
        elevation,
        user_id: user.id,
        geog: if let Some(ApiCoordinates { lat, lon }) = coordinates {
            Some(GeogPoint {
                x: lon,
                y: lat,
                srid: None,
            })
        } else {
            None
        },
    };

    // Create database entry
    // TODO: Error handling
    database.run(move |db| data::create_location(db, location)).await;
    log::info!("Created location for user {}", user.id);
    Status::Created
}

#[post("/locations", rank = 2)]
pub fn add_nologin() -> ApiError {
    ApiError::MissingAuthentication
}

#[post("/locations/<id>", data = "<data>")]
pub async fn edit(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
    data: Json<LocationCreateUpdateForm>,
) -> Result<Status, Status> {
    let user = user.into_inner();

    // Get location
    let mut location = match database.run(move |db| data::get_location_by_id(db, id)).await {
        Some(location) => location,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if location.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Update model
    let LocationCreateUpdateForm {
        name,
        country_code,
        elevation,
        coordinates,
    } = data.into_inner();
    location.name = name;
    location.country = country_code;
    location.elevation = elevation;
    if let Some(ApiCoordinates { lat, lon }) = coordinates {
        location.geog = Some(GeogPoint {
            x: lon,
            y: lat,
            srid: None,
        });
    } else {
        location.geog = None;
    }

    // Update database
    // TODO: Error handling
    database.run(move |db| data::update_location(db, &location)).await;

    // Render template
    Ok(Status::NoContent)
}

#[post("/locations/<id>", rank = 2)]
#[allow(unused_variables)]
pub fn edit_nologin(id: i32) -> ApiError {
    ApiError::MissingAuthentication
}

#[delete("/locations/<id>")]
pub async fn delete(user: auth::AuthUser, database: data::Database, id: i32) -> Result<Status, Status> {
    let user = user.into_inner();

    // Get data
    let location = match database
        .run(move |db| data::get_location_with_flight_count_by_id(db, id))
        .await
    {
        Some(location) => location,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if location.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Ensure that no related flights exist (otherwise a foreign key constraint
    // would be returned by the database when attempting to delete)
    if location.count > 0 {
        return Err(Status::Conflict);
    }

    // Delete database entry
    let location_id = location.id;
    database
        .run(move |db| data::delete_location_by_id(db, location_id))
        .await
        .map(|()| {
            log::info!("Deleted location with ID {}", location.id);
            Status::NoContent
        })
        .map_err(|e| {
            log::error!("Could not delete location with ID {}: {}", location.id, e);
            Status::InternalServerError
        })
}

#[delete("/locations/<id>", rank = 2)]
#[allow(unused_variables)]
pub fn delete_nologin(id: i32) -> ApiError {
    ApiError::MissingAuthentication
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
    ]
}

#[cfg(test)]
mod tests {
    use rocket::{self, http::ContentType, local::blocking::Client};

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
    fn list_locations() {
        let ctx = DbTestContext::new();
        let client = make_client();

        macro_rules! get_locations {
            ($cookie:expr) => {
                client
                    .get("/locations")
                    .private_cookie($cookie)
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }

        // No locations
        let g = data::get_locations_for_user(&mut *ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(g.len(), 0);

        // Empty list
        let resp = get_locations!(ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::Ok);
        let body = resp.into_string().expect("Response body wasn't valid text");
        assert_eq!(body, r#"{"locations":[]}"#);

        // Add locations
        data::create_location(
            &mut *ctx.force_get_conn(),
            NewLocation {
                name: "Misti".into(),
                country: "PE".into(),
                elevation: 5822,
                user_id: ctx.testuser1.user.id,
                geog: None,
            },
        );
        data::create_location(
            &mut *ctx.force_get_conn(),
            NewLocation {
                name: "Machu Picchu".into(),
                country: "PE".into(),
                elevation: 2430,
                user_id: ctx.testuser1.user.id,
                geog: Some(GeogPoint {
                    x: -72.54525463360524,
                    y: -13.163235172208347,
                    srid: None,
                }),
            },
        );

        // Query locations for user 1: Include both
        let resp = get_locations!(ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::Ok);
        let body = resp.into_string().expect("Response body wasn't valid text");
        assert_eq!(
            body,
            r#"{"locations":[{"id":2,"name":"Machu Picchu","countryCode":"PE","elevation":2430,"coordinates":{"lon":-72.54525463360524,"lat":-13.163235172208347},"flightCount":0},{"id":1,"name":"Misti","countryCode":"PE","elevation":5822,"flightCount":0}]}"#
        );

        // Query locations for user 2: Must be empty
        let resp = get_locations!(ctx.auth_cookie_user2());
        assert_eq!(resp.status(), Status::Ok);
        let body = resp.into_string().expect("Response body wasn't valid text");
        assert_eq!(body, r#"{"locations":[]}"#);

        // Without login
        let resp = client.get("/locations").dispatch();
        assert_eq!(resp.status(), Status::Unauthorized);
    }

    #[test]
    fn get_location() {
        let ctx = DbTestContext::new();
        let client = make_client();

        macro_rules! get_location {
            ($id:expr, $cookie:expr) => {
                client
                    .get(format!("/locations/{}", $id))
                    .private_cookie($cookie)
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }

        // Add location
        let location = data::create_location(
            &mut *ctx.force_get_conn(),
            NewLocation {
                name: "Machu Picchu".into(),
                country: "PE".into(),
                elevation: 2430,
                user_id: ctx.testuser1.user.id,
                geog: Some(GeogPoint {
                    x: -72.54525463360524,
                    y: -13.163235172208347,
                    srid: None,
                }),
            },
        );

        // Get location from user 1
        let resp = get_location!(location.id, ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::Ok);
        let body = resp.into_string().expect("Response body wasn't valid text");
        assert_eq!(
            body,
            r#"{"id":1,"name":"Machu Picchu","countryCode":"PE","elevation":2430,"coordinates":{"lon":-72.54525463360524,"lat":-13.163235172208347},"flightCount":0}"#
        );

        // Get location from user 2: Forbidden
        let resp = get_location!(location.id, ctx.auth_cookie_user2());
        assert_eq!(resp.status(), Status::Forbidden);

        // Get non-existing location: Not found
        let resp = get_location!(location.id + 9999, ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::NotFound);

        // Get location without login: Unauthorized
        let resp = client.get(format!("/locations/{}", location.id)).dispatch();
        assert_eq!(resp.status(), Status::Unauthorized);
    }

    #[test]
    fn add_location() {
        let ctx = DbTestContext::new();
        let client = make_client();

        // No locations
        let g = data::get_locations_for_user(&mut *ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(g.len(), 0);

        macro_rules! add_location {
            ($body:expr, $cookie:expr) => {
                client
                    .post("/locations")
                    .header(ContentType::Form)
                    .body($body)
                    .private_cookie($cookie)
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }

        // Add location
        let resp = add_location!(
            r#"{"name": "Testlocation", "countryCode": "CH", "elevation": -123}"#,
            ctx.auth_cookie_user1()
        );
        assert_eq!(resp.status(), Status::Created);

        // Verify database
        let g = data::get_locations_for_user(&mut *ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].name, "Testlocation");
        assert_eq!(g[0].country, "CH");
    }

    #[test]
    fn edit_location() {
        let ctx = DbTestContext::new();
        let client = make_client();

        macro_rules! update_location {
            ($id:expr, $body:expr, $cookie:expr) => {
                client
                    .post(format!("/locations/{}", $id))
                    .header(ContentType::JSON)
                    .body($body)
                    .private_cookie($cookie)
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }

        // Add location
        let location = data::create_location(
            &mut *ctx.force_get_conn(),
            NewLocation {
                name: "Machu Picchu".into(),
                country: "PE".into(),
                elevation: 2430,
                user_id: ctx.testuser1.user.id,
                geog: Some(GeogPoint {
                    x: -72.54525463360524,
                    y: -13.163235172208347,
                    srid: None,
                }),
            },
        );

        // The update JSON. Note: The "id" field must be ignored by the API endpoint!
        let update_json = r#"{
            "id": 9876,
            "name": "Machu Picchu 2",
            "countryCode": "XX",
            "elevation": 999
        }"#;

        // Update location from user 1
        let resp = update_location!(location.id, update_json, ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::NoContent);
        let updated_location =
            data::get_location_by_id(&mut *ctx.force_get_conn(), location.id).expect("Location not found");
        assert_eq!(updated_location.name, "Machu Picchu 2");
        assert_eq!(updated_location.country, "XX");
        assert_eq!(updated_location.elevation, 999);
        assert_eq!(updated_location.geog, None);

        // Update location from user 2: Forbidden
        let resp = update_location!(location.id, update_json, ctx.auth_cookie_user2());
        assert_eq!(resp.status(), Status::Forbidden);

        // Update non-existing location: Not found
        let resp = update_location!(location.id + 9999, update_json, ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::NotFound);
    }

    #[test]
    fn delete_location() {
        let ctx = DbTestContext::new();
        let client = make_client();

        macro_rules! delete_location {
            ($id:expr, $cookie:expr) => {
                client
                    .delete(format!("/locations/{}", $id))
                    .private_cookie($cookie)
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }

        // Add locations
        let location1 = data::create_location(
            &mut *ctx.force_get_conn(),
            NewLocation {
                name: "Machu Picchu".into(),
                country: "PE".into(),
                elevation: 2430,
                user_id: ctx.testuser1.user.id,
                geog: Some(GeogPoint {
                    x: -72.54525463360524,
                    y: -13.163235172208347,
                    srid: None,
                }),
            },
        );
        let location2 = data::create_location(
            &mut *ctx.force_get_conn(),
            NewLocation {
                name: "Misti".into(),
                country: "PE".into(),
                elevation: 5822,
                user_id: ctx.testuser1.user.id,
                geog: None,
            },
        );

        // Add flight to location 2
        data::create_flight(
            &mut *ctx.force_get_conn(),
            &NewFlight {
                number: Some(1),
                user_id: ctx.testuser1.user.id,
                launch_at: Some(location2.id),
                ..Default::default()
            },
            None,
        );

        // Delete locations from user 2: Forbidden
        for location_id in [location1.id, location2.id] {
            let resp = delete_location!(location_id, ctx.auth_cookie_user2());
            assert_eq!(resp.status(), Status::Forbidden);
        }

        // Delete location 1 from user 1: OK
        let resp = delete_location!(location1.id, ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::NoContent);

        // Delete location 2 from user 1: Conflict (existing flights)
        let resp = delete_location!(location2.id, ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::Conflict);

        // Delete non-existing location: Not found
        let resp = delete_location!(location2.id + 9999, ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::NotFound);
    }
}
