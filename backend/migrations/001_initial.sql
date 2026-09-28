CREATE TYPE user_role AS ENUM ('member', 'staff', 'admin');

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    role user_role NOT NULL DEFAULT 'member',
    is_active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT now(),
    first_name TEXT,
    last_name TEXT,
    email TEXT UNIQUE,
    phone TEXT,
    # gym_id, prof_pic_url, pr_id
);
