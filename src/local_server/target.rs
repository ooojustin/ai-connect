use url::Url;

use crate::OAuthError;

#[derive(Debug, Clone)]
pub(super) struct RedirectTarget {
    pub(super) scheme: String,
    pub(super) host: String,
    pub(super) port: u16,
    pub(super) path: String,
}

impl RedirectTarget {
    pub(super) fn parse(redirect_uri: &str) -> Result<Self, OAuthError> {
        let url = Url::parse(redirect_uri)?;
        if url.scheme() != "http" {
            return Err(OAuthError::InvalidRedirectUri(
                "redirect uri must use http scheme".to_string(),
            ));
        }

        let host = url.host_str().ok_or_else(|| {
            OAuthError::InvalidRedirectUri("redirect uri is missing host".to_string())
        })?;

        let port = url.port_or_known_default().ok_or_else(|| {
            OAuthError::InvalidRedirectUri("redirect uri is missing port".to_string())
        })?;

        Ok(Self {
            scheme: url.scheme().to_string(),
            host: host.to_string(),
            port,
            path: url.path().to_string(),
        })
    }

    pub(super) fn build_callback_url(&self, query: &str) -> Result<String, OAuthError> {
        let base = format!("{}://{}:{}{}", self.scheme, self.host, self.port, self.path);

        if query.is_empty() {
            return Ok(base);
        }

        let url = Url::parse(&format!("{base}?{query}"))?;
        Ok(url.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::RedirectTarget;

    #[test]
    fn parses_redirect_target() {
        let target = RedirectTarget::parse("http://localhost:8765/callback").unwrap();
        assert_eq!(target.host, "localhost");
        assert_eq!(target.port, 8765);
        assert_eq!(target.path, "/callback");
    }
}
