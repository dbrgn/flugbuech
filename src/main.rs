#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, sky!"
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}
