CREATE UNIQUE INDEX idx_memberships_stripe_sub ON memberships(stripe_subscription_id)
    WHERE stripe_subscription_id IS NOT NULL;
