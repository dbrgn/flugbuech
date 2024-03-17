//! Authentication and authorization related functionality.

use std::sync::Arc;

use log::{error, warn};
use rocket::{
    http::{Cookie, CookieJar, SameSite, Status},
    outcome::Outcome,
    post,
    request::{self, FromRequest, Request},
    routes,
    serde::json::Json,
    time::{Duration, OffsetDateTime},
    Route,
};
use serde::{Deserialize, Serialize};

use crate::{
    data::{self, Database},
    models::User,
    responders::{ApiError, RocketError},
};

pub const USER_COOKIE_ID: &str = "user_id";
pub const USER_COOKIE_NAME: &str = "user_name";

// Forms

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    username: String,
    email: String,
    password: String,
    news_opt_in: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PasswordChange {
    current_password: String,
    new_password: String,
}

/// User newtype, wraps the user model, provides guard transparency.
#[derive(Debug)]
pub struct AuthUser(User);

fn make_cookie(name: &'static str, value: String) -> Cookie {
    // Cookie expiration: 1 year
    let mut expiration = OffsetDateTime::now_utc();
    expiration += Duration::weeks(52);

    // Create cookie
    let mut cookie = Cookie::new(name, value);
    cookie.set_expires(expiration);
    cookie.set_same_site(SameSite::Lax);

    cookie
}

/// Add auth cookies to the specified cookie jar.
fn add_auth_cookies(cookies: &CookieJar, user: &User) {
    cookies.add_private(make_cookie(USER_COOKIE_ID, user.id.to_string()));
    cookies.add(make_cookie(USER_COOKIE_NAME, user.username.clone()));
}

/// Remove auth cookies from the specified cookie jar.
fn remove_auth_cookies(cookies: &CookieJar) {
    cookies.remove_private(Cookie::from(USER_COOKIE_ID));
    cookies.remove(Cookie::from(USER_COOKIE_NAME));
}

/// Get the user model from a request cookie.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // Get database access
        let database = match Database::from_request(request).await {
            Outcome::Error(f) => return Outcome::Error(f),
            Outcome::Forward(f) => return Outcome::Forward(f),
            Outcome::Success(database) => database,
        };

        // Look up login cookie
        let cookies = request.cookies();
        let user_cookie = match cookies.get_private(USER_COOKIE_ID) {
            Some(cookie) => cookie,
            None => return Outcome::Forward(Status::Unauthorized),
        };

        // Extract user ID
        let user_id: i32 = match user_cookie.value().parse() {
            Ok(int) => int,
            Err(_) => return Outcome::Forward(Status::Unauthorized),
        };

        // Ensure that username cookie is set as well
        if cookies.get(USER_COOKIE_NAME).is_none() {
            error!("Login cookie but no username cookie found. Removing auth cookies.");
            remove_auth_cookies(cookies);
            return Outcome::Forward(Status::Unauthorized);
        }

        // A login cookie was found. Look up the corresponding database user.
        match database.run(move |db| data::get_user(db, user_id)).await {
            Some(user) => Outcome::Success(AuthUser(user)),
            None => {
                error!("Login cookie with invalid user id found. Removing cookie.");
                remove_auth_cookies(cookies);
                Outcome::Forward(Status::Unauthorized)
            }
        }
    }
}

impl AuthUser {
    /// Convert this guard type into the inner user model.
    pub fn into_inner(self) -> User {
        self.0
    }
}

// API views

/// Login handler.
///
/// - Return "HTTP 204 No Content" if login was successful.
/// - Return "HTTP 400 Bad Request" if request was malformed.
/// - Return "HTTP 403 Forbidden" if username or password were wrong.
#[post("/auth/login", data = "<login>")]
pub async fn login(
    cookies: &CookieJar<'_>,
    database: Database,
    login: Json<Login>,
) -> Result<Status, (Status, Json<RocketError>)> {
    let username = login.username.clone();
    match database
        .run(move |db| data::validate_login(db, &login.username, &login.password))
        .await
    {
        Some(user) => {
            // Success, add auth cookies
            add_auth_cookies(cookies, &user);
            Ok(Status::NoContent)
        }
        None => {
            // Login failed
            warn!("Login failed for user {}", &username);
            Err(RocketError::new(
                Status::Forbidden,
                "BadCredentials",
                "Wrong username or password",
            ))
        }
    }
}

/// Logout handler.
#[post("/auth/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Status {
    remove_auth_cookies(cookies);
    Status::NoContent
}

/// Registration handler
///
/// - Return "HTTP 204 No Content" if registration was successful.
/// - Return "HTTP 422 Unprocessable Entity" if registration data was invalid.
#[post("/auth/registration", data = "<registration>")]
pub async fn registration(
    cookies: &CookieJar<'_>,
    database: Database,
    registration: Json<Registration>,
) -> Result<Status, (Status, Json<RocketError>)> {
    // TODO: Transation for registration

    let registration_clone = registration.clone();
    let registration_result = database
        .run(move |db| {
            data::validate_registration(
                db,
                &registration_clone.username,
                &registration_clone.email,
                &registration_clone.password,
            )
        })
        .await;

    match registration_result {
        Ok(_) => {
            let new_user = database
                .run(move |db| {
                    data::create_user(
                        db,
                        &registration.username,
                        &registration.email,
                        &registration.password,
                        registration.news_opt_in,
                    )
                })
                .await;
            add_auth_cookies(cookies, &new_user);
            Ok(Status::NoContent)
        }
        Err(data::RegistrationError::NonUniqueUsername) => Err(RocketError::new(
            Status::UnprocessableEntity,
            "invalid-username",
            format!(
                "Invalid username ({}): Invalid or already taken",
                registration.username,
            ),
        )),
        Err(data::RegistrationError::InvalidEmail) => Err(RocketError::new(
            Status::UnprocessableEntity,
            "invalid-email",
            format!("Invalid e-mail address ({})", registration.email),
        )),
        Err(data::RegistrationError::InvalidPasswordFormat) => Err(RocketError::new(
            Status::UnprocessableEntity,
            "invalid-password",
            "Invalid password: Too short",
        )),
    }
}

/// Password change handler
///
/// - Return "HTTP 204 No Content" if password change was successful.
/// - Return "HTTP 422 Unprocessable Entity" if password change data was invalid.
#[post("/auth/password/change", data = "<password_change>")]
pub async fn password_change(
    user: AuthUser,
    database: Database,
    password_change: Json<PasswordChange>,
) -> Result<Status, (Status, Json<RocketError>)> {
    let user = Arc::new(user.into_inner());

    macro_rules! fail {
        ($msg:expr) => {{
            log::error!("Could not change password for user {}: {}", user.id, $msg);
            return Err(RocketError::new(
                Status::UnprocessableEntity,
                "PasswordChangeError",
                $msg,
            ));
        }};
    }

    // Ensure minimum length of new password
    if password_change.new_password.len() < data::MIN_PASSWORD_LENGTH {
        fail!("New password must have at least 8 characters");
    }

    // Unwrap password fields
    let PasswordChange {
        current_password,
        new_password,
    } = password_change.into_inner();

    // Verify current password
    let user_clone = user.clone();
    match database
        .run(move |db| data::validate_login(db, &user_clone.username, &current_password))
        .await
    {
        Some(u) if u.id == user.id => { /* Valid password */ }
        Some(_) => fail!("Invalid user"),
        None => fail!("Invalid current password"),
    }

    // Update password
    database
        .run(move |db| data::update_password(db, &user, &new_password))
        .await;
    Ok(Status::NoContent)
}

#[post("/auth/password/change", rank = 2)]
pub fn password_change_nologin() -> ApiError {
    ApiError::MissingAuthentication
}

/// Return vec of all API routes.
pub fn api_routes() -> Vec<Route> {
    routes![
        login,
        logout,
        registration,
        password_change,
        password_change_nologin,
    ]
}

#[cfg(test)]
mod tests {
    use rocket::{
        self,
        http::{ContentType, Status},
        local::blocking::{Client, LocalResponse},
        serde::json,
    };

    use crate::test_utils::{make_test_config, DbTestContext};

    use super::*;

    /// Create a new test client with cookie tracking.
    fn make_api_client() -> Client {
        let app = rocket::custom(make_test_config())
            .attach(data::Database::fairing())
            .mount("/", api_routes());
        Client::tracked(app).expect("valid rocket instance")
    }

    #[test]
    fn login_wrong() {
        let ctx = DbTestContext::new();
        let client = make_api_client();
        let resp = client
            .post("/auth/login")
            .header(ContentType::JSON)
            .body(
                json::to_string(&Login {
                    username: ctx.testuser1.user.username,
                    password: "WRONG".to_string(),
                })
                .unwrap(),
            )
            .dispatch();

        // Login wrong: No cookies, error response
        assert_eq!(resp.cookies().get_private(USER_COOKIE_ID), None);
        assert_eq!(resp.cookies().get(USER_COOKIE_NAME), None);
        assert_eq!(resp.status(), Status::Forbidden);
        assert_eq!(
            resp.into_string().unwrap(),
            r#"{"error":{"code":403,"reason":"BadCredentials","description":"Wrong username or password"}}"#
        );
    }

    #[test]
    fn login_success() {
        let ctx = DbTestContext::new();
        let client = make_api_client();
        let resp = client
            .post("/auth/login")
            .header(ContentType::JSON)
            .body(
                json::to_string(&Login {
                    username: ctx.testuser1.user.username,
                    password: ctx.testuser1.password,
                })
                .unwrap(),
            )
            .dispatch();

        // Login successful: Correct status, cookies set
        assert_eq!(
            resp.status(),
            Status::NoContent,
            "Body: {}",
            resp.into_string().unwrap()
        );
        assert!(resp.cookies().get_private(USER_COOKIE_ID).is_some());
        assert!(resp.cookies().get(USER_COOKIE_NAME).is_some());
    }

    /// Request a password change, return the redirect response.
    fn password_change_request<'a>(
        client: &'a mut Client,
        ctx: &DbTestContext,
        current: &str,
        new: &str,
    ) -> LocalResponse<'a> {
        client
            .post("/auth/password/change")
            .header(ContentType::JSON)
            .body(
                json::to_string(&PasswordChange {
                    current_password: current.into(),
                    new_password: new.into(),
                })
                .unwrap(),
            )
            .private_cookie(ctx.auth_cookie_user1())
            .cookie(ctx.username_cookie())
            .dispatch()
    }

    #[test]
    fn password_change() {
        let ctx = DbTestContext::new();
        let mut client = make_api_client();

        macro_rules! password_valid {
            ($pw:expr) => {
                data::validate_login(
                    &mut *ctx.force_get_conn(),
                    &ctx.testuser1.user.username,
                    $pw,
                )
                .is_some()
            };
        }

        // Validate old password
        assert!(
            password_valid!(&ctx.testuser1.password),
            "Test user password does not work"
        );

        // Password too short
        let res = password_change_request(&mut client, &ctx, &ctx.testuser1.password, "abc");
        assert_eq!(res.status(), Status::UnprocessableEntity);
        assert!(res
            .into_string()
            .unwrap()
            .contains("New password must have at least 8 characters"));

        // Old password is wrong
        let res = password_change_request(&mut client, &ctx, "iawieoioijf", "abcdefgh");
        assert_eq!(res.status(), Status::UnprocessableEntity);
        assert!(res.into_string().unwrap().contains("Invalid current password"));

        // Ensure password wasn't changed
        assert!(
            password_valid!(&ctx.testuser1.password),
            "Test user password was changed"
        );

        // Successful password change
        let res = password_change_request(&mut client, &ctx, &ctx.testuser1.password, "abcdefgh");
        assert_eq!(res.status(), Status::NoContent);

        // Ensure password was changed
        assert!(
            !password_valid!(&ctx.testuser1.password),
            "Test user password still works"
        );
        assert!(password_valid!("abcdefgh"), "New password doesn't work");
    }
}
