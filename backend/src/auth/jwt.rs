use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
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
    pub typ: TokenType,
    pub iat: i64,
    pub exp: i64,
}

pub struct SessionTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// Carried in the OAuth `state` param across the round trip to the provider. Signing it
/// keeps the backend stateless (no table of in-flight logins) while still protecting
/// against CSRF: nobody can forge a valid `state` without our key.
///
/// `state` passes through the browser and URLs in plain sight (it's signed, not
/// encrypted), so it must never hold secrets. The PKCE verifier lives in an HttpOnly
/// cookie instead; `state` only holds its public challenge, to tie the two together.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowClaims {
    pub code_challenge: String,
    pub provider: String,
    pub redirect_to: String,
    pub respond_json: bool,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Clone)]
pub struct Jwt {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    // Derived from, but distinct from, the session key so a `state` token can never be
    // accepted as a session token.
    state_encoding_key: EncodingKey,
    state_decoding_key: DecodingKey,
    access_ttl: Duration,
    refresh_ttl: Duration,
}

impl Jwt {
    pub fn new(secret: &[u8], access_ttl_secs: i64, refresh_ttl_secs: i64) -> Self {
        let state_secret = Sha256::new()
            .chain_update(secret)
            .chain_update(b"oauth-state-v1")
            .finalize();

        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            state_encoding_key: EncodingKey::from_secret(&state_secret),
            state_decoding_key: DecodingKey::from_secret(&state_secret),
            access_ttl: Duration::seconds(access_ttl_secs),
            refresh_ttl: Duration::seconds(refresh_ttl_secs),
        }
    }

    pub fn issue_session(&self, user_id: Uuid) -> Result<SessionTokens, AppError> {
        let now = Utc::now();
        let access = SessionClaims {
            sub: user_id,
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
            expires_in: self.access_ttl.num_seconds(),
        })
    }

    pub fn verify_session(&self, token: &str, expected: TokenType) -> Result<SessionClaims, AppError> {
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
        decode::<FlowClaims>(token, &self.state_decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|_| AppError::BadRequest("invalid or expired oauth state".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jwt() -> Jwt {
        Jwt::new(b"test-secret-test-secret-test-secret", 900, 2_592_000)
    }

    fn flow(provider: &str, iat: i64, exp: i64) -> FlowClaims {
        FlowClaims {
            code_challenge: "challenge".into(),
            provider: provider.into(),
            redirect_to: "gymfrontend://auth/callback".into(),
            respond_json: false,
            iat,
            exp,
        }
    }

    #[test]
    fn issues_and_verifies_a_session_pair() {
        let jwt = jwt();
        let user_id = Uuid::new_v4();
        let tokens = jwt.issue_session(user_id).unwrap();

        let access = jwt.verify_session(&tokens.access_token, TokenType::Access).unwrap();
        assert_eq!(access.sub, user_id);
        let refresh = jwt.verify_session(&tokens.refresh_token, TokenType::Refresh).unwrap();
        assert_eq!(refresh.sub, user_id);
    }

    #[test]
    fn rejects_wrong_token_type() {
        let jwt = jwt();
        let tokens = jwt.issue_session(Uuid::new_v4()).unwrap();

        assert!(jwt.verify_session(&tokens.access_token, TokenType::Refresh).is_err());
        assert!(jwt.verify_session(&tokens.refresh_token, TokenType::Access).is_err());
    }

    #[test]
    fn rejects_tampered_token() {
        let jwt = jwt();
        let tokens = jwt.issue_session(Uuid::new_v4()).unwrap();
        let tampered = format!("{}x", tokens.access_token);

        assert!(jwt.verify_session(&tampered, TokenType::Access).is_err());
    }

    #[test]
    fn rejects_token_signed_with_another_secret() {
        let other = Jwt::new(b"another-secret-another-secret-12345", 900, 900);
        let tokens = other.issue_session(Uuid::new_v4()).unwrap();

        assert!(jwt().verify_session(&tokens.access_token, TokenType::Access).is_err());
    }

    #[test]
    fn state_token_is_not_a_session_token() {
        let jwt = jwt();
        let now = Utc::now().timestamp();
        let state_token = jwt.issue_state(&flow("google", now, now + 600)).unwrap();

        assert!(jwt.verify_session(&state_token, TokenType::Access).is_err());
    }

    #[test]
    fn issues_and_verifies_flow_state() {
        let jwt = jwt();
        let now = Utc::now().timestamp();
        let token = jwt.issue_state(&flow("apple", now, now + 600)).unwrap();
        let decoded = jwt.verify_state(&token).unwrap();

        assert_eq!(decoded.provider, "apple");
        assert_eq!(decoded.code_challenge, "challenge");
        assert_eq!(decoded.redirect_to, "gymfrontend://auth/callback");
    }

    #[test]
    fn rejects_expired_state() {
        let jwt = jwt();
        let now = Utc::now().timestamp();
        let token = jwt.issue_state(&flow("google", now - 700, now - 100)).unwrap();

        assert!(jwt.verify_state(&token).is_err());
    }
}
