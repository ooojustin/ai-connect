use thiserror::Error;

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("os rng error: {message}")]
    OsRng { message: String },

    #[error("url parse error: {0}")]
    Url(#[from] url::ParseError),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("invalid redirect uri: {0}")]
    InvalidRedirectUri(String),

    #[error("invalid header: {name}={value}")]
    InvalidHeader { name: String, value: String },

    #[error("http status {status}: {body}")]
    HttpStatus { status: u16, body: String },

    #[error("invalid response: {message}")]
    InvalidResponse { message: String, body: String },

    #[error("missing authorization code in callback url")]
    MissingAuthorizationCode,

    #[error("state mismatch (expected={expected}, received={received})")]
    StateMismatch { expected: String, received: String },

    #[cfg(feature = "local-server")]
    #[error("local server timed out after {timeout:?}")]
    LocalServerTimeout { timeout: std::time::Duration },
}
