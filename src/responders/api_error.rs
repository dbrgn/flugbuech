use std::{borrow::Cow, io::Cursor};

use rocket::{
    http::{ContentType, Status},
    response,
    serde::json,
    Request,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum ErrorKind {
    Authentication,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorBody {
    error_kind: ErrorKind,
    message: Cow<'static, str>,
}

pub enum ApiError {
    Authentication,
}

#[rocket::async_trait]
impl<'r> response::Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (status, body) = match self {
            ApiError::Authentication => (
                Status::Unauthorized,
                json::to_string(&ErrorBody {
                    error_kind: ErrorKind::Authentication,
                    message: Cow::Borrowed("Not authenticated, please log in"),
                })
                .unwrap(),
            ),
        };
        response::Response::build()
            .header(ContentType::JSON)
            .status(status)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}
