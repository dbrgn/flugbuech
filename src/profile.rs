//! Profile views.

use chrono::{DateTime, Utc};
use log::error;
use rocket::{get, http::Status, post, routes, serde::json::Json, Route};
use serde::{Deserialize, Serialize};

use crate::{auth, data, responders::ApiError};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiProfile {
    username: String,
    email: String,
    signed_up: DateTime<Utc>,
    news_opt_in: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiProfileUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    news_opt_in: Option<bool>,
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

#[post("/profile", data = "<data>")]
pub async fn edit(database: data::Database, user: auth::AuthUser, data: Json<ApiProfileUpdate>) -> Status {
    let user = user.into_inner();
    if let Some(news_opt_in) = data.news_opt_in {
        if let Err(e) = database
            .run(move |db| data::update_news_opt_in(db, &user, news_opt_in))
            .await
        {
            error!("Updating user's news opt-in failed: {e}");
            return Status::InternalServerError;
        }
    }
    Status::NoContent
}

#[post("/profile", rank = 2)]
#[allow(unused_variables)]
pub fn edit_nologin() -> ApiError {
    ApiError::MissingAuthentication
}

/// Return vec of all API routes.
pub fn api_routes() -> Vec<Route> {
    routes![get, get_nologin, edit, edit_nologin]
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use diesel::PgConnection;
    use rocket::{
        self,
        http::{ContentType, Status},
        local::blocking::Client,
        serde::json,
    };

    use crate::{
        data,
        test_utils::{make_test_config, DbTestContext},
    };

    use super::*;

    /// Create a new test client with cookie tracking.
    fn make_api_client() -> Client {
        let app = rocket::custom(make_test_config())
            .attach(data::Database::fairing())
            .mount("/", api_routes());
        Client::tracked(app).expect("valid rocket instance")
    }

    #[test]
    fn update_news_opt_in() {
        let ctx = DbTestContext::new();
        let client = make_api_client();

        /// Helper function: Assert news opt-in state
        fn assert_news_opt_in(conn_mutex: &Mutex<PgConnection>, user_id: i32, opted_in: bool) {
            let mut conn = conn_mutex.lock().unwrap();
            let user = data::get_user(&mut conn, user_id).expect("User not found");
            assert_eq!(user.news_opt_in, opted_in);
        }

        /// Helper macro: Send request
        macro_rules! update_profile_news_opt_in {
            ($opt_in:expr) => {
                client
                    .post("/profile")
                    .header(ContentType::JSON)
                    .body(
                        json::to_string(&ApiProfileUpdate {
                            news_opt_in: Some($opt_in),
                        })
                        .unwrap(),
                    )
                    .private_cookie(ctx.auth_cookie_user1())
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }

        // Initially, not opted in to news
        assert_news_opt_in(&ctx.conn, ctx.testuser1.user.id, false);

        // Opt in
        let resp1 = update_profile_news_opt_in!(true);
        assert_eq!(resp1.status(), Status::NoContent);
        assert_news_opt_in(&ctx.conn, ctx.testuser1.user.id, true);

        // Opt out again
        let resp2 = update_profile_news_opt_in!(false);
        assert_eq!(resp2.status(), Status::NoContent);
        assert_news_opt_in(&ctx.conn, ctx.testuser1.user.id, false);

        // Change is idempotent
        let resp3 = update_profile_news_opt_in!(false);
        assert_eq!(resp3.status(), Status::NoContent);
        assert_news_opt_in(&ctx.conn, ctx.testuser1.user.id, false);
    }
}
