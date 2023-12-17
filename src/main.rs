#[macro_use]
extern crate diesel;
extern crate diesel_migrations;

mod auth;
mod base64;
mod data;
mod filters;
mod flash;
mod flights;
mod gliders;
mod locations;
mod models;
mod optionresult;
mod process_igc;
mod profile;
mod schema;
mod stats;
mod templates;
#[cfg(test)]
mod test_utils;

use anyhow::{Context, Result};
use clap::{App, Arg};
use rocket::{
    catch, catchers,
    fs::FileServer,
    get,
    request::{FlashMessage, Request},
    routes,
};
use rocket_dyn_templates::Template;
use serde::{Deserialize, Serialize};

pub const MAX_UPLOAD_BYTES: u64 = 50 * 1024 * 1024;
pub const NAME: &str = "flugbuech";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = "Paragliding flight book.";

// Config

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    pub static_files_dir: Option<String>,
    pub plausible_domain: Option<String>,
    pub plausible_url: Option<String>,
}

// Index

#[derive(Serialize)]
struct IndexContext {
    user: Option<models::User>,
    user_count: i64,
    glider_count: i64,
    flight_count: i64,
    flashes: Vec<crate::flash::FlashMessage>,
}

#[get("/")]
async fn index(
    database: data::Database,
    user: Option<auth::AuthUser>,
    flash: Option<FlashMessage<'_>>,
) -> Template {
    let flash_messages = if let Some(f) = flash {
        vec![crate::flash::FlashMessage::from(f)]
    } else {
        Vec::new()
    };
    let (user_count, glider_count, flight_count) = database
        .run(|db| {
            (
                data::get_user_count(db),
                data::get_glider_count(db),
                data::get_flight_count(db),
            )
        })
        .await;
    let context = IndexContext {
        user: user.map(|u| u.into_inner()),
        user_count,
        glider_count,
        flight_count,
        flashes: flash_messages,
    };
    Template::render("index", &context)
}

// Handle missing DB

#[catch(503)]
fn service_not_available(_req: &Request) -> &'static str {
    "Service is not available. (Is the database up?)"
}

// Main

#[rocket::main]
async fn main() -> Result<()> {
    // Load env
    let _ = dotenv::dotenv();

    // Parse args
    let args = App::new(NAME)
        .about(DESCRIPTION)
        .version(VERSION)
        .arg(
            Arg::with_name("migrate")
                .long("migrate")
                .help("Run database migrations before starting"),
        )
        .get_matches();

    // Decide whether migrations should be run
    let migrate = args.is_present("migrate");
    if migrate {
        println!("Running database migrations...");
        data::run_migrations().unwrap();
    }

    // Initialize application
    let app = rocket::build();

    // Extract additional config values
    let figment = app.figment();
    let config: Config = figment.extract().context("Could not extract config")?;

    // Determine static files dir
    let static_files_dir = config
        .static_files_dir
        .as_deref()
        .unwrap_or(concat!(env!("CARGO_MANIFEST_DIR"), "/static"))
        .to_string();

    // Attach fairings
    let app = app
        .attach(data::Database::fairing())
        .attach(templates::fairing(&config));

    // Register custom error catchers
    let app = app.register("/", catchers![service_not_available]);

    // Attach routes
    let app = app
        // Main routes
        .mount(
            "/",
            routes![
                index,
                flights::list,
                flights::list_nologin,
                flights::flight,
                flights::edit_form,
                flights::edit,
                flights::delete_form,
                flights::delete,
                flights::igc_download,
                flights::submit_form,
                flights::submit_form_nologin,
                flights::submit,
                gliders::list,
                gliders::list_nologin,
                gliders::add_form,
                gliders::add_form_nologin,
                gliders::add,
                gliders::edit_form,
                gliders::edit,
                locations::view,
                locations::list,
                locations::list_nologin,
                locations::add_form,
                locations::add_form_nologin,
                locations::add,
                locations::edit_form,
                locations::edit,
                locations::delete_form,
                locations::delete,
                process_igc::process_igc,
                profile::view,
                profile::view_nologin,
                stats::stats,
                stats::stats_nologin,
            ],
        )
        // Auth routes
        .mount("/", auth::get_routes())
        // Static files
        .mount("/static", FileServer::from(static_files_dir));

    // Launch app
    app.ignite().await?.launch().await?;
    Ok(())
}
