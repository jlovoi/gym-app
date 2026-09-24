# gym backend

goal: make a backend to power a crossfit workout application.

API
1. workouts
  - View workouts
  - Daily workout displayed
2. memberships (payments)
  - Sign up for membership
  - Payment for punchcard/membership
3. classes
  - Schedule view (show coach that day)
  - Sign-ups (give signed-up/capacity count)
  - Sign up for classes
4. workout results
  - individuals log workouts
  - Public/private workout tracking
5. admin
  - Staff/admin view
  - Sign in for classes (coach takes attendance)
  - Modify workouts
  - Modify schedule

all above need:
- Login/auth (roles for member/staff/admin)


One of the most interesting learnings was the data model. Tracking workouts is actually non-trivial.

The database uses a hybrid relational/document model to accommodate variable workout structures while maintaining query performance for analytics.

For workouts specifically:
Relational Columns (`primary_value`): Used for sorting and aggregation (e.g., leaderboards). Stores a normalized numeric value (seconds, pounds, reps).
Document Columns (`data` JSONB): Used for display fidelity. Stores unstructured details (splits, scaling modifications, notes) required for the frontend but not for SQL-level ordering.

```sql
-- Users (Identity linked to Clerk)
CREATE TABLE users (
    id TEXT PRIMARY KEY, -- Clerk User ID
    stripe_customer_id TEXT,
    role user_role DEFAULT 'member', 
    is_active BOOLEAN DEFAULT false
);

-- Workouts (Definitions)
CREATE TABLE workouts (
    id UUID PRIMARY KEY,
    date DATE NOT NULL,
    description TEXT
);

-- Logs (workout results)
CREATE TABLE logs (
    id UUID PRIMARY KEY,
    user_id TEXT REFERENCES users(id),
    workout_id UUID REFERENCES workouts(id),
    
    -- Analytical Column (Indexed for sorting)
    primary_value NUMERIC, 
    is_rx BOOLEAN DEFAULT true,
    
    -- Display Column
    data JSONB, 
    
    UNIQUE(user_id, workout_id)
);
```

crates/stack
- neon database (PostgreSQL)
- clerk for authentication
- stripe for payments
- resend for email notifications
- AWS/cloudflare for deploy
- Axum/tokio
- Tower for middleware
- Sqlx (compile-time checked queries)
- Serde
- Tracing
- Prometheus
