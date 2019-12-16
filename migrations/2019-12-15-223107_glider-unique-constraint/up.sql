ALTER TABLE gliders
    ADD CONSTRAINT unique_per_user UNIQUE (user_id, manufacturer, model);
