ALTER TABLE users
    ADD COLUMN email TEXT UNIQUE;
UPDATE users
    SET email = concat(md5(random()::text), '@example.com')
    WHERE email IS NULL;
ALTER TABLE users
    ALTER COLUMN email SET NOT NULL;
