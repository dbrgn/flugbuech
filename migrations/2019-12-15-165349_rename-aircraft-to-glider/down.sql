ALTER TABLE users
    RENAME COLUMN last_glider_id TO last_aircraft_id;
ALTER TABLE flights
    RENAME COLUMN glider_id TO aircraft_id;
ALTER TABLE gliders
    RENAME TO aircraft;
