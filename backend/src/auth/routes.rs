use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    db,
    error::AppError,
    jwt::{FlowClaims, SessionTokens, TokenType},
    pkce,
    state::AppState,
};

use super::{apple, google, ExternalUser, Provider};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/{provider}/login", get(login))
        .route("/auth/google/callback", get(google_callback))
        .route("/auth/apple/callback", post(apple_callback))
        .route("/auth/refresh", post(refresh))
}

#[derive(Debug, Deserialize)]
struct LoginQuery {
    /// Where to send the browser after login completes. Defaults to FRONTEND_REDIRECT_URL.
    redirect_to: Option<String>,
    /// Pass `?format=json` to get the tokens back as a JSON body instead of a redirect --
    /// handy for testing the flow with curl before wiring up a real frontend.
    format: Option<String>,
}

async fn login(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<LoginQuery>,
) -> Result<Response, AppError> {
    let provider: Provider = provider.parse()?;
    let redirect_uri = format!("{}/auth/{provider}/callback", state.config.backend_base_url);

    let pkce = pkce::generate();
    let challenge = pkce.challenge.clone();

    let now = chrono::Utc::now().timestamp();
    let flow = FlowClaims {
        nonce: pkce::random_token(),
        provider: provider.to_string(),
        pkce_verifier: pkce.verifier,
        redirect_to: query.redirect_to,
        respond_json: query.format.as_deref() == Some("json"),
        iat: now,
        exp: now + 600,
    };
    let state_token = state.jwt.issue_state(&flow)?;

    let url = match provider {
        Provider::Google => {
            let cfg = state
                .config
                .google
                .as_ref()
                .ok_or_else(|| AppError::ProviderNotConfigured("google".to_string()))?;
            google::authorize_url(cfg, &redirect_uri, &state_token, &challenge)
        }
        Provider::Apple => {
            let cfg = state
                .config
                .apple
                .as_ref()
                .ok_or_else(|| AppError::ProviderNotConfigured("apple".to_string()))?;
            apple::authorize_url(cfg, &redirect_uri, &state_token, &challenge)
        }
    };

    Ok(Redirect::temporary(&url).into_response())
}

#[derive(Debug, Deserialize)]
struct GoogleCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

async fn google_callback(
    State(state): State<AppState>,
    Query(query): Query<GoogleCallbackQuery>,
) -> Result<Response, AppError> {
    if let Some(err) = query.error {
        return Err(AppError::Upstream(format!("google returned an error: {err}")));
    }
    let code = query
        .code
        .ok_or_else(|| AppError::BadRequest("missing code".to_string()))?;
    let state_token = query
        .state
        .ok_or_else(|| AppError::BadRequest("missing state".to_string()))?;

    let flow = state.jwt.verify_state(&state_token)?;
    if flow.provider != "google" {
        return Err(AppError::BadRequest("state/provider mismatch".to_string()));
    }

    let cfg = state
        .config
        .google
        .as_ref()
        .ok_or_else(|| AppError::ProviderNotConfigured("google".to_string()))?;
    let redirect_uri = format!("{}/auth/google/callback", state.config.backend_base_url);

    let external =
        google::exchange_and_fetch_user(&state.http, cfg, &code, &redirect_uri, &flow.pkce_verifier)
            .await?;

    complete_login(&state, "google", external, flow).await
}

#[derive(Debug, Deserialize)]
struct AppleCallbackForm {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    /// Only present on the user's very first authorization; a JSON string like
    /// `{"name":{"firstName":"...","lastName":"..."},"email":"..."}`.
    user: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AppleNamePayload {
    name: Option<AppleName>,
}

#[derive(Debug, Deserialize)]
struct AppleName {
    #[serde(rename = "firstName")]
    first_name: Option<String>,
    #[serde(rename = "lastName")]
    last_name: Option<String>,
}

async fn apple_callback(
    State(state): State<AppState>,
    Form(form): Form<AppleCallbackForm>,
) -> Result<Response, AppError> {
    if let Some(err) = form.error {
        return Err(AppError::Upstream(format!("apple returned an error: {err}")));
    }
    let code = form
        .code
        .ok_or_else(|| AppError::BadRequest("missing code".to_string()))?;
    let state_token = form
        .state
        .ok_or_else(|| AppError::BadRequest("missing state".to_string()))?;

    let flow = state.jwt.verify_state(&state_token)?;
    if flow.provider != "apple" {
        return Err(AppError::BadRequest("state/provider mismatch".to_string()));
    }

    let cfg = state
        .config
        .apple
        .as_ref()
        .ok_or_else(|| AppError::ProviderNotConfigured("apple".to_string()))?;
    let redirect_uri = format!("{}/auth/apple/callback", state.config.backend_base_url);

    let name = form
        .user
        .as_deref()
        .and_then(|raw| serde_json::from_str::<AppleNamePayload>(raw).ok())
        .and_then(|payload| payload.name)
        .map(|n| {
            [n.first_name, n.last_name]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|s| !s.is_empty());

    let external = apple::exchange_and_fetch_user(
        &state.http,
        cfg,
        &code,
        &redirect_uri,
        &flow.pkce_verifier,
        name,
    )
    .await?;

    complete_login(&state, "apple", external, flow).await
}

async fn complete_login(
    state: &AppState,
    provider: &str,
    external: ExternalUser,
    flow: FlowClaims,
) -> Result<Response, AppError> {
    let user = db::upsert_oauth_user(
        &state.db,
        provider,
        &external.provider_user_id,
        &external.email,
        external.name.as_deref(),
    )
    .await?;

    let tokens = state.jwt.issue_session(user.id, &user.email)?;

    if flow.respond_json {
        return Ok(tokens_json(&tokens).into_response());
    }

    let base = flow
        .redirect_to
        .unwrap_or_else(|| state.config.frontend_redirect_url.clone());
    // JWTs only use base64url characters plus '.', so no percent-encoding is needed here.
    // A fragment (not a query string) keeps the tokens out of server access logs and the
    // Referer header if the app's landing page makes further requests.
    let redirect_url =
        format!("{base}#access_token={}&refresh_token={}&token_type=bearer", tokens.access_token, tokens.refresh_token);

    Ok(Redirect::temporary(&redirect_url).into_response())
}

fn tokens_json(tokens: &SessionTokens) -> Json<serde_json::Value> {
    Json(json!({
        "access_token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
        "token_type": "bearer",
    }))
}

#[derive(Debug, Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = state
        .jwt
        .verify_session(&body.refresh_token, TokenType::Refresh)?;
    let user = db::find_user_by_id(&state.db, claims.sub)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let tokens = state.jwt.issue_session(user.id, &user.email)?;
    Ok(tokens_json(&tokens))
}
