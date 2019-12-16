ALTER TABLE aircraft
    RENAME TO gliders;
ALTER TABLE flights
    RENAME COLUMN aircraft_id TO glider_id;
ALTER TABLE users
    RENAME COLUMN last_aircraft_id TO last_glider_id;
