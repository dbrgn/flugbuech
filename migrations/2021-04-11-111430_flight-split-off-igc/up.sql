-- Create separate table for IGC files
CREATE TABLE igcs (
    flight_id INTEGER PRIMARY KEY REFERENCES flights(id) ON DELETE CASCADE,
    data BYTEA NOT NULL
);

-- Move data to dedicated igc table
INSERT INTO igcs
SELECT id as flight_id, igc as data
FROM flights
WHERE igc IS NOT NULL;

-- Drop old column
ALTER TABLE flights DROP COLUMN igc;
