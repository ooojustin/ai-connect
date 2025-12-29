use crate::OAuthProvider;

const AUTHORIZE_URL: &str = "https://claude.ai/oauth/authorize";
const TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";

const DEFAULT_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const DEFAULT_REDIRECT_URI: &str = "http://localhost:8765/callback";
const DEFAULT_SCOPE: &str = "org:create_api_key user:profile user:inference";

const AUTHORIZE_PARAMS: &[(&str, &str)] = &[("code", "true")];

#[derive(Debug, Clone, Copy, Default)]
pub struct AnthropicProvider;

impl OAuthProvider for AnthropicProvider {
    fn id(&self) -> &'static str {
        "anthropic"
    }

    fn authorize_url(&self) -> &'static str {
        AUTHORIZE_URL
    }

    fn token_url(&self) -> &'static str {
        TOKEN_URL
    }

    fn default_scope(&self) -> &'static str {
        DEFAULT_SCOPE
    }

    fn authorize_params(&self) -> Vec<(String, String)> {
        AUTHORIZE_PARAMS
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect()
    }

    fn include_state_in_token_request(&self) -> bool {
        true
    }
}

impl AnthropicProvider {
    pub fn default_client_id() -> &'static str {
        DEFAULT_CLIENT_ID
    }

    pub fn default_redirect_uri() -> &'static str {
        DEFAULT_REDIRECT_URI
    }
}
