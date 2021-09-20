//! Profile views.

use rocket::{get, request::FlashMessage, response::Redirect};
use rocket_dyn_templates::Template;

use crate::auth;

#[get("/profile")]
pub fn view(user: auth::AuthUser, flash: Option<FlashMessage>) -> Template {
    let context = auth::UserContext::with_flash(user.into_inner(), flash);
    Template::render("profile", context)
}

#[get("/profile", rank = 2)]
pub fn view_nologin() -> Redirect {
    Redirect::to("/auth/login")
}
