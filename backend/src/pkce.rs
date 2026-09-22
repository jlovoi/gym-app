use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct PkcePair {
    pub verifier: String,
    pub challenge: String,
}

fn random_bytes32() -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[..16].copy_from_slice(Uuid::new_v4().as_bytes());
    bytes[16..].copy_from_slice(Uuid::new_v4().as_bytes());
    bytes
}

/// Generates an RFC 7636 PKCE verifier/challenge pair (S256).
pub fn generate() -> PkcePair {
    let verifier = URL_SAFE_NO_PAD.encode(random_bytes32());
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    PkcePair { verifier, challenge }
}

/// A random URL-safe token, used as a nonce inside the signed OAuth "state" JWT.
pub fn random_token() -> String {
    URL_SAFE_NO_PAD.encode(random_bytes32())
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
        // RFC 7636 requires the verifier to be 43-128 characters.
        let pair = generate();
        assert!(pair.verifier.len() >= 43 && pair.verifier.len() <= 128);
    }

    #[test]
    fn successive_calls_are_not_reused() {
        let a = generate();
        let b = generate();
        assert_ne!(a.verifier, b.verifier);
        assert_ne!(random_token(), random_token());
    }
}
