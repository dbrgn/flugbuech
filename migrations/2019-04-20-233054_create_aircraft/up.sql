CREATE TABLE aircraft (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    model VARCHAR(255) NOT NULL,
    manufacturer VARCHAR(255) NOT NULL DEFAULT ''
)
