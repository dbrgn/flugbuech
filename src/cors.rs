use std::borrow::Cow;

use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    Request, Response,
};

enum CorsAllow {
    None,
    All,
    Origin(String),
}

pub struct Cors(CorsAllow);

impl Cors {
    pub fn from_config(cors_allow_origin: Option<&str>) -> Self {
        Self(match cors_allow_origin {
            None => CorsAllow::None,
            Some("*") => CorsAllow::All,
            Some(other) => CorsAllow::Origin(other.to_string()),
        })
    }
}

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: match self.0 {
                CorsAllow::None => "CORS (allow none)",
                CorsAllow::All => "CORS (allow all)",
                CorsAllow::Origin(_) => "CORS (allow configured origin)",
            },
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        let origin_opt: Option<Cow<'static, str>> = match self.0 {
            CorsAllow::None => None,
            CorsAllow::All => Some(Cow::Borrowed("*")),
            CorsAllow::Origin(ref origin) => Some(Cow::Owned(origin.clone())),
        };
        if let Some(origin) = origin_opt {
            response.set_header(Header::new("Access-Control-Allow-Origin", origin));
            response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, OPTIONS"));
            response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
        }
    }
}
