use chrono::Utc;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, decode_header, encode};
use serde::{Deserialize, Serialize};

use crate::config::AppleConfig;
use crate::error::AppError;

use super::ExternalUser;

const AUTH_URL: &str = "https://appleid.apple.com/auth/authorize";
const TOKEN_URL: &str = "https://appleid.apple.com/auth/token";
const KEYS_URL: &str = "https://appleid.apple.com/auth/keys";
const ISSUER: &str = "https://appleid.apple.com";
const SCOPES: &str = "name email";

pub fn authorize_url(cfg: &AppleConfig, redirect_uri: &str, state: &str, code_challenge: &str) -> String {
    let mut url = reqwest::Url::parse(AUTH_URL).expect("static url is valid");
    url.query_pairs_mut()
        .append_pair("client_id", &cfg.client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("response_type", "code")
        // Apple requires form_post whenever the name/email scopes are requested.
        .append_pair("response_mode", "form_post")
        .append_pair("scope", SCOPES)
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256");
    url.to_string()
}

#[derive(Serialize)]
struct ClientSecretClaims<'a> {
    iss: &'a str,
    iat: i64,
    exp: i64,
    aud: &'a str,
    sub: &'a str,
}

/// Apple has no static client secret: it's a short-lived ES256 JWT signed with the
/// Sign in with Apple private key, minted per token exchange.
fn client_secret(cfg: &AppleConfig) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = ClientSecretClaims {
        iss: &cfg.team_id,
        iat: now,
        exp: now + 300,
        aud: ISSUER,
        sub: &cfg.client_id,
    };
    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(cfg.key_id.clone());

    let key = EncodingKey::from_ec_pem(cfg.private_key_pem.as_bytes())
        .map_err(|e| AppError::Internal(format!("invalid APPLE_PRIVATE_KEY: {e}")))?;
    Ok(encode(&header, &claims, &key)?)
}

#[derive(Deserialize)]
struct TokenResponse {
    id_token: String,
}

#[derive(Deserialize)]
struct IdTokenClaims {
    sub: String,
    email: Option<String>,
    email_verified: Option<EmailVerified>,
}

// Apple sends this as either a JSON bool or the string "true"/"false".
#[derive(Deserialize)]
#[serde(untagged)]
enum EmailVerified {
    Bool(bool),
    Str(String),
}

impl EmailVerified {
    fn is_true(&self) -> bool {
        match self {
            EmailVerified::Bool(b) => *b,
            EmailVerified::Str(s) => s == "true",
        }
    }
}

/// Verifies the id_token's signature against Apple's published keys, plus issuer,
/// audience and expiry. Keys are fetched per login; Apple logins are rare enough that
/// caching them isn't worth the complexity yet.
async fn verify_id_token(
    http: &reqwest::Client,
    cfg: &AppleConfig,
    id_token: &str,
) -> Result<IdTokenClaims, AppError> {
    let rejected = |e: jsonwebtoken::errors::Error| AppError::Upstream(format!("apple id_token rejected: {e}"));

    let kid = decode_header(id_token)
        .map_err(rejected)?
        .kid
        .ok_or_else(|| AppError::Upstream("apple id_token has no kid".into()))?;

    let response = http.get(KEYS_URL).send().await?;
    if !response.status().is_success() {
        return Err(AppError::Upstream(format!("apple keys fetch failed: {}", response.status())));
    }
    let keys: JwkSet = response.json().await?;
    let jwk = keys
        .find(&kid)
        .ok_or_else(|| AppError::Upstream(format!("apple id_token signed with unknown key {kid}")))?;
    let key = DecodingKey::from_jwk(jwk).map_err(rejected)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[ISSUER]);
    validation.set_audience(&[&cfg.client_id]);
    validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);

    Ok(decode::<IdTokenClaims>(id_token, &key, &validation).map_err(rejected)?.claims)
}

pub async fn exchange_and_fetch_user(
    http: &reqwest::Client,
    cfg: &AppleConfig,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
    first_name: Option<String>,
    last_name: Option<String>,
) -> Result<ExternalUser, AppError> {
    let secret = client_secret(cfg)?;

    let response = http
        .post(TOKEN_URL)
        .form(&[
            ("client_id", cfg.client_id.as_str()),
            ("client_secret", secret.as_str()),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
            ("code_verifier", code_verifier),
        ])
        .send()
        .await?;
    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::Upstream(format!("apple token exchange failed: {body}")));
    }
    let token: TokenResponse = response.json().await?;

    let claims = verify_id_token(http, cfg, &token.id_token).await?;

    let email = match claims.email {
        Some(email) if claims.email_verified.as_ref().is_some_and(EmailVerified::is_true) => Some(email),
        Some(_) => return Err(AppError::Upstream("apple account email is not verified".into())),
        None => None,
    };

    Ok(ExternalUser {
        provider_user_id: claims.sub,
        email,
        first_name,
        last_name,
    })
}
