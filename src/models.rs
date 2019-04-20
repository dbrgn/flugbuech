use diesel::{Associations, Identifiable, Queryable};

use crate::schema::{users, aircraft};

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[table_name="users"]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(User, foreign_key="pilot_id")]
#[table_name="aircraft"]
pub struct Aircraft {
    pub id: i32,
    pub pilot_id: i32,
    pub name: String,
    pub brand: String,
}
