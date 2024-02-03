-- Add down migration script here
DROP TABLE IF EXISTS weekly_scaling;

DROP TABLE IF EXISTS parameter_links;

DROP TABLE IF EXISTS workout_parameters;

DROP TABLE IF EXISTS workout_instances;

DROP TYPE IF EXISTS workout_parameter_type;

