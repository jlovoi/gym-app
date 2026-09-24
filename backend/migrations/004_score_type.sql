CREATE TYPE score_type AS ENUM ('time', 'reps', 'weight', 'rounds_reps', 'distance', 'custom');

ALTER TABLE workouts
    ADD COLUMN score_type score_type NOT NULL DEFAULT 'time',
    ADD COLUMN score_label TEXT,
    ADD COLUMN score_config JSONB;
