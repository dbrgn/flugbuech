use std::env;
use std::sync::{Mutex, MutexGuard};

use diesel::connection::SimpleConnection;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_migrations;
use dotenv;
use lazy_static::lazy_static;
use log::debug;

use crate::data::create_user;
use crate::models::User;

lazy_static! {
    static ref DB_MUTEX: Mutex<()> = Mutex::new(());
}

pub struct TestUser {
    pub user: User,
    pub password: String,
}

pub(crate) struct DbTestContext<'a> {
    /// The database connection.
    pub(crate) conn: Mutex<PgConnection>,

    /// A pre-created test user.
    pub(crate) testuser1: TestUser,
    /// A pre-created test user.
    pub(crate) testuser2: TestUser,

    /// Used to prevent concurrent database access.
    #[allow(dead_code)]
    db_mutex: MutexGuard<'a, ()>,
}

impl<'a> DbTestContext<'a> {
    pub fn new() -> Self {
        // Load env
        let _ = dotenv::dotenv();

        // Lock mutex
        //
        // Because test failures result in a poisoned mutex, we ignore that and
        // restore the regular mutex.
        let db_mutex = match DB_MUTEX.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        // Connect to test database
        debug!("Connecting to test database...");
        let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
        let conn = PgConnection::establish(&database_url).expect(&format!(
            "Could not establish database connection with \"{}\"",
            database_url
        ));
        debug!("Connected to test database");

        // Drop all tables
        conn.batch_execute("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
            .expect("Could not clean up test database");

        // Run migrations
        diesel_migrations::run_pending_migrations(&conn).expect("Could not run database migrations");

        // Create test user
        let testuser1 = create_user(&conn, "testuser1", "testpass", "user1@example.com");
        let testuser2 = create_user(&conn, "testuser2", "testpass", "user2@example.com");

        DbTestContext {
            conn: Mutex::new(conn),
            testuser1: TestUser {
                user: testuser1,
                password: "testpass".into(),
            },
            testuser2: TestUser {
                user: testuser2,
                password: "testpass".into(),
            },
            db_mutex,
        }
    }

    /// Return a connection even if the mutex is poisoned (after another test
    /// failed).
    pub fn force_get_conn(&self) -> MutexGuard<PgConnection> {
        match self.conn.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}
