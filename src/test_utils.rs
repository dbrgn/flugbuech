use std::{
    collections::HashMap,
    env,
    sync::{Mutex, MutexGuard},
};

use diesel::{connection::SimpleConnection, pg::PgConnection, prelude::*};
use diesel_migrations::MigrationHarness;
use dotenv;
use lazy_static::lazy_static;
use log::debug;
use rocket::{config::Config, figment::Figment, http::Cookie};

use crate::{
    data::{self, create_user},
    models::User,
};

lazy_static! {
    static ref DB_MUTEX: Mutex<()> = Mutex::new(());
}

pub struct TestUser {
    pub user: User,
    pub password: String,
}

pub struct DbTestContext<'a> {
    /// The database connection.
    pub conn: Mutex<PgConnection>,

    /// A pre-created test user.
    pub testuser1: TestUser,
    /// A pre-created test user.
    pub testuser2: TestUser,

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
        let mut conn = PgConnection::establish(&database_url).expect(&format!(
            "Could not establish database connection with \"{}\"",
            database_url
        ));
        debug!("Connected to test database");

        // Drop all tables
        conn.batch_execute("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
            .expect("Could not clean up test database");

        // Run migrations
        conn.run_pending_migrations(data::MIGRATIONS)
            .expect("Could not run database migrations");

        // Create test user
        let testuser1 = create_user(&mut conn, "testuser1", "user1@example.com", "testpass");
        let testuser2 = create_user(&mut conn, "testuser2", "user2@example.com", "testpass");

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

    fn auth_cookie(&self, user: &TestUser) -> Cookie<'static> {
        Cookie::new(crate::auth::USER_COOKIE_ID, user.user.id.to_string())
    }

    /// Create an auth cookie for testuser1.
    pub fn auth_cookie_user1(&self) -> Cookie<'static> {
        self.auth_cookie(&self.testuser1)
    }

    /// Create an auth cookie for testuser2.
    pub fn auth_cookie_user2(&self) -> Cookie<'static> {
        self.auth_cookie(&self.testuser2)
    }

    /// Create a generic username cookie.
    pub fn username_cookie(&self) -> Cookie<'static> {
        Cookie::new(crate::auth::USER_COOKIE_NAME, "testuser".to_string())
    }
}

pub fn make_test_config() -> rocket::figment::Figment {
    // Load env
    let _ = dotenv::dotenv();

    // Database config
    let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    let mut database = HashMap::new();
    database.insert("url", database_url);

    // Generate figment config
    Figment::from(Config::default())
        .select(Config::DEBUG_PROFILE)
        .merge(("databases.flugbuech", database))
}
