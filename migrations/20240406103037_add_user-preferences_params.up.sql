-- Add up migration script here
ALTER TABLE user_preferences
    ADD COLUMN tau double precision DEFAULT 0.08097493 NOT NULL,
    ADD COLUMN c double precision DEFAULT 0.000002370473 NOT NULL;

ALTER TABLE activities
    ADD COLUMN avg_heartrate smallint,
    ADD COLUMN LOAD int;

UPDATE
    activities
SET
    avg_heartrate = s.avg_heartrate,
    LOAD = ROUND((0.000002370473 * exp(0.0809749 * s.avg_heartrate) + 1) * s.duration / 60)
FROM (
    SELECT
        AVG(r.heartrate) AS avg_heartrate,
        COUNT(r.id) AS duration,
        r.activity_id
    FROM
        records r
    WHERE
        r.heartrate IS NOT NULL
        AND r.heartrate > 100
    GROUP BY
        r.activity_id) AS s
WHERE
    s.activity_id = activities.id;

