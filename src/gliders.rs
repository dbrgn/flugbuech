//! Glider views.

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use rocket::http::Status;
use rocket::request::{Form, FromForm, Request};
use rocket::response::{self, Redirect};
use rocket::{get, post, uri};
use rocket_contrib::templates::Template;
use serde::Serialize;

use crate::models::{Glider, NewGlider, User};
use crate::{auth, data};

// Forms

#[derive(FromForm, Debug)]
pub(crate) struct GliderForm {
    manufacturer: String,
    model: String,
}

// Contexts

#[derive(Serialize)]
struct GliderContext {
    user: User,
    glider: Option<Glider>,
    error_msg: Option<String>,
}

#[derive(Serialize)]
struct GlidersContext {
    user: User,
    gliders: Vec<Glider>,
}

// Renderers

fn render_form(user: User, glider: Option<Glider>, error_msg: Option<String>) -> Template {
    let context = GliderContext {
        user,
        glider,
        error_msg,
    };
    Template::render("glider", &context)
}

// Views

#[get("/gliders")]
pub(crate) fn list(db: data::Database, user: auth::AuthUser) -> Template {
    let user = user.into_inner();

    // Get all gliders
    let gliders = data::get_gliders_for_user(&db, &user);

    // Render template
    let context = GlidersContext { user, gliders };
    Template::render("gliders", &context)
}

#[get("/gliders", rank = 2)]
pub(crate) fn list_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

#[get("/gliders/add")]
pub(crate) fn add_form(user: auth::AuthUser) -> Template {
    render_form(user.into_inner(), None, None)
}

#[get("/gliders/add", rank = 2)]
pub(crate) fn add_form_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

/// The result of a form validation: Either a successful redirect, or a
/// template response with an error status.
///
/// TODO: Generalize this and move it into a helper module.
pub enum ValidationResult {
    Success(Redirect),
    Invalid(Template, Status),
}

impl<'r> response::Responder<'r> for ValidationResult {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        match self {
            ValidationResult::Success(redirect) => redirect.respond_to(req),
            ValidationResult::Invalid(template, status) => template.respond_to(req).map(|mut response| {
                response.set_status(status);
                response
            }),
        }
    }
}

#[post("/gliders/add", data = "<data>")]
pub(crate) fn add(user: auth::AuthUser, db: data::Database, data: Form<GliderForm>) -> ValidationResult {
    let user = user.into_inner();

    // Destructure data
    let GliderForm { manufacturer, model } = data.into_inner();

    // Create model
    let glider = NewGlider {
        user_id: user.id,
        manufacturer,
        model,
    };

    // Create database entry
    match data::create_glider(&db, glider) {
        Ok(_) => {
            // Glider created, redirect to glider list
            ValidationResult::Success(Redirect::to(uri!(list)))
        },
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _info)) => {
            ValidationResult::Invalid(
                render_form(
                    user,
                    None,
                    Some("You can't add the same glider twice.".to_string()),
                ),
                Status::Conflict,
            )
        },
        Err(other) => ValidationResult::Invalid(
            render_form(user, None, Some(format!("Unknown error: {}", other))),
            Status::InternalServerError,
        ),
    }
}

#[get("/gliders/<id>/edit")]
pub(crate) fn edit_form(user: auth::AuthUser, _db: data::Database, id: i32) -> &'static str {
    let _user = user.into_inner();
    let _id = id;

    "Not yet implemented, will be added soon! :)"
}

#[cfg(test)]
mod tests {
    use rocket::http::ContentType;
    use rocket::local::Client;
    use rocket::{self, routes};

    use crate::templates;
    use crate::test_utils::{make_test_config, DbTestContext};

    use super::*;

    /// Create a new test client. Cookie tracking is disabled.
    fn make_client() -> Client {
        let app = rocket::custom(make_test_config())
            .attach(data::Database::fairing())
            .attach(templates::fairing())
            .mount("/", routes![add]);
        Client::untracked(app).expect("valid rocket instance")
    }

    #[test]
    fn add_glider() {
        let ctx = DbTestContext::new();
        let client = make_client();

        // No gliders
        let g = data::get_gliders_for_user(&*ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(g.len(), 0);

        macro_rules! add_glider {
            ($body:expr, $cookie:expr) => {
                client
                    .post("/gliders/add")
                    .header(ContentType::Form)
                    .body($body)
                    .private_cookie($cookie)
                    .dispatch()
            };
        }

        // Add glider
        let resp = add_glider!("manufacturer=Advance&model=Epsilon%208", ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::SeeOther);

        // Verify database
        let g = data::get_gliders_for_user(&*ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].manufacturer, "Advance");
        assert_eq!(g[0].model, "Epsilon 8");

        // Cannot add a glider twice
        let resp = add_glider!("manufacturer=Advance&model=Epsilon%208", ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::Conflict);
        assert_eq!(
            data::get_gliders_for_user(&*ctx.force_get_conn(), &ctx.testuser1.user).len(),
            1,
        );

        // ...but another user can!
        assert_eq!(
            data::get_gliders_for_user(&*ctx.force_get_conn(), &ctx.testuser2.user).len(),
            0,
        );
        let resp = add_glider!("manufacturer=Advance&model=Epsilon%208", ctx.auth_cookie_user2());
        assert_eq!(resp.status(), Status::SeeOther);
        assert_eq!(
            data::get_gliders_for_user(&*ctx.force_get_conn(), &ctx.testuser2.user).len(),
            1,
        );
    }
}
