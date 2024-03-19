//! Glider views.

use chrono::NaiveDate;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use rocket::{delete, error, get, http::Status, post, routes, serde::json::Json, Route};
use serde::{Deserialize, Serialize};

use crate::{
    auth, data,
    models::{GliderWithStats, NewGlider},
    responders::{ApiError, RocketError},
};

// Forms

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GliderCreateUpdateForm {
    manufacturer: String,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<String>, // ISO date (e.g. 2010-11-30) // TODO: Use OptionResult<NaiveDate>
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<String>, // ISO date (e.g. 2010-11-30) // TODO: Use OptionResult<NaiveDate>
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cost: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
}

// API types

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiGlider {
    id: i32,
    manufacturer: String,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<String>, // ISO date (e.g. 2010-11-30) // TODO: Use OptionResult<NaiveDate>
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<String>, // ISO date (e.g. 2010-11-30) // TODO: Use OptionResult<NaiveDate>
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cost: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    stats: ApiGliderStats,
}

impl From<GliderWithStats> for ApiGlider {
    fn from(glider: GliderWithStats) -> Self {
        ApiGlider {
            id: glider.id,
            manufacturer: glider.manufacturer,
            model: glider.model,
            since: glider.since.map(|date| date.to_string()),
            until: glider.until.map(|date| date.to_string()),
            source: glider.source,
            cost: glider.cost,
            comment: glider.comment,
            stats: ApiGliderStats {
                flights: glider.flights,
                seconds: glider.seconds,
                seconds_complete: glider.seconds_complete,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiGliderStats {
    flights: i64,
    seconds: i64,
    seconds_complete: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGliders {
    /// List of user's gliders.
    gliders: Vec<ApiGlider>,
    /// The user's last used glider ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    last_glider_id: Option<i32>,
}

// API endpoints

#[get("/gliders")]
pub async fn list(database: data::Database, user: auth::AuthUser) -> Json<ApiGliders> {
    let user = user.into_inner();

    // Get all gliders for user
    let gliders: Vec<ApiGlider> = database
        .run({
            let user = user.clone();
            move |db| data::get_gliders_with_stats_for_user(db, &user)
        })
        .await
        .into_iter()
        .map(Into::into)
        .collect();

    Json(ApiGliders {
        gliders,
        last_glider_id: user.last_glider_id,
    })
}

#[get("/gliders", rank = 2)]
pub fn list_nologin() -> ApiError {
    ApiError::MissingAuthentication
}

#[post("/gliders", data = "<data>")]
pub async fn add(
    user: auth::AuthUser,
    database: data::Database,
    data: Json<GliderCreateUpdateForm>,
) -> Result<Status, (Status, Json<RocketError>)> {
    log::debug!("gliders::add");
    let user = user.into_inner();

    // Destructure data
    let GliderCreateUpdateForm {
        manufacturer,
        model,
        since,
        until,
        source,
        cost,
        comment,
    } = data.into_inner();

    // Create model
    let glider = NewGlider {
        user_id: user.id,
        manufacturer,
        model,
        since: since.and_then(|strval| NaiveDate::parse_from_str(&strval, "%Y-%m-%d").ok()),
        until: until.and_then(|strval| NaiveDate::parse_from_str(&strval, "%Y-%m-%d").ok()),
        source,
        cost,
        comment,
    };

    // Create database entry
    match database.run(move |db| data::create_glider(db, glider)).await {
        Ok(_) => {
            // Glider created
            log::info!("Created glider for user {}", user.id);
            Ok(Status::Created)
        }
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _info)) => Err(RocketError::new(
            Status::Conflict,
            "Conflict",
            "You can't add the same glider twice.",
        )),
        Err(other) => {
            error!("Could not create glider: {:?}", other);
            Err(RocketError::new(
                Status::InternalServerError,
                "InternalServerError",
                "Internal server error",
            ))
        }
    }
}

#[post("/gliders", rank = 2)]
pub fn add_nologin() -> ApiError {
    ApiError::MissingAuthentication
}

#[post("/gliders/<id>", data = "<data>")]
pub async fn edit(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
    data: Json<GliderCreateUpdateForm>,
) -> Result<Status, Status> {
    let user = user.into_inner();

    // Get glider
    let mut glider = database
        .run(move |db| data::get_glider_by_id(db, id))
        .await
        .ok_or(Status::NotFound)?;

    // Ownership check
    if glider.user_id != user.id {
        // TODO: Don't leak this information, use ApiError::NotFound instead
        return Err(Status::Forbidden);
    }

    // Update model
    let GliderCreateUpdateForm {
        manufacturer,
        model,
        since,
        until,
        source,
        cost,
        comment,
    } = data.into_inner();
    glider.manufacturer = manufacturer;
    glider.model = model;
    glider.since = since.and_then(|strval| NaiveDate::parse_from_str(&strval, "%Y-%m-%d").ok());
    glider.until = until.and_then(|strval| NaiveDate::parse_from_str(&strval, "%Y-%m-%d").ok());
    glider.source = source;
    glider.cost = cost;
    glider.comment = comment;

    // Update database
    database.run(move |db| data::update_glider(db, &glider)).await;
    Ok(Status::NoContent)
}

#[post("/gliders/<id>", rank = 2)]
#[allow(unused_variables)]
pub fn edit_nologin(id: i32) -> ApiError {
    ApiError::MissingAuthentication
}

#[delete("/gliders/<id>")]
pub async fn delete(user: auth::AuthUser, database: data::Database, id: i32) -> Result<Status, Status> {
    let user = user.into_inner();

    // Get data
    let glider = match database
        .run(move |db| data::get_glider_with_stats_by_id(db, id))
        .await
    {
        Some(location) => location,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if glider.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Ensure that no related flights exist
    if glider.flights > 0 {
        return Err(Status::Conflict);
    }

    // Delete database entry
    database
        .run(move |db| data::delete_glider_by_id(db, id))
        .await
        .map(|()| {
            log::info!("Deleted glider with ID {}", id);
            Status::NoContent
        })
        .map_err(|e| {
            log::error!("Could not delete glider with ID {}: {}", id, e);
            Status::InternalServerError
        })
}

#[delete("/gliders/<id>", rank = 2)]
#[allow(unused_variables)]
pub fn delete_nologin(id: i32) -> ApiError {
    ApiError::MissingAuthentication
}

/// Return vec of all API routes.
pub fn api_routes() -> Vec<Route> {
    routes![
        list,
        list_nologin,
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
    use std::time::Duration;

    use chrono::{NaiveDate, Utc};
    use rocket::{self, http::ContentType, local::blocking::Client};

    use crate::{
        models::NewFlight,
        test_utils::{make_test_config, DbTestContext},
    };

    use super::*;

    /// Create a new test client. Cookie tracking is disabled.
    fn make_client() -> Client {
        let app = rocket::custom(make_test_config())
            .attach(data::Database::fairing())
            .mount("/", api_routes());
        Client::untracked(app).expect("valid rocket instance")
    }

    #[test]
    fn add_glider_constraints() {
        let ctx = DbTestContext::new();
        let client = make_client();

        // No gliders
        let g = data::get_gliders_for_user(&mut *ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(g.len(), 0);

        macro_rules! add_glider {
            ($body:expr, $cookie:expr) => {
                client
                    .post("/gliders")
                    .header(ContentType::JSON)
                    .body($body)
                    .private_cookie($cookie)
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }

        // Add glider
        let resp = add_glider!(
            r#"{"manufacturer": "Advance", "model": "Epsilon 8"}"#,
            ctx.auth_cookie_user1()
        );
        assert_eq!(resp.status(), Status::Created);

        // Verify database
        let g = data::get_gliders_for_user(&mut *ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].manufacturer, "Advance");
        assert_eq!(g[0].model, "Epsilon 8");

        // Cannot add a glider twice
        let resp = add_glider!(
            r#"{"manufacturer": "Advance", "model": "Epsilon 8"}"#,
            ctx.auth_cookie_user1()
        );
        assert_eq!(resp.status(), Status::Conflict);
        assert_eq!(
            data::get_gliders_for_user(&mut *ctx.force_get_conn(), &ctx.testuser1.user).len(),
            1,
        );

        // ...but another user can!
        assert_eq!(
            data::get_gliders_for_user(&mut *ctx.force_get_conn(), &ctx.testuser2.user).len(),
            0,
        );
        let resp = add_glider!(
            r#"{"manufacturer": "Advance", "model": "Epsilon 8"}"#,
            ctx.auth_cookie_user2()
        );
        assert_eq!(resp.status(), Status::Created);
        assert_eq!(
            data::get_gliders_for_user(&mut *ctx.force_get_conn(), &ctx.testuser2.user).len(),
            1,
        );
    }

    #[test]
    fn add_glider_data() {
        let ctx = DbTestContext::new();
        let client = make_client();

        // No gliders
        let g = data::get_gliders_for_user(&mut *ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(g.len(), 0);

        macro_rules! add_glider {
            ($body:expr, $cookie:expr) => {
                client
                    .post("/gliders")
                    .header(ContentType::JSON)
                    .body($body)
                    .private_cookie($cookie)
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }

        // Add glider
        let resp = add_glider!(
            r#"{"manufacturer": "Ozone", "model": "Enzo 2", "since": "2019-02-03", "until": "2019-11-20", "source": "Flycenter", "cost": 3344, "comment": "Sold it to Peter."}"#,
            ctx.auth_cookie_user1()
        );
        assert_eq!(resp.status(), Status::Created);

        // Verify database
        let g = data::get_gliders_for_user(&mut *ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].manufacturer, "Ozone");
        assert_eq!(g[0].model, "Enzo 2");
        assert_eq!(g[0].since, Some(NaiveDate::from_ymd_opt(2019, 2, 3).unwrap()));
        assert_eq!(g[0].until, Some(NaiveDate::from_ymd_opt(2019, 11, 20).unwrap()));
        assert_eq!(g[0].source, Some("Flycenter".into()));
        assert_eq!(g[0].cost, Some(3344));
        assert_eq!(g[0].comment, Some("Sold it to Peter.".into()));
    }

    #[test]
    fn get_gliders_with_stats() {
        let ctx = DbTestContext::new();
        let client = make_client();

        macro_rules! add_glider {
            ($body:expr, $cookie:expr) => {
                client
                    .post("/gliders")
                    .header(ContentType::JSON)
                    .body($body)
                    .private_cookie($cookie)
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }
        macro_rules! get_gliders {
            ($cookie:expr) => {
                client
                    .get("/gliders")
                    .private_cookie($cookie)
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }

        // No gliders
        let resp = get_gliders!(ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(resp.into_json::<ApiGliders>().unwrap().gliders.len(), 0);

        // Add gliders
        let resp = add_glider!(r#"{"manufacturer": "A", "model": "1"}"#, ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::Created);

        // One glider, no flights
        let resp = get_gliders!(ctx.auth_cookie_user1());
        let gliders = resp.into_json::<ApiGliders>().unwrap().gliders;
        assert_eq!(gliders.len(), 1);
        assert_eq!(gliders[0].manufacturer, "A");
        assert_eq!(gliders[0].model, "1");
        assert_eq!(gliders[0].stats.flights, 0);
        assert_eq!(gliders[0].stats.seconds, 0);
        assert_eq!(gliders[0].stats.seconds_complete, true);

        // Add two flights with launch/landing time
        data::create_flight(
            &mut *ctx.force_get_conn(),
            &NewFlight {
                user_id: ctx.testuser1.user.id,
                ..Default::default()
            },
            None,
        );
        let t1 = Utc::now() - Duration::from_secs(3600);
        for i in [0, 1] {
            data::create_flight(
                &mut *ctx.force_get_conn(),
                &NewFlight {
                    user_id: ctx.testuser1.user.id,
                    glider_id: Some(gliders[0].id),
                    launch_time: Some(t1 + Duration::from_secs(i * 600)),
                    landing_time: Some(t1 + Duration::from_secs(i * 600 + 400)),
                    ..Default::default()
                },
                None,
            );
        }
        let resp = get_gliders!(ctx.auth_cookie_user1());
        let gliders = resp.into_json::<ApiGliders>().unwrap().gliders;
        assert_eq!(gliders.len(), 1);
        assert_eq!(gliders[0].stats.flights, 2);
        assert_eq!(gliders[0].stats.seconds, 800);
        assert_eq!(gliders[0].stats.seconds_complete, true);

        // Add a flight without launch/landing time
        data::create_flight(
            &mut *ctx.force_get_conn(),
            &NewFlight {
                user_id: ctx.testuser1.user.id,
                glider_id: Some(gliders[0].id),
                ..Default::default()
            },
            None,
        );
        let resp = get_gliders!(ctx.auth_cookie_user1());
        let gliders = resp.into_json::<ApiGliders>().unwrap().gliders;
        assert_eq!(gliders.len(), 1);
        assert_eq!(gliders[0].stats.flights, 3);
        assert_eq!(gliders[0].stats.seconds, 800);
        assert_eq!(gliders[0].stats.seconds_complete, false);
    }
}
