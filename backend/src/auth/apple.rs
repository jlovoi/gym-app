use chrono::Utc;
use jsonwebtoken::{dangerous::insecure_decode_claims, encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::{config::AppleConfig, error::AppError};

use super::ExternalUser;

const AUTH_URL: &str = "https://appleid.apple.com/auth/authorize";
const TOKEN_URL: &str = "https://appleid.apple.com/auth/token";
const ISSUER: &str = "https://appleid.apple.com";
const SCOPES: &str = "name email";

pub fn authorize_url(
    cfg: &AppleConfig,
    redirect_uri: &str,
    state: &str,
    code_challenge: &str,
) -> String {
    let mut url = reqwest::Url::parse(AUTH_URL).expect("static url is valid");
    url.query_pairs_mut()
        .append_pair("client_id", &cfg.client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("response_type", "code")
        // Apple requires form_post whenever the "name"/"email" scopes are requested.
        .append_pair("response_mode", "form_post")
        .append_pair("scope", SCOPES)
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256");
    url.to_string()
}

#[derive(Serialize)]
struct ClientSecretClaims {
    iss: String,
    iat: i64,
    exp: i64,
    aud: String,
    sub: String,
}

/// Apple doesn't take a static client secret: it must be a fresh ES256-signed JWT,
/// minted with your Sign in with Apple private key. We generate one per token exchange
/// (Apple allows up to 6 months, but there's no benefit to us caching a longer-lived one).
fn client_secret(cfg: &AppleConfig) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = ClientSecretClaims {
        iss: cfg.team_id.clone(),
        iat: now,
        exp: now + 300,
        aud: ISSUER.to_string(),
        sub: cfg.client_id.clone(),
    };

    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(cfg.key_id.clone());

    let key = EncodingKey::from_ec_pem(cfg.private_key_pem.as_bytes())
        .map_err(|err| AppError::BadRequest(format!("invalid APPLE_PRIVATE_KEY: {err}")))?;

    Ok(encode(&header, &claims, &key)?)
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    id_token: String,
}

#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    iss: String,
    aud: String,
    exp: i64,
    sub: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    email_verified: Option<EmailVerified>,
}

// Apple has been observed sending this as either a JSON bool or the string "true"/"false".
#[derive(Debug, Deserialize)]
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

pub async fn exchange_and_fetch_user(
    http: &reqwest::Client,
    cfg: &AppleConfig,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
    form_name: Option<String>,
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
        return Err(AppError::Upstream(format!(
            "apple token exchange failed: {body}"
        )));
    }
    let token_res: TokenResponse = response.json().await?;

    // The id_token comes straight from Apple's token endpoint over TLS -- not via a
    // redirect an attacker could tamper with -- so skipping signature verification here
    // is an accepted simplification. We still pin iss/aud/exp defensively.
    let claims: IdTokenClaims = insecure_decode_claims(&token_res.id_token)
        .map_err(|err| AppError::Upstream(format!("apple id_token could not be parsed: {err}")))?;

    if claims.iss != ISSUER {
        return Err(AppError::Upstream(
            "apple id_token has unexpected issuer".to_string(),
        ));
    }
    if claims.aud != cfg.client_id {
        return Err(AppError::Upstream(
            "apple id_token has unexpected audience".to_string(),
        ));
    }
    if claims.exp < Utc::now().timestamp() {
        return Err(AppError::Upstream(
            "apple id_token is expired".to_string(),
        ));
    }
    if !claims
        .email_verified
        .as_ref()
        .map(EmailVerified::is_true)
        .unwrap_or(false)
    {
        return Err(AppError::Upstream(
            "apple account email is not verified".to_string(),
        ));
    }

    let email = claims
        .email
        .ok_or_else(|| AppError::Upstream("apple id_token missing email".to_string()))?;

    Ok(ExternalUser {
        provider_user_id: claims.sub,
        email,
        name: form_name,
    })
}
