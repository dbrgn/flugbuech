-- Drop user table triggers. Instead, call crypt manually.
--
-- The problem with the triggers is that they sometimes change the password
-- when that was not intended.

DROP TRIGGER trigger_users_insert ON users;
DROP TRIGGER trigger_users_update ON users;

DROP FUNCTION proc_users_update;
DROP FUNCTION proc_users_insert;
