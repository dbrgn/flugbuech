//! Location views.

use std::convert::TryFrom;

use diesel_geography::types::GeogPoint;
use rocket::{
    get,
    http::Status,
    post,
    response::{Flash, Redirect},
    serde::json::Json,
    uri,
};
use rocket_dyn_templates::Template;
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
pub struct LocationCreateForm {
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

#[post("/locations", data = "<data>")]
pub async fn add(user: auth::AuthUser, database: data::Database, data: Json<LocationCreateForm>) -> Status {
    log::debug!("locations::add");
    let user = user.into_inner();

    // Unwrap form data
    let LocationCreateForm {
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
    ApiError::Authentication
}

// Views

#[get("/locations/<id>/edit")]
pub async fn edit_form(user: auth::AuthUser, database: data::Database, id: i32) -> Result<Template, Status> {
    let user = user.into_inner();

    // Get location
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

    // Render template
    let context = LocationContext { user, location };
    Ok(Template::render("location_edit", &context))
}

//#[post("/locations/<id>/edit", data = "<data>")]
//pub async fn edit(
//    user: auth::AuthUser,
//    database: data::Database,
//    id: i32,
//    data: Form<LocationCreateForm>,
//) -> Result<Redirect, Status> {
//    let user = user.into_inner();
//
//    // Get location
//    let mut location = match database.run(move |db| data::get_location_by_id(db, id)).await {
//        Some(location) => location,
//        None => return Err(Status::NotFound),
//    };
//
//    // Ownership check
//    if location.user_id != user.id {
//        return Err(Status::Forbidden);
//    }
//
//    // Update model
//    let LocationCreateForm {
//        name,
//        country_code,
//        elevation,
//        coordinates,
//    } = data.into_inner();
//    location.name = name;
//    location.country = country_code;
//    location.elevation = elevation;
//    if let Some(ApiCoordinates { lat, lon }) = coordinates {
//        location.geog = Some(GeogPoint {
//            x: lon,
//            y: lat,
//            srid: None,
//        });
//    } else {
//        location.geog = None;
//    }
//
//    // Update database
//    // TODO: Error handling
//    database.run(move |db| data::update_location(db, &location)).await;
//
//    // Render template
//    Ok(Redirect::to(uri!(list)))
//}

#[get("/locations/<id>/delete")]
pub async fn delete_form(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
) -> Result<Template, Status> {
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

    // Render template
    let context = LocationContext { user, location };
    Ok(Template::render("location_delete", context))
}

#[post("/locations/<id>/delete")]
pub async fn delete(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
) -> Result<Flash<Redirect>, Status> {
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
            Flash::success(
                Redirect::to(uri!(list)),
                format!("Location \"{}\" deleted", location.name),
            )
        })
        .or_else(|e| {
            log::error!("Could not delete location with ID {}: {}", location.id, e);
            Ok(Flash::error(
                Redirect::to(uri!(list)),
                format!("Could not delete location \"{}\"", location.name),
            ))
        })
}

#[cfg(test)]
mod tests {
    use rocket::{self, http::ContentType, local::blocking::Client, routes};

    use crate::{
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
            .mount("/", routes![list, get, add]);
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
}
