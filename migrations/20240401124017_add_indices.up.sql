-- Add up migration script here
CREATE INDEX IF NOT EXISTS IX_templates_user ON workout_templates (user_id);

CREATE INDEX IF NOT EXISTS IX_workout_params_templ_id ON workout_parameters (workout_template_id);

CREATE INDEX IF NOT EXISTS IX_scaling_user ON weekly_scaling (user_id);

CREATE INDEX IF NOT EXISTS IX_scaling_user_date ON weekly_scaling (user_id, year, week);

CREATE INDEX IF NOT EXISTS IX_work_instance_user ON workout_instances (user_id);

CREATE INDEX IF NOT EXISTS IX_work_instance_user_act_date ON workout_instances (user_id, active, start_date);

CREATE INDEX IF NOT EXISTS IX_work_instance_templ ON workout_instances (workout_template_id);

CREATE INDEX IF NOT EXISTS IX_param_links ON parameter_links (parameter_id, instance_id);

CREATE INDEX IF NOT EXISTS IX_user_permissions ON user_permissions (user_id);

CREATE INDEX IF NOT EXISTS IX_activities_user ON activities (user_id);

CREATE INDEX IF NOT EXISTS IX_activities_user_date ON activities (user_id, start_time, end_time);

CREATE INDEX IF NOT EXISTS IX_records_hr ON records (heartrate);

CREATE INDEX IF NOT EXISTS IX_records_activity ON records (activity_id);

CREATE INDEX IF NOT EXISTS IX_laps ON laps (activity_id);

CREATE INDEX IF NOT EXISTS IX_sessions ON sessions (activity_id);

