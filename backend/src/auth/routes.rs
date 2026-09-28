//! OAuth login flow.
//!
//! 1. The app opens `GET /auth/{provider}/start?redirect_to=gymfrontend://auth/callback`
//!    in a browser (e.g. `WebBrowser.openAuthSessionAsync`).
//! 2. We redirect to the provider with PKCE and a signed `state`.
//! 3. The provider sends the user back to `/auth/{provider}/callback`. We exchange the
//!    code, find or create the row in `users`, and issue our own access/refresh tokens.
//! 4. We redirect to `redirect_to#access_token=...&refresh_token=...&expires_in=...`,
//!    or `redirect_to#error=...` if the login failed.
//!
//! Without `redirect_to`, step 4 returns the tokens as JSON instead, which is handy
//! for trying the flow out in a desktop browser.

use axum::extract::{Form, Path, Query, State};
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::auth::jwt::{FlowClaims, SessionTokens, TokenType};
use crate::auth::{Provider, apple, google, pkce};
use crate::db::users as users_db;
use crate::error::AppError;
use crate::state::AppState;

const FLOW_TTL_SECS: i64 = 600;
const VERIFIER_COOKIE: &str = "oauth_pkce";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/{provider}/start", get(start))
        .route("/auth/google/callback", get(google_callback))
        // Apple posts the result back as a form because we ask for name/email scopes.
        .route("/auth/apple/callback", post(apple_callback))
        .route("/auth/refresh", post(refresh))
}

fn callback_url(state: &AppState, provider: Provider) -> String {
    format!("{}/auth/{}/callback", state.config.public_url, provider)
}

// ── Start ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct StartQuery {
    redirect_to: Option<String>,
}

async fn start(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<StartQuery>,
) -> Result<Response, AppError> {
    let provider: Provider = provider.parse()?;

    let (redirect_to, respond_json) = match query.redirect_to {
        Some(url) if state.config.auth.allowed_redirects.contains(&url) => (url, false),
        Some(url) => return Err(AppError::BadRequest(format!("redirect_to not allowed: {url}"))),
        None => (String::new(), true),
    };

    let pkce = pkce::generate();
    let now = Utc::now().timestamp();
    let flow_state = state.jwt.issue_state(&FlowClaims {
        code_challenge: pkce.challenge.clone(),
        provider: provider.to_string(),
        redirect_to,
        respond_json,
        iat: now,
        exp: now + FLOW_TTL_SECS,
    })?;

    let redirect_uri = callback_url(&state, provider);
    let url = match provider {
        Provider::Google => {
            let cfg = state.config.auth.google.as_ref().ok_or(AppError::NotFound)?;
            google::authorize_url(cfg, &redirect_uri, &flow_state, &pkce.challenge)
        }
        Provider::Apple => {
            let cfg = state.config.auth.apple.as_ref().ok_or(AppError::NotFound)?;
            apple::authorize_url(cfg, &redirect_uri, &flow_state, &pkce.challenge)
        }
    };

    // The PKCE verifier stays in this browser's cookie jar, never in a URL. It also
    // does the job of a CSRF cookie: the signed `state` alone proves *we* started a
    // flow, not that *this browser* did, so without the matching verifier someone could
    // log a victim into the attacker's account by sending them the attacker's callback link.
    let cookie = verifier_cookie(&state, &pkce.verifier, FLOW_TTL_SECS);
    Ok(([(SET_COOKIE, cookie)], Redirect::to(&url)).into_response())
}

fn verifier_cookie(state: &AppState, value: &str, max_age: i64) -> HeaderValue {
    // Apple's form_post callback is a cross-site POST, which only carries the cookie
    // with SameSite=None (and that requires Secure, i.e. https).
    let attrs = if state.config.public_url.starts_with("https://") {
        "Secure; SameSite=None"
    } else {
        "SameSite=Lax"
    };
    let cookie = format!("{VERIFIER_COOKIE}={value}; Path=/auth; Max-Age={max_age}; HttpOnly; {attrs}");
    HeaderValue::from_str(&cookie).expect("cookie is ascii")
}

fn read_cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
}

// ── Callbacks ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GoogleCallback {
    state: String,
    code: Option<String>,
    error: Option<String>,
}

async fn google_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<GoogleCallback>,
) -> Result<Response, AppError> {
    let (flow, verifier) = verify_flow(&state, &headers, &params.state, Provider::Google)?;

    let result = async {
        let code = provider_code(params.code, params.error)?;
        let cfg = state.config.auth.google.as_ref().ok_or(AppError::NotFound)?;
        google::exchange_and_fetch_user(
            &state.http,
            cfg,
            &code,
            &callback_url(&state, Provider::Google),
            &verifier,
        )
        .await
    }
    .await;

    finish(&state, &flow, Provider::Google, result).await
}

#[derive(Deserialize)]
struct AppleCallback {
    state: String,
    code: Option<String>,
    error: Option<String>,
    /// JSON, only sent on the user's first authorization.
    user: Option<String>,
}

#[derive(Deserialize)]
struct AppleUserField {
    name: Option<AppleName>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppleName {
    first_name: Option<String>,
    last_name: Option<String>,
}

async fn apple_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(params): Form<AppleCallback>,
) -> Result<Response, AppError> {
    let (flow, verifier) = verify_flow(&state, &headers, &params.state, Provider::Apple)?;

    let name = params
        .user
        .as_deref()
        .and_then(|json| serde_json::from_str::<AppleUserField>(json).ok())
        .and_then(|user| user.name);
    let (first_name, last_name) = match name {
        Some(name) => (name.first_name, name.last_name),
        None => (None, None),
    };

    let result = async {
        let code = provider_code(params.code, params.error)?;
        let cfg = state.config.auth.apple.as_ref().ok_or(AppError::NotFound)?;
        apple::exchange_and_fetch_user(
            &state.http,
            cfg,
            &code,
            &callback_url(&state, Provider::Apple),
            &verifier,
            first_name,
            last_name,
        )
        .await
    }
    .await;

    finish(&state, &flow, Provider::Apple, result).await
}

/// Returns the flow and this browser's PKCE verifier.
///
/// Errors here mean the callback wasn't part of a login this browser started, so we
/// can't trust `redirect_to` and fail with a plain 400 instead of redirecting.
fn verify_flow(
    state: &AppState,
    headers: &HeaderMap,
    token: &str,
    provider: Provider,
) -> Result<(FlowClaims, String), AppError> {
    let flow = state.jwt.verify_state(token)?;
    if flow.provider != provider.as_str() {
        return Err(AppError::BadRequest("oauth state is for another provider".into()));
    }
    let verifier = read_cookie(headers, VERIFIER_COOKIE)
        .filter(|verifier| pkce::challenge_for(verifier) == flow.code_challenge)
        .ok_or_else(|| AppError::BadRequest("oauth login was started in another browser".into()))?;
    Ok((flow, verifier.to_string()))
}

fn provider_code(code: Option<String>, error: Option<String>) -> Result<String, AppError> {
    match (code, error) {
        (_, Some(error)) => Err(AppError::BadRequest(format!("provider returned error: {error}"))),
        (Some(code), None) => Ok(code),
        (None, None) => Err(AppError::BadRequest("missing authorization code".into())),
    }
}

async fn finish(
    state: &AppState,
    flow: &FlowClaims,
    provider: Provider,
    external: Result<super::ExternalUser, AppError>,
) -> Result<Response, AppError> {
    let tokens = match external {
        Ok(external) => {
            let user = users_db::find_or_create_for_login(&state.db, provider, &external).await;
            match user {
                Ok(user) => state.jwt.issue_session(user.id).map(TokenResponse::from),
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => Err(e),
    };

    let clear_cookie = [(SET_COOKIE, verifier_cookie(state, "", 0))];

    if flow.respond_json {
        return Ok((clear_cookie, Json(tokens?)).into_response());
    }

    let fragment = match tokens {
        Ok(t) => format!(
            "access_token={}&refresh_token={}&expires_in={}&token_type={}",
            t.access_token, t.refresh_token, t.expires_in, t.token_type
        ),
        Err(e) => {
            tracing::warn!(%provider, "oauth login failed: {e}");
            let code = match e {
                AppError::BadRequest(msg) if msg.contains("access_denied") || msg.contains("user_cancelled") => {
                    "access_denied"
                }
                _ => "login_failed",
            };
            format!("error={code}")
        }
    };

    let target = format!("{}#{}", flow.redirect_to, fragment);
    Ok((clear_cookie, Redirect::to(&target)).into_response())
}

// ── Tokens ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    token_type: &'static str,
}

impl From<SessionTokens> for TokenResponse {
    fn from(t: SessionTokens) -> Self {
        Self {
            access_token: t.access_token,
            refresh_token: t.refresh_token,
            expires_in: t.expires_in,
            token_type: "Bearer",
        }
    }
}

#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let claims = state.jwt.verify_session(&body.refresh_token, TokenType::Refresh)?;
    // Deleted users can't keep minting tokens.
    users_db::find_by_id(&state.db, claims.sub)
        .await?
        .ok_or(AppError::Unauthorized)?;
    Ok(Json(state.jwt.issue_session(claims.sub)?.into()))
}
