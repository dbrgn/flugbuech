//! Stats views.

use rocket::{get, response::Redirect};
use rocket_contrib::templates::Template;
use serde::Serialize;

use crate::{
    auth,
    data::{self, LocationOrderBy},
    models::{LocationWithCount, User},
};

// Contexts

#[derive(Serialize)]
struct StatsContext {
    user: User,
    launch_locations: Vec<LocationWithCount>,
    landing_locations: Vec<LocationWithCount>,
}

// Views

#[get("/stats")]
pub(crate) fn stats(db: data::Database, user: auth::AuthUser) -> Template {
    let user = user.into_inner();

    // Get all locations
    let launch_locations = data::get_locations_with_stats_for_user(&db, &user, LocationOrderBy::Launches, 10);
    let landing_locations =
        data::get_locations_with_stats_for_user(&db, &user, LocationOrderBy::Landings, 10);

    // Render template
    let context = StatsContext {
        user,
        launch_locations,
        landing_locations,
    };
    Template::render("stats", &context)
}

#[get("/stats", rank = 2)]
pub(crate) fn stats_nologin() -> Redirect {
    Redirect::to("/auth/login")
}
