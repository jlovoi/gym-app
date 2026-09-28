use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct PkcePair {
    pub verifier: String,
    pub challenge: String,
}

/// 32 bytes from the OS RNG (v4 UUIDs are generated with `getrandom`).
fn random_bytes32() -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[..16].copy_from_slice(Uuid::new_v4().as_bytes());
    bytes[16..].copy_from_slice(Uuid::new_v4().as_bytes());
    bytes
}

/// Generates an RFC 7636 PKCE verifier/challenge pair (S256).
pub fn generate() -> PkcePair {
    let verifier = URL_SAFE_NO_PAD.encode(random_bytes32());
    let challenge = challenge_for(&verifier);
    PkcePair { verifier, challenge }
}

/// The S256 challenge for a verifier: base64url(sha256(verifier)).
pub fn challenge_for(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_matches_sha256_of_verifier() {
        let pair = generate();
        let expected = URL_SAFE_NO_PAD.encode(Sha256::digest(pair.verifier.as_bytes()));
        assert_eq!(pair.challenge, expected);
    }

    #[test]
    fn verifier_meets_rfc7636_length_bounds() {
        let pair = generate();
        assert!((43..=128).contains(&pair.verifier.len()));
    }

    #[test]
    fn successive_calls_differ() {
        assert_ne!(generate().verifier, generate().verifier);
    }
}
