use rocket::fairing::Fairing;
use rocket_dyn_templates::Template;

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
        engines
            .tera
            .register_filter("linebreaksbr", filters::linebreaksbr);
    })
}
