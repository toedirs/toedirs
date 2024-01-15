-- Add up migration script here
CREATE TYPE workout_type AS ENUM (
    'run',
    'strength',
    'cycling',
    'hiking',
    'endurance'
);

CREATE TABLE IF NOT EXISTS workout_templates (
    id bigserial NOT NULL PRIMARY KEY,
    user_id integer NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    template_name text NOT NULL UNIQUE,
    workout_type workout_type NOT NULL
);

