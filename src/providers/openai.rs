use crate::{OAuthProvider, TokenRequestFormat};

// References:
// - https://github.com/openai/codex/blob/810ebe0d2b23cdf29f65e6ca50ee46fa1c24a877/codex-rs/login/src/server.rs#L380-L418
// - https://github.com/openai/codex/blob/810ebe0d2b23cdf29f65e6ca50ee46fa1c24a877/codex-rs/core/src/auth.rs#L618
// - https://github.com/openai/codex/blob/810ebe0d2b23cdf29f65e6ca50ee46fa1c24a877/codex-rs/login/src/server.rs#L32

const AUTHORIZE_URL: &str = "https://auth.openai.com/oauth/authorize";
const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";

const DEFAULT_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const DEFAULT_REDIRECT_URI: &str = "http://localhost:1455/auth/callback";
const DEFAULT_SCOPE: &str = "openid profile email offline_access";
const DEFAULT_ORIGINATOR: &str = "codex_cli_rs";

const AUTHORIZE_PARAMS: &[(&str, &str)] = &[
    ("id_token_add_organizations", "true"),
    ("codex_cli_simplified_flow", "true"),
    ("originator", DEFAULT_ORIGINATOR),
];

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
        AUTHORIZE_URL
    }

    fn token_url(&self) -> &'static str {
        TOKEN_URL
    }

    fn default_scope(&self) -> &'static str {
        DEFAULT_SCOPE
    }

    fn authorize_params(&self) -> Vec<(String, String)> {
        let mut params: Vec<(String, String)> = AUTHORIZE_PARAMS
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();
        if self.originator != DEFAULT_ORIGINATOR {
            set_param(&mut params, "originator", self.originator.clone());
        }
        params
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
        DEFAULT_CLIENT_ID
    }

    pub fn default_redirect_uri() -> &'static str {
        DEFAULT_REDIRECT_URI
    }
}

fn set_param(params: &mut Vec<(String, String)>, key: &str, value: String) {
    if let Some((_, existing)) = params.iter_mut().find(|(param, _)| param == key) {
        *existing = value;
    } else {
        params.push((key.to_string(), value));
    }
}
