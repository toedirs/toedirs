-- Add up migration script here
CREATE TABLE IF NOT EXISTS workout_instances (
    id bigserial NOT NULL PRIMARY KEY,
    user_id integer NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    workout_template_id integer NOT NULL REFERENCES workout_templates (id) ON DELETE CASCADE,
    start_date timestamp with time zone NOT NULL,
    rrule text NOT NULL,
    active boolean NOT NULL DEFAULT TRUE
);

CREATE TYPE workout_parameter_type AS ENUM (
    'time_s',
    'distance_m',
    'trainingload'
);

CREATE TABLE IF NOT EXISTS workout_parameter (
    id bigserial NOT NULL PRIMARY KEY,
    workout_template_id integer NOT NULL REFERENCES workout_templates (id) ON DELETE CASCADE,
    name text NOT NULL,
    parameter_type workout_parameter_type NOT NULL,
    value integer NOT NULL,
    scaling boolean NOT NULL DEFAULT TRUE,
    position integer NOT NULL
);

CREATE TABLE IF NOT EXISTS parameter_link (
    instance_id integer NOT NULL REFERENCES workout_instances (id) ON DELETE CASCADE,
    parameter_id integer NOT NULL REFERENCES workout_parameter (id) ON DELETE CASCADE,
    value_override integer
);

