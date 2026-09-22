use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionClaims {
    pub sub: Uuid,
    pub email: String,
    pub typ: TokenType,
    pub iat: i64,
    pub exp: i64,
}

pub struct SessionTokens {
    pub access_token: String,
    pub refresh_token: String,
}

/// The short-lived, self-contained payload carried in the OAuth `state` param across the
/// redirect to the provider and back. Signing it lets the backend stay stateless (no
/// server-side store for in-flight logins) while still getting CSRF protection: an
/// attacker cannot forge a valid `state` without this server's signing key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowClaims {
    pub nonce: String,
    pub provider: String,
    pub pkce_verifier: String,
    pub redirect_to: Option<String>,
    pub respond_json: bool,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Clone)]
pub struct Jwt {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    // Signed with a key derived from, but distinct from, the session key so a leaked or
    // replayed in-flight "state" token can never be mistaken for a real session token.
    state_encoding_key: EncodingKey,
    state_decoding_key: DecodingKey,
    access_ttl: Duration,
    refresh_ttl: Duration,
}

impl Jwt {
    pub fn new(secret: &[u8], access_ttl_secs: i64, refresh_ttl_secs: i64) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(secret);
        hasher.update(b"oauth-state-v1");
        let state_secret = hasher.finalize();

        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            state_encoding_key: EncodingKey::from_secret(&state_secret),
            state_decoding_key: DecodingKey::from_secret(&state_secret),
            access_ttl: Duration::seconds(access_ttl_secs),
            refresh_ttl: Duration::seconds(refresh_ttl_secs),
        }
    }

    pub fn issue_session(&self, user_id: Uuid, email: &str) -> Result<SessionTokens, AppError> {
        let now = Utc::now();
        let access = SessionClaims {
            sub: user_id,
            email: email.to_string(),
            typ: TokenType::Access,
            iat: now.timestamp(),
            exp: (now + self.access_ttl).timestamp(),
        };
        let refresh = SessionClaims {
            typ: TokenType::Refresh,
            exp: (now + self.refresh_ttl).timestamp(),
            ..access.clone()
        };

        Ok(SessionTokens {
            access_token: encode(&Header::default(), &access, &self.encoding_key)?,
            refresh_token: encode(&Header::default(), &refresh, &self.encoding_key)?,
        })
    }

    pub fn verify_session(
        &self,
        token: &str,
        expected: TokenType,
    ) -> Result<SessionClaims, AppError> {
        let data = decode::<SessionClaims>(token, &self.decoding_key, &Validation::default())
            .map_err(|_| AppError::Unauthorized)?;
        if data.claims.typ != expected {
            return Err(AppError::Unauthorized);
        }
        Ok(data.claims)
    }

    pub fn issue_state(&self, claims: &FlowClaims) -> Result<String, AppError> {
        Ok(encode(&Header::default(), claims, &self.state_encoding_key)?)
    }

    pub fn verify_state(&self, token: &str) -> Result<FlowClaims, AppError> {
        let data = decode::<FlowClaims>(token, &self.state_decoding_key, &Validation::default())
            .map_err(|_| AppError::BadRequest("invalid or expired oauth state".to_string()))?;
        Ok(data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jwt() -> Jwt {
        Jwt::new(b"test-secret", 900, 2_592_000)
    }

    #[test]
    fn issues_and_verifies_a_session_pair() {
        let jwt = jwt();
        let user_id = Uuid::new_v4();
        let tokens = jwt.issue_session(user_id, "user@example.com").unwrap();

        let access = jwt
            .verify_session(&tokens.access_token, TokenType::Access)
            .unwrap();
        assert_eq!(access.sub, user_id);
        assert_eq!(access.email, "user@example.com");

        let refresh = jwt
            .verify_session(&tokens.refresh_token, TokenType::Refresh)
            .unwrap();
        assert_eq!(refresh.sub, user_id);
    }

    #[test]
    fn rejects_wrong_token_type() {
        let jwt = jwt();
        let tokens = jwt.issue_session(Uuid::new_v4(), "user@example.com").unwrap();

        assert!(jwt
            .verify_session(&tokens.access_token, TokenType::Refresh)
            .is_err());
        assert!(jwt
            .verify_session(&tokens.refresh_token, TokenType::Access)
            .is_err());
    }

    #[test]
    fn rejects_tampered_token() {
        let jwt = jwt();
        let tokens = jwt.issue_session(Uuid::new_v4(), "user@example.com").unwrap();
        let tampered = format!("{}x", tokens.access_token);

        assert!(jwt.verify_session(&tampered, TokenType::Access).is_err());
    }

    #[test]
    fn session_and_state_keys_are_not_interchangeable() {
        let jwt = jwt();
        let flow = FlowClaims {
            nonce: "nonce".to_string(),
            provider: "google".to_string(),
            pkce_verifier: "verifier".to_string(),
            redirect_to: None,
            respond_json: false,
            iat: Utc::now().timestamp(),
            exp: Utc::now().timestamp() + 600,
        };
        let state_token = jwt.issue_state(&flow).unwrap();

        // A state token must never verify as a session token, even though both are HS256
        // JWTs signed by the same `Jwt` instance.
        assert!(jwt
            .verify_session(&state_token, TokenType::Access)
            .is_err());
    }

    #[test]
    fn issues_and_verifies_flow_state() {
        let jwt = jwt();
        let flow = FlowClaims {
            nonce: "nonce".to_string(),
            provider: "apple".to_string(),
            pkce_verifier: "verifier".to_string(),
            redirect_to: Some("myapp://callback".to_string()),
            respond_json: true,
            iat: Utc::now().timestamp(),
            exp: Utc::now().timestamp() + 600,
        };
        let token = jwt.issue_state(&flow).unwrap();
        let decoded = jwt.verify_state(&token).unwrap();

        assert_eq!(decoded.provider, "apple");
        assert_eq!(decoded.pkce_verifier, "verifier");
        assert_eq!(decoded.redirect_to.as_deref(), Some("myapp://callback"));
        assert!(decoded.respond_json);
    }

    #[test]
    fn rejects_expired_state() {
        let jwt = jwt();
        let now = Utc::now().timestamp();
        let flow = FlowClaims {
            nonce: "nonce".to_string(),
            provider: "google".to_string(),
            pkce_verifier: "verifier".to_string(),
            redirect_to: None,
            respond_json: false,
            iat: now - 700,
            exp: now - 100,
        };
        let token = jwt.issue_state(&flow).unwrap();

        assert!(jwt.verify_state(&token).is_err());
    }
}
