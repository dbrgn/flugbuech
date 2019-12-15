//! Glider views.

use rocket::request::{Form, FromForm};
use rocket::response::Redirect;
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
    glider: Glider,
}

#[derive(Serialize)]
struct GlidersContext {
    user: User,
    gliders: Vec<Glider>,
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
    let context = auth::UserContext::new(user.into_inner());
    Template::render("glider", &context)
}

#[get("/gliders/add", rank = 2)]
pub(crate) fn add_form_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

#[post("/gliders/add", data = "<data>")]
pub(crate) fn add(user: auth::AuthUser, db: data::Database, data: Form<GliderForm>) -> Redirect {
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
    // TODO: Error handling
    data::create_glider(&db, glider);

    // Redirect to glider list
    Redirect::to(uri!(list))
}

#[cfg(test)]
mod tests {
    use rocket::{self, routes};
    use rocket::http::{ContentType, Status};
    use rocket::local::Client;

    use crate::test_utils::{DbTestContext, make_test_config};

    use super::*;

    /// Create a new test client. Cookie tracking is disabled.
    fn make_client() -> Client {
        let app = rocket::custom(make_test_config())
            .attach(data::Database::fairing())
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
            ($body:expr) => {
                client
                    .post("/gliders/add")
                    .header(ContentType::Form)
                    .body($body)
                    .private_cookie(ctx.make_auth_cookie())
                    .dispatch()
            }
        }

        // Add glider
        let resp = add_glider!("manufacturer=Advance&model=Epsilon%208");
        assert_eq!(resp.status(), Status::SeeOther);

        // Verify database
        let g = data::get_gliders_for_user(&*ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].manufacturer, "Advance");
        assert_eq!(g[0].model, "Epsilon 8");
    }
}
