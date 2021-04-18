use rocket::fairing::Fairing;
use rocket_contrib::templates::Template;

use crate::filters;

pub fn fairing() -> impl Fairing {
    Template::custom(|engines| {
        // Autoescape HTML and Tera files
        engines.tera.autoescape_on(vec![".html", ".tera"]);

        // Register custom filters
        engines.tera.register_filter("duration", filters::duration);
        engines
            .tera
            .register_filter("xcontest_icon", filters::xcontest_icon);
    })
}
