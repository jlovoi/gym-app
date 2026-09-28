# Running the app

`docker run -d --name gym-pg -e POSTGRES_PASSWORD=postgres -p 5432:5432 postgres:16`
`cargo run`

# Auth

Login is OAuth (Google and Apple) handled by this backend, which then issues its own
JWTs. Profile data lives on `users`; `user_identities` maps each provider account to a
user, and logins with the same verified email are linked to the same user.

Flow for the app:

1. Open `GET /auth/{google|apple}/start?redirect_to=gymfrontend://auth/callback` with
   `WebBrowser.openAuthSessionAsync`.
2. On success the browser lands on
   `gymfrontend://auth/callback#access_token=…&refresh_token=…&expires_in=…&token_type=Bearer`,
   or `…#error=access_denied|login_failed`.
3. Send `Authorization: Bearer <access_token>` on API calls. When it expires,
   `POST /auth/refresh` with `{"refresh_token": "…"}` for a new pair.

Leave out `redirect_to` to get the tokens back as JSON, handy for testing in a browser.

## Environment

| Variable | Required | Notes |
| --- | --- | --- |
| `JWT_SECRET` | yes | ≥ 32 random bytes, e.g. `openssl rand -base64 48` |
| `PUBLIC_URL` | prod | Backend's public URL; callbacks are `{PUBLIC_URL}/auth/{provider}/callback`. Defaults to `http://localhost:$PORT` |
| `AUTH_ALLOWED_REDIRECTS` | no | Comma-separated exact-match list. Defaults to `gymfrontend://auth/callback`; add your `exp://…/--/auth/callback` URL for Expo Go |
| `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` | for Google | A *Web application* OAuth client with the callback URL above registered |
| `APPLE_CLIENT_ID` | for Apple | The Services ID (not the App ID) |
| `APPLE_TEAM_ID`, `APPLE_KEY_ID`, `APPLE_PRIVATE_KEY` | for Apple | Sign in with Apple key; the PEM may use literal `\n`s |
| `ACCESS_TOKEN_TTL_SECS`, `REFRESH_TOKEN_TTL_SECS` | no | Default 15 min / 30 days |

Apple only accepts `https` callback URLs, so test Apple against the deployed backend
(or a tunnel).
