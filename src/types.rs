use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::OAuthError;

#[derive(Debug, Clone)]
pub struct AuthorizationRequest {
    pub authorization_url: String,
    pub pkce: crate::PkcePair,
    pub state: String,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub struct AuthorizationResponse {
    pub code: String,
    pub state: Option<String>,
}

impl AuthorizationResponse {
    pub fn from_callback(code: &str, state: Option<&str>) -> Self {
        if state.is_none() {
            if let Some((code_part, state_part)) = code.split_once('#') {
                return Self {
                    code: code_part.to_string(),
                    state: Some(state_part.to_string()),
                };
            }
        }

        Self {
            code: code.to_string(),
            state: state.map(str::to_string),
        }
    }

    pub fn from_url(callback_url: &str) -> Result<Self, OAuthError> {
        let url = Url::parse(callback_url)?;
        let mut code = None;
        let mut state = None;

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "code" => code = Some(value.to_string()),
                "state" => state = Some(value.to_string()),
                _ => {}
            }
        }

        let code = code.ok_or(OAuthError::MissingAuthorizationCode)?;
        Ok(Self::from_callback(&code, state.as_deref()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
    pub expires_in: Option<u64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::AuthorizationResponse;
    use crate::OAuthError;

    #[test]
    fn from_callback_splits_state_from_code() {
        let response = AuthorizationResponse::from_callback("abc123#state456", None);
        assert_eq!(response.code, "abc123");
        assert_eq!(response.state.as_deref(), Some("state456"));
    }

    #[test]
    fn from_url_parses_query_params() {
        let response =
            AuthorizationResponse::from_url("http://localhost/callback?code=abc123&state=state456")
                .unwrap();
        assert_eq!(response.code, "abc123");
        assert_eq!(response.state.as_deref(), Some("state456"));
    }

    #[test]
    fn from_url_requires_code() {
        let result = AuthorizationResponse::from_url("http://localhost/callback?state=state456");
        assert!(matches!(result, Err(OAuthError::MissingAuthorizationCode)));
    }
}
