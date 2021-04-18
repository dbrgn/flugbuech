-- Re-add column
ALTER TABLE flights
    ADD COLUMN igc BYTEA NULL;

-- Merge tables
UPDATE flights
SET igc = igcs.data
FROM igcs
WHERE igcs.flight_id = flights.id;

-- Drop old table
DROP TABLE igcs;
