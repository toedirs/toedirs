-- Add down migration script here
DROP INDEX IF EXISTS IX_user_preferences;

DROP INDEX IF EXISTS IX_user_preferences_date;

DROP TABLE IF EXISTS user_preferences;

