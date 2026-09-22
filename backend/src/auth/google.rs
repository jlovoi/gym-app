use serde::Deserialize;

use crate::{config::GoogleConfig, error::AppError};

use super::ExternalUser;

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://www.googleapis.com/oauth2/v3/token";
const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";
const SCOPES: &str = "openid email profile";

pub fn authorize_url(
    cfg: &GoogleConfig,
    redirect_uri: &str,
    state: &str,
    code_challenge: &str,
) -> String {
    let mut url = reqwest::Url::parse(AUTH_URL).expect("static url is valid");
    url.query_pairs_mut()
        .append_pair("client_id", &cfg.client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", SCOPES)
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256")
        // Ask for a refresh token even if the user has already consented before.
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent");
    url.to_string()
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    sub: String,
    email: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    email_verified: bool,
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
        return Err(AppError::Upstream(format!(
            "google token exchange failed: {body}"
        )));
    }
    let token_res: TokenResponse = response.json().await?;

    let response = http
        .get(USERINFO_URL)
        .bearer_auth(&token_res.access_token)
        .send()
        .await?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::Upstream(format!(
            "google userinfo request failed: {body}"
        )));
    }
    let user_info: UserInfo = response.json().await?;

    if !user_info.email_verified {
        return Err(AppError::Upstream(
            "google account email is not verified".to_string(),
        ));
    }

    Ok(ExternalUser {
        provider_user_id: user_info.sub,
        email: user_info.email,
        name: user_info.name,
    })
}
