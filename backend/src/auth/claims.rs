use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClerkClaims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
    pub nbf: Option<u64>,
    pub iss: String,
    pub azp: Option<String>,
}
