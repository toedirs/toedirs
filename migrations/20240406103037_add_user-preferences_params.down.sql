-- Add down migration script here
ALTER TABLE user_preferences
    DROP COLUMN IF EXISTS tau,
    DROP COLUMN IF EXISTS c;

ALTER TABLE activities
    DROP COLUMN avg_heartrate,
    DROP COLUMN LOAD;

