-- Restore proc_users_update function and trigger

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
