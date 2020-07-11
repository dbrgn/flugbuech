use std::collections::HashMap;

use serde::Serialize;

#[derive(Serialize)]
pub struct FlashMessage {
    pub name: String,
    pub msg: String,
}

impl<'a, 'r> From<rocket::request::FlashMessage<'a, 'r>> for FlashMessage {
    fn from(f: rocket::request::FlashMessage) -> Self {
        FlashMessage {
            name: f.name().to_string(),
            msg: f.msg().to_string(),
        }
    }
}

/// Return a HashMap that contains the key "flashes" if a flash message is
/// provided.
pub fn context_from_flash_opt(
    flash: Option<rocket::request::FlashMessage>,
) -> HashMap<&'static str, Vec<FlashMessage>> {
    let mut context = HashMap::new();
    if let Some(f) = flash {
        context.insert("flashes", vec![crate::flash::FlashMessage::from(f)]);
    }
    context
}

pub fn flashes_from_flash_opt(flash: Option<rocket::request::FlashMessage>) -> Vec<FlashMessage> {
    match flash {
        Some(f) => vec![f.into()],
        None => vec![],
    }
}
