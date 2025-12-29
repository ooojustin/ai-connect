use crate::OAuthProvider;

#[derive(Debug, Clone, Copy, Default)]
pub struct AnthropicProvider;

impl OAuthProvider for AnthropicProvider {
    fn id(&self) -> &'static str {
        "anthropic"
    }

    fn authorize_url(&self) -> &'static str {
        "https://claude.ai/oauth/authorize"
    }

    fn token_url(&self) -> &'static str {
        "https://console.anthropic.com/v1/oauth/token"
    }

    fn default_scope(&self) -> &'static str {
        "org:create_api_key user:profile user:inference"
    }

    fn authorize_params(&self) -> Vec<(String, String)> {
        vec![("code".to_string(), "true".to_string())]
    }
}

impl AnthropicProvider {
    pub fn default_client_id() -> &'static str {
        "9d1c250a-e61b-44d9-88ed-5944d1962f5e"
    }

    pub fn default_redirect_uri() -> &'static str {
        "http://localhost:8765/callback"
    }
}
