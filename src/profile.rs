//! Profile views.

use rocket::get;
use rocket::response::Redirect;
use rocket_contrib::templates::Template;

use crate::auth;

#[get("/profile")]
pub fn view(user: auth::AuthUser) -> Template {
    let context = auth::UserContext::new(user.into_inner());
    Template::render("profile", context)
}

#[get("/profile", rank = 2)]
pub fn view_nologin() -> Redirect {
    Redirect::to("/auth/login")
}
