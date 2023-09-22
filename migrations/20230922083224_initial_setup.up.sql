-- Add up migration script here


CREATE TABLE IF NOT EXISTS users (
  id         SERIAL NOT NULL PRIMARY KEY,
  username   TEXT NOT NULL UNIQUE,
  password   TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS user_permissions (
    user_id  INTEGER NOT NULL,
    token    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS activities  (
    id SERIAL PRIMARY KEY,
    user_id integer NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    start_time timestamp with time zone NOT NULL,
    end_time timestamp with time zone NOT NULL,
    duration NUMERIC(8,1) NOT NULL,
    device varchar(255)
);

CREATE TABLE IF NOT EXISTS sessions (
    id SERIAL PRIMARY KEY,
    activity_id integer NOT NULL REFERENCES activities (id) ON DELETE CASCADE,
    start_time timestamp with time zone NOT NULL,
    end_time timestamp with time zone NOT NULL,
    duration NUMERIC(8,1) NOT NULL,
    sport varchar(50) NOT NULL,
    calories smallint
);

CREATE TABLE IF NOT EXISTS laps (
    id SERIAL PRIMARY KEY,
    activity_id integer NOT NULL REFERENCES activities (id) ON DELETE CASCADE,
    start_time timestamp with time zone NOT NULL,
    end_time timestamp with time zone NOT NULL,
    duration NUMERIC(8,1) NOT NULL,
    distance NUMERIC(9, 2),
    calories smallint,
    average_heartrate smallint,
    min_heartrate smallint,
    max_heartrate smallint,
    average_speed NUMERIC(5,3),
    min_speed NUMERIC(5,3),
    max_speed NUMERIC(5,3),
    sport varchar(100),
    ascent smallint,
    descent smallint,
    average_power smallint
);


CREATE TABLE IF NOT EXISTS records (
    id SERIAL PRIMARY KEY,
    activity_id integer NOT NULL REFERENCES activities (id) ON DELETE CASCADE,
    session_id integer REFERENCES sessions (id),
    lap_id integer  REFERENCES laps (id),
    date_recorded timestamp with time zone NOT NULL,
    coordinates point,
    distance NUMERIC(9, 2),
    altitude NUMERIC(6,1),
    heartrate smallint,
    speed NUMERIC(5,3),
    cadence smallint,
    power smallint,
    step_length NUMERIC(5,1),
    pace NUMERIC(5,3)
);


 CREATE TABLE IF NOT EXISTS events (
    id SERIAL PRIMARY KEY,
    activity_id integer NOT NULL REFERENCES activities (id) ON DELETE CASCADE,
    session_id integer REFERENCES sessions (id),
    lap_id integer REFERENCES laps (id),
    date_recorded timestamp with time zone NOT NULL,
    event_type varchar(100) NOT NULL
);
