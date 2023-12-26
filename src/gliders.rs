//! Glider views.

use chrono::NaiveDate;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use rocket::{
    form::{Form, FromForm},
    get,
    http::Status,
    post,
    request::Request,
    response::{self, Redirect, Responder},
    uri,
};
use rocket_dyn_templates::Template;
use serde::Serialize;

use crate::models::{Glider, GliderWithStats, NewGlider, User};
use crate::{auth, data};

// Forms

#[derive(FromForm, Debug)]
pub struct GliderForm {
    manufacturer: String,
    model: String,
    since: Option<String>, // ISO date (e.g. 2010-11-30) // TODO: Use OptionResult<NaiveDate>
    until: Option<String>, // ISO date (e.g. 2010-11-30) // TODO: Use OptionResult<NaiveDate>
    source: Option<String>,
    cost: Option<i32>,
    comment: Option<String>,
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
    gliders: Vec<GliderWithStats>,
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
pub async fn list(database: data::Database, user: auth::AuthUser) -> Template {
    let user = user.into_inner();

    // Get all gliders
    let gliders = database
        .run({
            let user = user.clone();
            move |db| data::get_gliders_with_stats_for_user(db, &user)
        })
        .await;

    // Render template
    let context = GlidersContext { user, gliders };
    Template::render("gliders", &context)
}

#[get("/gliders", rank = 2)]
pub fn list_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

#[get("/gliders/add")]
pub fn add_form(user: auth::AuthUser) -> Template {
    render_form(user.into_inner(), None, None)
}

#[get("/gliders/add", rank = 2)]
pub fn add_form_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

/// The result of a form validation: Either a successful redirect, or a
/// template response with an error status.
///
/// TODO: Generalize this and move it into a helper module.
pub enum ValidationResult {
    Success(Box<Redirect>),
    Invalid(Template, Status),
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for ValidationResult {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
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
pub async fn add(user: auth::AuthUser, database: data::Database, data: Form<GliderForm>) -> ValidationResult {
    log::debug!("gliders::add");
    let user = user.into_inner();

    // Destructure data
    let GliderForm {
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
            // Glider created, redirect to glider list
            log::info!("Created glider for user {}", user.id);
            ValidationResult::Success(Box::new(Redirect::to(uri!(list))))
        }
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _info)) => {
            ValidationResult::Invalid(
                render_form(
                    user,
                    None,
                    Some("You can't add the same glider twice.".to_string()),
                ),
                Status::Conflict,
            )
        }
        Err(other) => ValidationResult::Invalid(
            render_form(user, None, Some(format!("Unknown error: {}", other))),
            Status::InternalServerError,
        ),
    }
}

#[get("/gliders/<id>/edit")]
pub async fn edit_form(user: auth::AuthUser, database: data::Database, id: i32) -> Result<Template, Status> {
    let user = user.into_inner();

    // Get glider
    let glider = database
        .run(move |db| data::get_glider_with_id(db, id))
        .await
        .ok_or(Status::NotFound)?;

    // Ownership check
    if glider.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Render template
    let context = GliderContext {
        user,
        glider: Some(glider),
        error_msg: None,
    };
    Ok(Template::render("glider", &context))
}

#[post("/gliders/<id>/edit", data = "<data>")]
pub async fn edit(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
    data: Form<GliderForm>,
) -> Result<Redirect, Status> {
    let user = user.into_inner();

    // Get glider
    let mut glider = database
        .run(move |db| data::get_glider_with_id(db, id))
        .await
        .ok_or(Status::NotFound)?;

    // Ownership check
    if glider.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Update model
    let GliderForm {
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
    // TODO: Error handling
    database.run(move |db| data::update_glider(db, &glider)).await;

    // Render template
    Ok(Redirect::to(uri!(list)))
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rocket::{self, http::ContentType, local::blocking::Client, routes};

    use crate::{
        flights, templates,
        test_utils::{make_test_config, DbTestContext},
        Config,
    };

    use super::*;

    /// Create a new test client. Cookie tracking is disabled.
    fn make_client() -> Client {
        let app = rocket::custom(make_test_config())
            .attach(data::Database::fairing())
            .attach(templates::fairing(&Config::default()))
            .mount("/", routes![add, flights::submit]);
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
                    .post("/gliders/add")
                    .header(ContentType::Form)
                    .body($body)
                    .private_cookie($cookie)
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }

        // Add glider
        let resp = add_glider!("manufacturer=Advance&model=Epsilon%208", ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::SeeOther);

        // Verify database
        let g = data::get_gliders_for_user(&mut *ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].manufacturer, "Advance");
        assert_eq!(g[0].model, "Epsilon 8");

        // Cannot add a glider twice
        let resp = add_glider!("manufacturer=Advance&model=Epsilon%208", ctx.auth_cookie_user1());
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
        let resp = add_glider!("manufacturer=Advance&model=Epsilon%208", ctx.auth_cookie_user2());
        assert_eq!(resp.status(), Status::SeeOther);
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
                    .post("/gliders/add")
                    .header(ContentType::Form)
                    .body($body)
                    .private_cookie($cookie)
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }

        // Add glider
        let resp = add_glider!(
            "manufacturer=Ozone&model=Enzo%202&since=2019-02-03&until=2019-11-20&source=Flycenter&cost=3344&comment=Sold%20it%20to%20Joe.",
            ctx.auth_cookie_user1()
        );
        assert_eq!(resp.status(), Status::SeeOther);

        // Verify database
        let g = data::get_gliders_for_user(&mut *ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].manufacturer, "Ozone");
        assert_eq!(g[0].model, "Enzo 2");
        assert_eq!(g[0].since, Some(NaiveDate::from_ymd_opt(2019, 2, 3).unwrap()));
        assert_eq!(g[0].until, Some(NaiveDate::from_ymd_opt(2019, 11, 20).unwrap()));
        assert_eq!(g[0].source, Some("Flycenter".into()));
        assert_eq!(g[0].cost, Some(3344));
        assert_eq!(g[0].comment, Some("Sold it to Joe.".into()));
    }

    #[test]
    fn get_gliders_with_stats() {
        let ctx = DbTestContext::new();
        let client = make_client();

        macro_rules! add_glider {
            ($body:expr, $cookie:expr) => {
                client
                    .post("/gliders/add")
                    .header(ContentType::Form)
                    .body($body)
                    .private_cookie($cookie)
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }
        macro_rules! add_flight {
            ($glider:expr, $launch_date:expr, $launch_time:expr, $landing_time: expr, $cookie:expr) => {
                client
                    .post("/flights/add")
                    .header(ContentType::Form)
                    .body(&format!(
                        "igc_data=&number=&glider={}&launch_site=&landing_site=&launch_date={}&launch_time={}&landing_time={}&track_distance=&xcontest_tracktype=&xcontest_distance=&xcontest_url=&comment=&video_url=",
                        $glider, $launch_date, $launch_time, $landing_time,
                    ))
                    .private_cookie($cookie)
                    .cookie(ctx.username_cookie())
                    .dispatch()
            };
        }

        // No gliders
        let g = data::get_gliders_with_stats_for_user(&mut *ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(g.len(), 0);

        // Add gliders
        let resp = add_glider!("manufacturer=A&model=1", ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::SeeOther);

        // One glider, no flights
        let g = data::get_gliders_with_stats_for_user(&mut *ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(1, g.len(), "No gliders found");
        assert_eq!(g[0].manufacturer, "A");
        assert_eq!(g[0].model, "1");
        assert_eq!(g[0].flights, 0);
        assert_eq!(g[0].seconds, 0);
        assert_eq!(g[0].seconds_complete, true);

        // Add a flight, without launch/landing time
        let resp = add_flight!(g[0].id, "", "", "", ctx.auth_cookie_user1());
        assert_eq!(resp.status(), Status::SeeOther);
        let g = data::get_gliders_with_stats_for_user(&mut *ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(1, g.len(), "No gliders found");
        assert_eq!(g[0].flights, 1);
        assert_eq!(g[0].seconds, 0);
        assert_eq!(g[0].seconds_complete, false);

        // Add two flights with launch/landing time
        let resp = add_flight!(
            g[0].id,
            "2020-09-25",
            "12:00:00",
            "13:00:00",
            ctx.auth_cookie_user1()
        );
        assert_eq!(resp.status(), Status::SeeOther);
        let resp = add_flight!(
            g[0].id,
            "2020-09-25",
            "12:30:00",
            "12:35:00",
            ctx.auth_cookie_user1()
        );
        assert_eq!(resp.status(), Status::SeeOther);
        let g = data::get_gliders_with_stats_for_user(&mut *ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(1, g.len(), "No gliders found");
        assert_eq!(g[0].flights, 3);
        assert_eq!(g[0].seconds, 3600 + 300);
        assert_eq!(g[0].seconds_complete, false);
    }
}
