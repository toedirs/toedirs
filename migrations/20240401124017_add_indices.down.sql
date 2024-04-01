-- Add down migration script here
DROP INDEX IF EXISTS IX_templates_user;

DROP INDEX IF EXISTS IX_workout_params_templ_id;

DROP INDEX IF EXISTS IX_scaling_user;

DROP INDEX IF EXISTS IX_scaling_user_date;

DROP INDEX IF EXISTS IX_work_instance_user;

DROP INDEX IF EXISTS IX_work_instance_user_act_date;

DROP INDEX IF EXISTS IX_work_instance_templ;

DROP INDEX IF EXISTS IX_param_links;

DROP INDEX IF EXISTS IX_user_permissions;

DROP INDEX IF EXISTS IX_activities_user;

DROP INDEX IF EXISTS IX_activities_user_date;

DROP INDEX IF EXISTS IX_records_hr;

DROP INDEX IF EXISTS IX_records_activity;

DROP INDEX IF EXISTS IX_laps;

DROP INDEX IF EXISTS IX_sessions;

