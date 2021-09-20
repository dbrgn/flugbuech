//! Location views.

use diesel_geography::types::GeogPoint;
use rocket::{
    form::{Form, FromForm},
    get,
    http::Status,
    post,
    request::FlashMessage,
    response::{Flash, Redirect},
    uri,
};
use rocket_dyn_templates::Template;
use serde::Serialize;

use crate::{
    auth, data,
    flash::flashes_from_flash_opt,
    models::{LocationWithCount, NewLocation, User},
};

// Forms

#[derive(FromForm, Debug)]
pub struct LocationForm {
    name: String,
    country: String,
    elevation: i32,
    lat: Option<f64>,
    lng: Option<f64>,
}

// Contexts

#[derive(Serialize)]
struct LocationContext {
    user: User,
    location: LocationWithCount,
}

#[derive(Serialize)]
struct LocationsContext {
    user: User,
    locations: Vec<LocationWithCount>,
    flashes: Vec<crate::flash::FlashMessage>,
}

// Views

#[get("/locations/<id>")]
pub async fn view(user: auth::AuthUser, database: data::Database, id: i32) -> Result<Template, Status> {
    let user = user.into_inner();

    // Get data
    let location = match database
        .run(move |db| data::get_location_with_flight_count_by_id(db, id))
        .await
    {
        Some(location) => location,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if location.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Render template
    let context = LocationContext { user, location };
    Ok(Template::render("location", context))
}

#[get("/locations")]
pub async fn list(
    database: data::Database,
    user: auth::AuthUser,
    flash: Option<FlashMessage<'_>>,
) -> Template {
    let user = user.into_inner();

    // Get all locations
    let locations = database
        .run({
            let user = user.clone();
            move |db| data::get_all_locations_with_stats_for_user(db, &user)
        })
        .await;

    // Render template
    let context = LocationsContext {
        user,
        locations,
        flashes: flashes_from_flash_opt(flash),
    };
    Template::render("locations", &context)
}

#[get("/locations", rank = 2)]
pub fn list_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

#[get("/locations/add")]
pub fn add_form(user: auth::AuthUser) -> Template {
    let context = auth::UserContext::new(user.into_inner());
    Template::render("location_edit", &context)
}

#[get("/locations/add", rank = 2)]
pub fn add_form_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

#[post("/locations/add", data = "<data>")]
pub async fn add(user: auth::AuthUser, database: data::Database, data: Form<LocationForm>) -> Redirect {
    log::debug!("locations::add");
    let user = user.into_inner();

    let LocationForm {
        name,
        country,
        elevation,
        lat,
        lng,
    } = data.into_inner();

    // Create model
    let location = NewLocation {
        name,
        country,
        elevation,
        user_id: user.id,
        geog: if let (Some(lat), Some(lng)) = (lat, lng) {
            Some(GeogPoint {
                x: lng,
                y: lat,
                srid: None,
            })
        } else {
            None
        },
    };

    // Create database entry
    // TODO: Error handling
    database.run(move |db| data::create_location(db, location)).await;
    log::info!("Created location for user {}", user.id);

    // Redirect to location list
    Redirect::to(uri!(list))
}

#[get("/locations/<id>/edit")]
pub async fn edit_form(user: auth::AuthUser, database: data::Database, id: i32) -> Result<Template, Status> {
    let user = user.into_inner();

    // Get location
    let location = match database
        .run(move |db| data::get_location_with_flight_count_by_id(db, id))
        .await
    {
        Some(location) => location,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if location.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Render template
    let context = LocationContext { user, location };
    Ok(Template::render("location_edit", &context))
}

#[post("/locations/<id>/edit", data = "<data>")]
pub async fn edit(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
    data: Form<LocationForm>,
) -> Result<Redirect, Status> {
    let user = user.into_inner();

    // Get location
    let mut location = match database.run(move |db| data::get_location_by_id(db, id)).await {
        Some(location) => location,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if location.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Update model
    let LocationForm {
        name,
        country,
        elevation,
        lat,
        lng,
    } = data.into_inner();
    location.name = name;
    location.country = country;
    location.elevation = elevation;
    if let (Some(lat), Some(lng)) = (lat, lng) {
        location.geog = Some(GeogPoint {
            x: lng,
            y: lat,
            srid: None,
        });
    } else {
        location.geog = None;
    }

    // Update database
    // TODO: Error handling
    database.run(move |db| data::update_location(db, &location)).await;

    // Render template
    Ok(Redirect::to(uri!(list)))
}

#[get("/locations/<id>/delete")]
pub async fn delete_form(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
) -> Result<Template, Status> {
    let user = user.into_inner();

    // Get data
    let location = match database
        .run(move |db| data::get_location_with_flight_count_by_id(db, id))
        .await
    {
        Some(location) => location,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if location.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Ensure that no related flights exist (otherwise a foreign key constraint
    // would be returned by the database when attempting to delete)
    if location.count > 0 {
        return Err(Status::Conflict);
    }

    // Render template
    let context = LocationContext { user, location };
    Ok(Template::render("location_delete", context))
}

#[post("/locations/<id>/delete")]
pub async fn delete(
    user: auth::AuthUser,
    database: data::Database,
    id: i32,
) -> Result<Flash<Redirect>, Status> {
    let user = user.into_inner();

    // Get data
    let location = match database
        .run(move |db| data::get_location_with_flight_count_by_id(db, id))
        .await
    {
        Some(location) => location,
        None => return Err(Status::NotFound),
    };

    // Ownership check
    if location.user_id != user.id {
        return Err(Status::Forbidden);
    }

    // Ensure that no related flights exist (otherwise a foreign key constraint
    // would be returned by the database when attempting to delete)
    if location.count > 0 {
        return Err(Status::Conflict);
    }

    // Delete database entry
    let location_id = location.id;
    database
        .run(move |db| data::delete_location_by_id(db, location_id))
        .await
        .map(|()| {
            log::info!("Deleted location with ID {}", location.id);
            Flash::success(
                Redirect::to(uri!(list)),
                format!("Location \"{}\" deleted", location.name),
            )
        })
        .or_else(|e| {
            log::error!("Could not delete location with ID {}: {}", location.id, e);
            Ok(Flash::error(
                Redirect::to(uri!(list)),
                format!("Could not delete location \"{}\"", location.name),
            ))
        })
}
