CREATE TYPE user_role AS ENUM ('member', 'staff', 'admin');
CREATE TYPE membership_status AS ENUM ('active', 'past_due', 'canceled', 'expired');

-- Users (Identity linked to Clerk)
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    stripe_customer_id TEXT,
    role user_role DEFAULT 'member',
    is_active BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Workouts (programming)
CREATE TABLE workouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    date DATE NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Logs (workout results)
CREATE TABLE logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT REFERENCES users(id),
    workout_id UUID REFERENCES workouts(id),
    primary_value DOUBLE PRECISION,
    is_rx BOOLEAN DEFAULT true,
    data JSONB,
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(user_id, workout_id)
);

-- Classes (concrete instances)
CREATE TABLE classes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    date DATE NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    capacity INT NOT NULL DEFAULT 20,
    coach_id TEXT REFERENCES users(id),
    workout_id UUID REFERENCES workouts(id),
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Class sign-ups + attendance
CREATE TABLE class_signups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    class_id UUID REFERENCES classes(id) ON DELETE CASCADE,
    user_id TEXT REFERENCES users(id),
    signed_up_at TIMESTAMPTZ DEFAULT now(),
    attended BOOLEAN DEFAULT false,
    UNIQUE(class_id, user_id)
);

-- Memberships
CREATE TABLE memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT REFERENCES users(id),
    stripe_subscription_id TEXT,
    status membership_status DEFAULT 'active',
    plan_type TEXT NOT NULL,
    classes_remaining INT,
    current_period_end TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes
CREATE INDEX idx_workouts_date ON workouts(date);
CREATE INDEX idx_logs_workout_id ON logs(workout_id);
CREATE INDEX idx_logs_user_id ON logs(user_id);
CREATE INDEX idx_logs_primary_value ON logs(primary_value);
CREATE INDEX idx_classes_date ON classes(date);
CREATE INDEX idx_class_signups_class_id ON class_signups(class_id);
CREATE INDEX idx_memberships_user_id ON memberships(user_id);
