-- Load pgcrypto
CREATE EXTENSION IF NOT EXISTS pgcrypto;
COMMENT ON EXTENSION pgcrypto IS 'cryptographic functions';

-- Add password field to users table
ALTER TABLE users
ADD COLUMN password TEXT NULL;

-- On insertion, hash the password with a newly generated salt
CREATE OR REPLACE FUNCTION proc_users_insert() RETURNS trigger AS $$
begin
    new.password := crypt(new.password, gen_salt('bf', 10));
    return new;
end
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_users_insert
BEFORE INSERT ON users
FOR EACH ROW
EXECUTE PROCEDURE proc_users_insert();

-- On password update, hash the new password
CREATE OR REPLACE FUNCTION proc_users_update() RETURNS trigger AS $$
begin
    IF new.password = NULL THEN
        new.password := old.password;
    ELSE
        new.password := crypt(new.password, gen_salt('bf', 10));
    END IF;
    return new;
end
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_users_update
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE PROCEDURE proc_users_update();

-- Generate random password for existing users
UPDATE users
SET password = (
    SELECT gen_random_uuid()::text
    WHERE users.id = users.id
);

-- Make password field non-nullable
ALTER TABLE users
ALTER COLUMN password SET NOT NULL;
