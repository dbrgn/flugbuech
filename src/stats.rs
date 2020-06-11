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
    flight_time_per_year: Vec<(u16, u64)>, // (year, seconds)
    flight_time_total: u64,
}

// Views

#[get("/stats")]
pub(crate) fn stats(db: data::Database, user: auth::AuthUser) -> Template {
    let user = user.into_inner();

    // Get all locations
    let launch_locations = data::get_locations_with_stats_for_user(&db, &user, LocationOrderBy::Launches, 10);
    let landing_locations =
        data::get_locations_with_stats_for_user(&db, &user, LocationOrderBy::Landings, 10);

    // Get hours per year
    let flight_time_per_year = data::get_flight_time_per_year_for_user(&db, &user);
    let flight_time_total = flight_time_per_year.iter().map(|(_, seconds)| seconds).sum();

    // Render template
    let context = StatsContext {
        user,
        launch_locations,
        landing_locations,
        flight_time_per_year,
        flight_time_total,
    };
    Template::render("stats", &context)
}

#[get("/stats", rank = 2)]
pub(crate) fn stats_nologin() -> Redirect {
    Redirect::to("/auth/login")
}
