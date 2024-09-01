-- Add up migration script here
CREATE TYPE heartrate_zone AS ENUM (
    'zone1',
    'zone2',
    'zone3'
);

CREATE TABLE slope_speed (
    id bigserial NOT NULL PRIMARY KEY,
    user_id integer NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    activity_id integer NOT NULL REFERENCES activities (id) ON DELETE CASCADE,
    start_time timestamp with time zone,
    sport varchar(50),
    slope numeric(3, 2) NOT NULL,
    average_speed numeric(5, 3) NOT NULL,
    heartrate_zone heartrate_zone NOT NULL
);


/*
Fix activity start times that had wrong timezones
 */
UPDATE
    activities a
SET
    start_time = "start",
    end_time = "end"
FROM (
    SELECT
        min(r.timestamp) AS "start",
        max(r.timestamp) AS "end",
        r.activity_id
    FROM
        records r
    GROUP BY
        r.activity_id) r
WHERE
    r.activity_id = a.id;

INSERT INTO slope_speed (user_id, activity_id, start_time, sport, slope, average_speed, heartrate_zone)
SELECT
    user_id,
    activity_id,
    activity_start AS start_time,
    sport,
    GREATEST (-1.0, LEAST (1.0, slope)) AS slope,
    GREATEST (0.0, LEAST (15.5, avg(speed))) AS avg_speed,
    hr_zone
FROM (
    SELECT
        data.start_time,
        data.user_id,
        ROUND((data.end_alt - data.start_alt) / (data.end_dist - data.start_dist) / 0.05) * 0.05 AS slope,
        data.hr,
        CASE WHEN data.hr < up.aerobic_threshold THEN
            'zone1'::heartrate_zone
        WHEN data.hr >= up.aerobic_threshold THEN
            'zone2'::heartrate_zone
        ELSE
            'zone3'::heartrate_zone
        END AS hr_zone,
        data.speed,
        data.activity_id,
        data.activity_start,
        s.sport
    FROM ( SELECT DISTINCT
            first_value(altitude) OVER (PARTITION BY floor(distance / 100),
                activity_id ORDER BY "timestamp" ASC) AS start_alt,
            last_value(altitude) OVER (PARTITION BY floor(distance / 100),
                activity_id ORDER BY "timestamp" ASC RANGE BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS end_alt,
            first_value(distance) OVER (PARTITION BY floor(distance / 100),
                activity_id ORDER BY "timestamp" ASC) AS start_dist,
            last_value(distance) OVER (PARTITION BY floor(distance / 100),
                activity_id ORDER BY "timestamp" ASC RANGE BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS end_dist,
            avg(heartrate) OVER (PARTITION BY floor(distance / 100),
                activity_id ORDER BY "timestamp" ASC RANGE BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS hr,
            avg(speed) OVER (PARTITION BY floor(distance / 100),
                activity_id ORDER BY "timestamp" ASC RANGE BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS speed,
            first_value("timestamp") OVER (PARTITION BY floor(distance / 100),
                activity_id ORDER BY "timestamp" ASC) AS start_time,
            activity_id,
            activities.start_time AS activity_start,
            activities.user_id
        FROM (
            SELECT
                *
            FROM
                records
            WHERE
                altitude IS NOT NULL
                AND speed > 0) records
        LEFT JOIN activities ON activities.id = records.activity_id
    WHERE
        records."timestamp" > (activities.start_time + interval '10 minutes')
    ORDER BY
        start_time) data
    JOIN user_preferences up ON (up.start_time <= data.start_time
            OR up.start_time IS NULL)
        AND (up.end_time > data.start_time
            OR up.end_time IS NULL)
        AND up.user_id = data.user_id
    JOIN sessions s ON s.start_time <= data.start_time
        AND s.end_time >= data.start_time
        AND s.activity_id = data.activity_id
WHERE
    data.end_alt IS NOT NULL
    AND data.start_alt IS NOT NULL
    AND data.end_dist != data.start_dist) slope_data
GROUP BY
    activity_id,
    user_id,
    activity_start,
    slope,
    hr_zone,
    sport
ORDER BY
    sport,
    slope,
    hr_zone
