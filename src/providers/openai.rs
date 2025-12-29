use crate::{OAuthProvider, TokenRequestFormat};

pub const DEFAULT_ORIGINATOR: &str = "codex_cli_rs";

#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    originator: String,
}

impl Default for OpenAIProvider {
    fn default() -> Self {
        Self {
            originator: DEFAULT_ORIGINATOR.to_string(),
        }
    }
}

impl OAuthProvider for OpenAIProvider {
    fn id(&self) -> &'static str {
        "openai"
    }

    fn authorize_url(&self) -> &'static str {
        "https://auth.openai.com/oauth/authorize"
    }

    fn token_url(&self) -> &'static str {
        "https://auth.openai.com/oauth/token"
    }

    fn default_scope(&self) -> &'static str {
        "openid profile email offline_access"
    }

    fn authorize_params(&self) -> Vec<(String, String)> {
        vec![
            ("id_token_add_organizations".to_string(), "true".to_string()),
            ("codex_cli_simplified_flow".to_string(), "true".to_string()),
            ("originator".to_string(), self.originator.clone()),
        ]
    }

    fn token_request_format(&self) -> TokenRequestFormat {
        TokenRequestFormat::Form
    }

    fn token_headers(&self) -> Vec<(String, String)> {
        vec![("Accept".to_string(), "application/json".to_string())]
    }
}

impl OpenAIProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_originator(mut self, originator: impl Into<String>) -> Self {
        self.originator = originator.into();
        self
    }

    pub fn default_client_id() -> &'static str {
        "app_EMoamEEZ73f0CkXaXp7hrann"
    }

    pub fn default_redirect_uri() -> &'static str {
        "http://localhost:1455/auth/callback"
    }
}
