use std::{collections::HashMap, sync::Arc};

use rocket::fairing::Fairing;
use rocket_dyn_templates::{
    tera::{self, Value},
    Template,
};

use crate::{filters, Config};

pub fn fairing(config: &Config) -> impl Fairing {
    let config = Arc::new(config.clone());

    Template::custom(move |engines| {
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

        // Register custom functions
        let config = config.clone();
        engines
            .tera
            .register_function("plausible_config", move |_args: &HashMap<String, Value>| {
                // If plausible – a privacy-preserving, non-personal-data-logging,
                // GDPR-compliant visitor stats software – is enabled in the config, return
                // its configuration values.
                if let Config {
                    plausible_domain: Some(ref domain),
                    plausible_url: Some(ref url),
                    ..
                } = *config
                {
                    let mut map = tera::Map::new();
                    map.insert("url".into(), Value::String(url.to_string()));
                    map.insert("domain".into(), Value::String(domain.to_string()));
                    Ok(Value::Object(map))
                } else {
                    Ok(Value::Null)
                }
            });
    })
}
