CREATE TYPE user_role AS ENUM ('member', 'staff', 'admin');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role user_role NOT NULL DEFAULT 'member',
    is_active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    first_name TEXT,
    last_name TEXT,
    -- Stored lowercased; used to link logins from different providers to one user.
    email TEXT UNIQUE,
    phone TEXT
    -- TODO: gym_id, prof_pic_url, pr_id
);

-- One row per OAuth login a user has used (e.g. Google and Apple can both point at
-- the same user). Profile data lives on `users`; this only maps provider ids to it.
CREATE TABLE user_identities (
    provider TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (provider, provider_user_id)
);

CREATE INDEX user_identities_user_id_idx ON user_identities (user_id);
