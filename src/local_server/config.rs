use std::time::Duration;

use crate::OAuthError;

use super::target::RedirectTarget;

pub(crate) const DEFAULT_SUCCESS_HTML: &str = include_str!("html/success.html");
pub(crate) const DEFAULT_ERROR_HTML: &str = include_str!("html/error.html");

#[derive(Debug, Clone)]
pub struct LocalServerConfig {
    pub host: String,
    pub port: u16,
    pub path: String,
    pub timeout: Option<Duration>,
    pub success_html: String,
    pub error_html: String,
}

impl LocalServerConfig {
    pub fn new(host: impl Into<String>, port: u16, path: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port,
            path: normalize_path(path.into()),
            timeout: None,
            success_html: DEFAULT_SUCCESS_HTML.to_string(),
            error_html: DEFAULT_ERROR_HTML.to_string(),
        }
    }

    pub fn from_redirect_uri(redirect_uri: &str) -> Result<Self, OAuthError> {
        let target = RedirectTarget::parse(redirect_uri)?;
        Ok(Self {
            host: target.host,
            port: target.port,
            path: target.path,
            timeout: None,
            success_html: DEFAULT_SUCCESS_HTML.to_string(),
            error_html: DEFAULT_ERROR_HTML.to_string(),
        })
    }

    pub fn redirect_uri(&self) -> String {
        format!("http://{}:{}{}", self.host, self.port, self.path)
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_success_html(mut self, html: impl Into<String>) -> Self {
        self.success_html = html.into();
        self
    }

    pub fn with_error_html(mut self, html: impl Into<String>) -> Self {
        self.error_html = html.into();
        self
    }
}

fn normalize_path(path: String) -> String {
    if path.is_empty() {
        "/".to_string()
    } else if path.starts_with('/') {
        path
    } else {
        format!("/{}", path)
    }
}

#[cfg(test)]
mod tests {
    use super::LocalServerConfig;

    #[test]
    fn local_server_config_normalizes_path() {
        let config = LocalServerConfig::new("localhost", 8765, "callback");
        assert_eq!(config.path, "/callback");
        assert_eq!(config.redirect_uri(), "http://localhost:8765/callback");
    }
}
