use diesel::Queryable;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
}
