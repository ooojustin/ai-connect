use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{TryRngCore, rngs::OsRng};
use sha2::{Digest, Sha256};

use crate::OAuthError;

const VERIFIER_BYTES: usize = 32;

#[derive(Debug, Clone)]
pub struct PkcePair {
    pub code_verifier: String,
    pub code_challenge: String,
}

impl PkcePair {
    pub fn generate() -> Result<Self, OAuthError> {
        let mut bytes = [0u8; VERIFIER_BYTES];
        OsRng
            .try_fill_bytes(&mut bytes)
            .map_err(|err| OAuthError::OsRng {
                message: err.to_string(),
            })?;
        Ok(Self::from_verifier(URL_SAFE_NO_PAD.encode(bytes)))
    }

    pub fn from_verifier(code_verifier: impl Into<String>) -> Self {
        let code_verifier = code_verifier.into();
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let digest = hasher.finalize();
        let code_challenge = URL_SAFE_NO_PAD.encode(digest);
        Self {
            code_verifier,
            code_challenge,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PkcePair;

    #[test]
    fn generates_url_safe_pkce() {
        let pkce = PkcePair::generate().unwrap();
        for value in [&pkce.code_verifier, &pkce.code_challenge] {
            assert!(!value.contains('='), "pkce values should be unpadded");
            assert!(!value.contains('+'), "pkce values should be url safe");
            assert!(!value.contains('/'), "pkce values should be url safe");
        }
    }
}
