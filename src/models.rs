use diesel::{Associations, Identifiable, Queryable};
use serde::Serialize;

use crate::schema::{users, aircraft};

#[derive(Identifiable, Queryable, Serialize, PartialEq, Debug, Clone)]
#[table_name="users"]
pub struct User {
    pub id: i32,
    pub username: String,
    /// Password is automatically hashed on insert or update
    /// by a PostgreSQL trigger.
    pub password: String,
}

#[derive(Identifiable, Queryable, Associations, Serialize, PartialEq, Debug, Clone)]
#[belongs_to(User, foreign_key="user_id")]
#[table_name="aircraft"]
pub struct Aircraft {
    pub id: i32,
    pub user_id: i32,
    pub model: String,
    pub manufacturer: String,
}
