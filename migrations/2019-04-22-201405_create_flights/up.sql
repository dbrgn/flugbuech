CREATE TABLE locations (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    country CHAR(2) NOT NULL,
    elevation INTEGER NOT NULL
);

CREATE DOMAIN tracktype AS TEXT
CHECK(
   VALUE = 'free_flight'
OR VALUE = 'flat_triangle'
OR VALUE = 'fai_triangle'
);

CREATE DOMAIN url AS TEXT
CHECK(VALUE ~ '^https?://');

CREATE TABLE flights (
    id SERIAL PRIMARY KEY,
    number INTEGER,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    aircraft_id INTEGER REFERENCES aircraft(id) ON DELETE SET NULL,
    launch_at INTEGER REFERENCES locations(id) ON DELETE NO ACTION,
    landing_at INTEGER REFERENCES locations(id) ON DELETE NO ACTION,
    launch_time TIMESTAMP WITH TIME ZONE NULL,
    landing_time TIMESTAMP WITH TIME ZONE NULL,
    track_distance REAL NULL,
    xcontest_tracktype tracktype NULL,
    xcontest_distance REAL NULL,
    xcontest_url url NULL,
    comment TEXT NULL,
    video_url url NULL,
    UNIQUE(user_id, number)
);
