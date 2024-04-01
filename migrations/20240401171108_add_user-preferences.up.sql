-- Add up migration script here
CREATE TABLE IF NOT EXISTS user_preferences (
    id bigserial NOT NULL PRIMARY KEY,
    user_id integer NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    start_time timestamp with time zone,
    end_time timestamp with time zone,
    aerobic_threshold int4 NOT NULL,
    anaerobic_threshold int4 NOT NULL,
    max_heartrate int4 NOT NULL
);

CREATE INDEX IF NOT EXISTS IX_user_preferences ON user_preferences (user_id);

CREATE INDEX IF NOT EXISTS IX_user_preferences_date ON user_preferences (user_id, start_time, end_time);

