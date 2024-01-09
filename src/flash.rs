use serde::Serialize;

#[derive(Serialize)]
pub struct FlashMessage {
    pub name: String,
    pub msg: String,
}

impl<'a> From<rocket::request::FlashMessage<'a>> for FlashMessage {
    fn from(f: rocket::request::FlashMessage) -> Self {
        FlashMessage {
            name: f.kind().to_string(),
            msg: f.message().to_string(),
        }
    }
}
