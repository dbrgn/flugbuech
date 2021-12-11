//! Authentication and authorization related functionality.

use std::sync::Arc;

use log::error;
use rocket::{
    form::Form,
    get,
    http::{Cookie, CookieJar},
    outcome::Outcome,
    post,
    request::{self, FlashMessage, FromRequest, Request},
    response::{Flash, Redirect},
    routes, uri, FromForm, Route,
};
use rocket_dyn_templates::Template;
use serde::Serialize;

use crate::{
    data::{self, Database},
    flash::context_from_flash_opt,
    models::User,
};

pub const USER_COOKIE_ID: &str = "user_id";

#[derive(FromForm)]
pub struct Login {
    username: String,
    password: String,
}

/// User newtype, wraps the user model, provides guard transparency.
#[derive(Debug)]
pub struct AuthUser(User);

/// Get the user model from a request cookie.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // Get database access
        let database = match Database::from_request(request).await {
            Outcome::Failure(f) => return Outcome::Failure(f),
            Outcome::Forward(f) => return Outcome::Forward(f),
            Outcome::Success(database) => database,
        };

        // Get login cookie
        let cookies = request.cookies();
        let user_cookie = match cookies.get_private(USER_COOKIE_ID) {
            Some(cookie) => cookie,
            None => return Outcome::Forward(()),
        };

        // A login cookie was found. Look up the corresponding database user.
        let id: i32 = match user_cookie.value().parse() {
            Ok(int) => int,
            Err(_) => return Outcome::Forward(()),
        };
        match database.run(move |db| data::get_user(db, id)).await {
            Some(user) => Outcome::Success(AuthUser(user)),
            None => {
                error!("Login cookie with invalid user id found. Removing cookie.");
                cookies.remove_private(user_cookie);
                Outcome::Forward(())
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

/// Login handler.
#[post("/auth/login", data = "<login>")]
pub async fn login(
    cookies: &CookieJar<'_>,
    login: Form<Login>,
    database: Database,
) -> Result<Redirect, Flash<Redirect>> {
    match database
        .run(move |db| data::validate_login(db, &login.username, &login.password))
        .await
    {
        Some(user) => {
            cookies.add_private(Cookie::new(USER_COOKIE_ID, user.id.to_string()));
            Ok(Redirect::to("/"))
        }
        None => Err(Flash::error(
            Redirect::to(uri!(login_page)),
            "Invalid username/password.",
        )),
    }
}

/// Logout handler.
#[get("/auth/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove_private(Cookie::named(USER_COOKIE_ID));
    Redirect::to("/")
}

/// Redirect requests to login page if user is already logged in.
#[get("/auth/login")]
pub fn login_user(_user: AuthUser) -> Redirect {
    Redirect::to("/")
}

/// Show the login page (with flash messages) if not already logged in.
#[get("/auth/login", rank = 2)]
pub fn login_page(flash: Option<FlashMessage>) -> Template {
    Template::render("login", &context_from_flash_opt(flash))
}

#[derive(FromForm, Clone)]
pub struct Registration {
    username: String,
    email: String,
    password: String,
    password_confirmation: String,
}

/// Redirect requests to registration page if user is already logged in.
#[get("/auth/registration")]
pub fn register_user(_user: AuthUser) -> Redirect {
    Redirect::to("/")
}

#[derive(Serialize)]
struct RegistrationContext {
    min_password_length: usize,
    flashes: Vec<crate::flash::FlashMessage>,
}

/// Show the registration page (with flash messages) if not already logged in.
#[get("/auth/registration", rank = 2)]
pub fn registration_page(flash: Option<FlashMessage>) -> Template {
    let flash_messages = if let Some(f) = flash {
        vec![crate::flash::FlashMessage::from(f)]
    } else {
        Vec::new()
    };
    let context = RegistrationContext {
        min_password_length: data::MIN_PASSWORD_LENGTH,
        flashes: flash_messages,
    };
    Template::render("registration", &context)
}

/// Registration handler
#[post("/auth/registration", data = "<registration>")]
pub async fn registration(
    cookies: &CookieJar<'_>,
    registration: Form<Registration>,
    database: Database,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let registration_clone = registration.clone();
    let registration_result = database
        .run(move |db| {
            data::validate_registration(
                db,
                &registration_clone.email,
                &registration_clone.username,
                &registration_clone.password,
                &registration_clone.password_confirmation,
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
                        &registration.password,
                        &registration.email,
                    )
                })
                .await;
            cookies.add_private(Cookie::new(USER_COOKIE_ID, new_user.id.to_string()));
            Ok(Flash::success(
                Redirect::to("/"),
                "Your account was successfully created",
            ))
        }
        Err(error) => {
            let msg = error.to_string();
            log::error!("Was not able to register user: {}", msg);
            Err(Flash::error(Redirect::to(uri!(registration_page)), msg))
        }
    }
}

#[derive(FromForm)]
pub struct PasswordChange {
    current: String,
    new1: String,
    new2: String,
}

/// Change user password.
#[get("/auth/password/change")]
pub fn password_change_form(user: AuthUser, flash: Option<FlashMessage>) -> Template {
    let context = UserContext::with_flash(user.into_inner(), flash);
    Template::render("password-change", &context)
}

/// Change user password.
#[get("/auth/password/change", rank = 2)]
pub fn password_change_form_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

#[post("/auth/password/change", data = "<password_change>")]
pub async fn password_change(
    user: AuthUser,
    password_change: Form<PasswordChange>,
    database: Database,
) -> Flash<Redirect> {
    let user = Arc::new(user.into_inner());

    macro_rules! fail {
        ($msg:expr) => {{
            log::error!("Could not change password for user {}: {}", user.id, $msg);
            return Flash::error(Redirect::to(uri!(password_change_form)), $msg);
        }};
    }

    // Compare new passwords
    if password_change.new1 != password_change.new2 {
        fail!("Passwords don't match");
    }

    // Ensure minimum length of new password
    if password_change.new1.len() < data::MIN_PASSWORD_LENGTH {
        fail!("Password must have at least 8 characters");
    }

    // Unwrap password fields
    let PasswordChange {
        current: pw_current,
        new1: pw_new,
        ..
    } = password_change.into_inner();

    // Verify current password
    let user_clone = user.clone();
    match database
        .run(move |db| data::validate_login(db, &user_clone.username, &pw_current))
        .await
    {
        Some(u) if u.id == user.id => { /* Valid password */ }
        Some(_) => fail!("Invalid user"),
        None => fail!("Invalid current password"),
    }

    // Update password
    database
        .run(move |db| data::update_password(db, &user, &pw_new))
        .await;
    Flash::success(Redirect::to(uri!(crate::profile::view)), "Password changed")
}

/// Return the auth routes.
pub fn get_routes() -> Vec<Route> {
    routes![
        login,
        login_user,
        login_page,
        registration,
        register_user,
        registration_page,
        logout,
        password_change_form,
        password_change_form_nologin,
        password_change,
    ]
}

/// A context that just contains the user.
#[derive(Serialize)]
pub struct UserContext {
    pub user: User,
    pub flashes: Vec<crate::flash::FlashMessage>,
}

impl UserContext {
    pub fn new(user: User) -> Self {
        Self {
            user,
            flashes: vec![],
        }
    }

    pub fn with_flash(user: User, flash: Option<FlashMessage>) -> Self {
        Self {
            user,
            flashes: if let Some(f) = flash {
                vec![f.into()]
            } else {
                vec![]
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use rocket::{
        self,
        http::{ContentType, Status},
        local::blocking::Client,
        routes,
    };

    use crate::{
        templates,
        test_utils::{make_test_config, DbTestContext},
        Config,
    };

    use super::*;

    /// Create a new test client with cookie tracking.
    fn make_client() -> Client {
        let mut test_routes = get_routes();
        test_routes.extend_from_slice(&routes![crate::profile::view]);
        let app = rocket::custom(make_test_config())
            .attach(data::Database::fairing())
            .attach(templates::fairing(&Config::default()))
            .mount("/", test_routes);
        Client::tracked(app).expect("valid rocket instance")
    }

    #[derive(Debug)]
    struct ChangeResult {
        redirect_url: String,
        body: String,
    }

    /// Request a password change, return the redirect URL and the body of the redirect target.
    fn password_change_request<'a>(
        client: &'a mut Client,
        ctx: &DbTestContext,
        current: &str,
        new1: &str,
        new2: &str,
    ) -> ChangeResult {
        let resp1 = client
            .post("/auth/password/change")
            .header(ContentType::Form)
            .body(&format!("current={}&new1={}&new2={}", current, new1, new2))
            .private_cookie(ctx.auth_cookie_user1())
            .dispatch();
        assert_eq!(resp1.status(), Status::SeeOther);
        let redirect_url = resp1
            .headers()
            .get_one("location")
            .expect("location header")
            .to_string();
        let resp2 = client
            .get(redirect_url.clone())
            .private_cookie(ctx.auth_cookie_user1())
            .dispatch();
        assert_eq!(resp2.status(), Status::Ok);
        ChangeResult {
            redirect_url,
            body: resp2.into_string().expect("body"),
        }
    }

    #[test]
    fn password_change_error() {
        let ctx = DbTestContext::new();
        let mut client = make_client();

        macro_rules! password_valid {
            ($pw:expr) => {
                data::validate_login(&*ctx.force_get_conn(), &ctx.testuser1.user.username, $pw).is_some()
            };
        }

        // Validate old password
        assert!(
            password_valid!(&ctx.testuser1.password),
            "Test user password does not work"
        );

        // New password mismatch
        let res = password_change_request(&mut client, &ctx, "a", "b", "c");
        assert_eq!(res.redirect_url, "/auth/password/change");
        assert!(
            res.body.contains("Error: Passwords don&#x27;t match"),
            "Error message not found"
        );

        // Password too short
        let res = password_change_request(&mut client, &ctx, &ctx.testuser1.password, "abc", "abc");
        assert_eq!(res.redirect_url, "/auth/password/change");
        assert!(
            res.body
                .contains("Error: Password must have at least 8 characters"),
            "Error message not found"
        );

        // Old password is wrong
        let res = password_change_request(&mut client, &ctx, "iawieoioijf", "abcdefgh", "abcdefgh");
        assert_eq!(res.redirect_url, "/auth/password/change");
        assert!(
            res.body.contains("Error: Invalid current password"),
            "Error message not found"
        );

        // Ensure password wasn't changed
        assert!(
            password_valid!(&ctx.testuser1.password),
            "Test user password was changed"
        );

        // Successful password change
        let res = password_change_request(&mut client, &ctx, &ctx.testuser1.password, "abcdefgh", "abcdefgh");
        assert_eq!(res.redirect_url, "/profile");
        println!("{}", res.body);
        assert!(res.body.contains("Password changed"), "Success message not found");

        // Ensure password was changed
        assert!(
            !password_valid!(&ctx.testuser1.password),
            "Test user password still works"
        );
        assert!(password_valid!("abcdefgh"), "New password doesn't work");
    }
}
