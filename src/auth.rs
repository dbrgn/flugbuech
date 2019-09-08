//! Authentication and authorization related functionality.

use std::collections::HashMap;

use log::error;
use rocket::http::{Cookie, Cookies};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FlashMessage, Form, FromRequest, Request};
use rocket::response::{Flash, Redirect};
use rocket::{get, post, routes, uri, FromForm, Route};
use rocket_contrib::templates::Template;

use crate::data::{self, Database};
use crate::models::User;

const USER_COOKIE_ID: &str = "user_id";

#[derive(FromForm)]
pub struct Login {
    username: String,
    password: String,
}

/// User newtype, wraps the user model, provides guard transparency.
#[derive(Debug)]
pub struct AuthUser(User);

/// Get the user model from a request cookie.
impl<'a, 'r> FromRequest<'a, 'r> for AuthUser {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<AuthUser, Self::Error> {
        Database::from_request(request).and_then(|db| {
            let mut cookies = request.cookies();
            cookies
                .get_private(USER_COOKIE_ID)
                .and_then(|cookie| {
                    // A login cookie was found. Look up the corresponding database user.
                    cookie.value().parse().ok().and_then(|id| {
                        let user = data::get_user(&db, id);
                        if user.is_none() {
                            error!("Login cookie with invalid user id found. Removing cookie.");
                            cookies.remove_private(cookie);
                        }
                        user
                    })
                })
                .map(AuthUser)
                .or_forward(())
        })
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
pub fn login(mut cookies: Cookies, login: Form<Login>, db: Database) -> Result<Redirect, Flash<Redirect>> {
    match data::validate_login(&db, &login.username, &login.password) {
        Some(user) => {
            cookies.add_private(Cookie::new(USER_COOKIE_ID, user.id.to_string()));
            Ok(Redirect::to("/"))
        },
        None => Err(Flash::error(
            Redirect::to(uri!(login_page)),
            "Invalid username/password.",
        )),
    }
}

/// Logout handler.
#[get("/auth/logout")]
pub fn logout(mut cookies: Cookies) -> Redirect {
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
    let mut context = HashMap::new();
    if let Some(ref msg) = flash {
        context.insert("flash", msg.msg());
    }
    Template::render("login", &context)
}

/// Return the auth routes.
pub fn get_routes() -> Vec<Route> {
    routes![login, login_user, login_page, logout]
}
