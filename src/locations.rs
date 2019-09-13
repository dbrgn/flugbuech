//! Location views.

use rocket::request::{Form, FromForm};
use rocket::response::Redirect;
use rocket::{get, post, uri};
use rocket_contrib::templates::Template;
use serde::Serialize;

use crate::models::{Location, NewLocation, User};
use crate::{auth, data};

#[derive(FromForm, Debug)]
pub(crate) struct LocationForm {
    name: String,
    country: String,
    elevation: i32,
}

#[derive(Serialize)]
struct LocationsContext {
    user: User,
    locations: Vec<Location>,
}

#[get("/locations")]
pub(crate) fn list(db: data::Database, user: auth::AuthUser) -> Template {
    let user = user.into_inner();

    // Get all locations
    let locations = data::get_locations_for_user(&db, &user);

    // Render template
    let context = LocationsContext { user, locations };
    Template::render("locations", &context)
}

#[get("/locations", rank = 2)]
pub(crate) fn list_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

#[get("/locations/add")]
pub(crate) fn add_form(user: auth::AuthUser) -> Template {
    let user = user.into_inner();

    // Render template
    let context = auth::UserContext::new(user);
    Template::render("location", &context)
}

#[get("/locations/add", rank = 2)]
pub(crate) fn add_form_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

#[post("/locations/add", data = "<data>")]
pub(crate) fn add(user: auth::AuthUser, db: data::Database, data: Form<LocationForm>) -> Redirect {
    let user = user.into_inner();

    // Create model
    let location = NewLocation {
        name: data.name.clone(),
        country: data.country.clone(),
        elevation: data.elevation,
        user_id: user.id,
    };

    // Create database entry
    // TODO: Error handling
    data::create_location(&db, location);

    // Redirect to location list
    Redirect::to(uri!(list))
}
