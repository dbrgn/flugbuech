ALTER TABLE users
    ADD COLUMN signed_up TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now();
