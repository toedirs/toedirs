-- Add up migration script here
CREATE TABLE IF NOT EXISTS workout_exclusion_dates (
    workout_instance_id integer NOT NULL REFERENCES workout_instances (id) ON DELETE CASCADE,
    exclusion_date timestamp with time zone NOT NULL
);

