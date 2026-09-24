DROP INDEX IF EXISTS idx_memberships_stripe_sub;
ALTER TABLE memberships ADD CONSTRAINT memberships_stripe_subscription_id_key UNIQUE (stripe_subscription_id);
