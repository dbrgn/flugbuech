ALTER TABLE locations
    ADD COLUMN user_id INTEGER NULL REFERENCES users(id) ON DELETE CASCADE;

UPDATE locations
    SET user_id = 1;

ALTER TABLE locations
    ALTER COLUMN user_id SET NOT NULL;
