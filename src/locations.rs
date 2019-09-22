//! Location views.

use diesel_geography::types::GeogPoint;
use rocket::http::Status;
use rocket::request::{Form, FromForm};
use rocket::response::Redirect;
use rocket::{get, post, uri};
use rocket_contrib::templates::Template;
use serde::Serialize;

use crate::models::{Location, NewLocation, User};
use crate::{auth, data};

// Forms

#[derive(FromForm, Debug)]
pub(crate) struct LocationForm {
    name: String,
    country: String,
    elevation: i32,
    lat: Option<f64>,
    lon: Option<f64>,
}

// Contexts

#[derive(Serialize)]
struct LocationContext {
    user: User,
    location: Location,
}

#[derive(Serialize)]
struct LocationsContext {
    user: User,
    locations: Vec<Location>,
}

// Views

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

#[get("/locations/<id>/edit")]
pub(crate) fn edit_form(user: auth::AuthUser, db: data::Database, id: i32) -> Result<Template, Status> {
    let user = user.into_inner();

    // Get location
    let location = match data::get_location_with_id(&db, id) {
        Some(location) => location,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if location.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Render template
    let context = LocationContext { user, location };
    Ok(Template::render("location", &context))
}

#[post("/locations/<id>/edit", data = "<data>")]
pub(crate) fn edit(
    user: auth::AuthUser,
    db: data::Database,
    id: i32,
    data: Form<LocationForm>,
) -> Result<Redirect, Status> {
    let user = user.into_inner();

    // Get location
    let mut location = match data::get_location_with_id(&db, id) {
        Some(location) => location,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if location.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Update model
    let LocationForm { name, country, elevation, lat, lon } = data.into_inner();
    location.name = name;
    location.country = country;
    location.elevation = elevation;
    if let (Some(lat), Some(lon)) = (lat, lon) {
        location.geog = Some(GeogPoint { x: lon, y: lat, srid: None });
    } else {
        location.geog = None;
    }

    // Update database
    // TODO: Error handling
    data::update_location(&db, &location);

    // Render template
    Ok(Redirect::to(uri!(list)))
}
