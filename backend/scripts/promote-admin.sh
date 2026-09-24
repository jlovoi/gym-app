#!/usr/bin/env bash
set -euo pipefail

# Load .env if present
if [[ -f .env ]]; then
  set -a; source .env; set +a
fi

EMAIL="${1:-}"
if [[ -z "$EMAIL" ]]; then
  echo "Usage: ./scripts/promote-admin.sh <email>"
  exit 1
fi

# Look up user in Clerk by email
ENCODED_EMAIL=$(python3 -c "import urllib.parse; print(urllib.parse.quote('$EMAIL'))")
RESPONSE=$(curl -s "https://api.clerk.com/v1/users?email_address=$ENCODED_EMAIL&limit=1" \
  -H "Authorization: Bearer ${CLERK_SECRET_KEY:?Set CLERK_SECRET_KEY in .env}")

USER_ID=$(echo "$RESPONSE" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d[0]['id'] if d else '')" 2>/dev/null)

if [[ -z "$USER_ID" ]]; then
  echo "Error: No Clerk user found with email: $EMAIL"
  exit 1
fi

echo "Found Clerk user: $USER_ID"

# Promote to admin in DB (user must already exist)
RESULT=$(psql "$DATABASE_URL" -t -c "
  UPDATE users SET role = 'admin', is_active = true WHERE id = '$USER_ID' RETURNING id;
")

if [[ -z "$(echo "$RESULT" | xargs)" ]]; then
  echo "Error: User $USER_ID not found in database. They need to sign in first."
  exit 1
fi

echo "Done — $EMAIL ($USER_ID) is now an active admin."
