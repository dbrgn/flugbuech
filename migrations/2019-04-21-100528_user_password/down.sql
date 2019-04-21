DROP TRIGGER trigger_users_update ON users RESTRICT;
DROP TRIGGER trigger_users_insert ON users RESTRICT;
DROP FUNCTION proc_users_update RESTRICT;
DROP FUNCTION proc_users_insert RESTRICT;

ALTER TABLE users DROP COLUMN password;
