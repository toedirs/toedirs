-- Add up migration script here
CREATE TABLE IF NOT EXISTS workout_instances (
    id bigserial NOT NULL PRIMARY KEY,
    user_id integer NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    workout_template_id integer NOT NULL REFERENCES workout_templates (id) ON DELETE CASCADE,
    start_date timestamp with time zone NOT NULL,
    rrule text NOT NULL,
    active boolean NOT NULL DEFAULT TRUE
);

