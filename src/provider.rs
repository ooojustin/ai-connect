#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenRequestFormat {
    Json,
    Form,
}

pub trait OAuthProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn authorize_url(&self) -> &'static str;
    fn token_url(&self) -> &'static str;
    fn default_scope(&self) -> &'static str;

    fn authorize_params(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn token_params(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn token_request_format(&self) -> TokenRequestFormat {
        TokenRequestFormat::Json
    }

    fn token_headers(&self) -> Vec<(String, String)> {
        Vec::new()
    }
}
