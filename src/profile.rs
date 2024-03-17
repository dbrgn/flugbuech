//! Profile views.

use chrono::{DateTime, Utc};
use rocket::{get, routes, serde::json::Json, Route};
use serde::Serialize;

use crate::{auth, responders::ApiError};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiProfile {
    username: String,
    email: String,
    signed_up: DateTime<Utc>,
    news_opt_in: bool,
}

#[get("/profile")]
pub fn get(user: auth::AuthUser) -> Json<ApiProfile> {
    let user = user.into_inner();
    Json(ApiProfile {
        username: user.username,
        email: user.email,
        signed_up: user.signed_up,
        news_opt_in: user.news_opt_in,
    })
}

#[get("/profile", rank = 2)]
pub fn get_nologin() -> ApiError {
    ApiError::MissingAuthentication
}

/// Return vec of all API routes.
pub fn api_routes() -> Vec<Route> {
    routes![get, get_nologin]
}
