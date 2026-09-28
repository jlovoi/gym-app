use serde::Deserialize;

use crate::config::GoogleConfig;
use crate::error::AppError;

use super::ExternalUser;

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";
const SCOPES: &str = "openid email profile";

pub fn authorize_url(cfg: &GoogleConfig, redirect_uri: &str, state: &str, code_challenge: &str) -> String {
    let mut url = reqwest::Url::parse(AUTH_URL).expect("static url is valid");
    url.query_pairs_mut()
        .append_pair("client_id", &cfg.client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", SCOPES)
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("prompt", "select_account");
    url.to_string()
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct UserInfo {
    sub: String,
    email: Option<String>,
    #[serde(default)]
    email_verified: bool,
    given_name: Option<String>,
    family_name: Option<String>,
}

pub async fn exchange_and_fetch_user(
    http: &reqwest::Client,
    cfg: &GoogleConfig,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<ExternalUser, AppError> {
    let response = http
        .post(TOKEN_URL)
        .form(&[
            ("client_id", cfg.client_id.as_str()),
            ("client_secret", cfg.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
            ("code_verifier", code_verifier),
        ])
        .send()
        .await?;
    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::Upstream(format!("google token exchange failed: {body}")));
    }
    let token: TokenResponse = response.json().await?;

    let response = http
        .get(USERINFO_URL)
        .bearer_auth(&token.access_token)
        .send()
        .await?;
    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::Upstream(format!("google userinfo failed: {body}")));
    }
    let info: UserInfo = response.json().await?;

    // Emails are used to link accounts across providers, so only trust verified ones.
    if !info.email_verified {
        return Err(AppError::Upstream("google account email is not verified".into()));
    }

    Ok(ExternalUser {
        provider_user_id: info.sub,
        email: info.email,
        first_name: info.given_name,
        last_name: info.family_name,
    })
}
