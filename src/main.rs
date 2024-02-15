#![allow(clippy::needless_borrow)]

#[macro_use]
extern crate diesel;
extern crate diesel_migrations;

mod auth;
mod base64;
mod cors;
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
mod responders;
mod schema;
mod stats;
mod templates;
#[cfg(test)]
mod test_utils;

use anyhow::{Context, Result};
use clap::{Arg, ArgAction, Command};
use rocket::{
    catch, catchers,
    fs::FileServer,
    get,
    request::{FlashMessage, Request},
    routes,
};
use rocket_dyn_templates::{context, Template};
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

    /// Allowed CORS origin.
    ///
    /// - Set to `*` to allow CORS requests from all origins
    /// - Set to a specific origin to allow CORS requests from that origin
    pub cors_allow_origin: Option<String>,
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

#[get("/privacy-policy")]
async fn privacy_policy() -> Template {
    Template::render("privacy_policy", context! {})
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
    let args = Command::new(NAME)
        .about(DESCRIPTION)
        .version(VERSION)
        .arg(
            Arg::new("migrate")
                .long("migrate")
                .action(ArgAction::SetTrue)
                .help("Run database migrations before starting"),
        )
        .get_matches();

    // Decide whether migrations should be run
    let migrate = args.get_flag("migrate");
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
        .attach(templates::fairing(&config))
        .attach(cors::Cors::from_config(config.cors_allow_origin.as_deref()));

    // Register custom error catchers
    let app = app.register("/", catchers![service_not_available]);

    // Attach routes
    let app = app
        // Main routes
        .mount(
            "/",
            routes![
                index,
                privacy_policy,
                flights::edit_form,
                flights::edit,
                flights::delete_form,
                flights::delete,
                flights::submit_form,
                flights::submit_form_nologin,
                flights::submit,
            ],
        )
        // Auth routes
        .mount("/", auth::get_routes())
        // API routes
        .mount(
            "/api/v1/",
            [
                auth::api_routes(),
                profile::api_routes(),
                stats::api_routes(),
                locations::api_routes(),
                gliders::api_routes(),
                flights::api_routes(),
                process_igc::api_routes(),
            ]
            .concat(),
        )
        // Static files
        .mount("/static", FileServer::from(static_files_dir));

    // Launch app
    app.ignite().await?.launch().await?;
    Ok(())
}
