use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use url::Url;

use crate::{AuthorizationResponse, OAuthError};

const DEFAULT_SUCCESS_HTML: &str = r#"<!doctype html>
<html>
  <head><meta charset="utf-8" /><title>Authorization complete</title></head>
  <body>
    <p>Authorization complete. You may close this window.</p>
  </body>
</html>
"#;

const DEFAULT_ERROR_HTML: &str = r#"<!doctype html>
<html>
  <head><meta charset="utf-8" /><title>Authorization error</title></head>
  <body>
    <p>Authorization failed. You may close this window and try again.</p>
  </body>
</html>
"#;

#[derive(Debug, Clone)]
pub struct LocalServer {
    target: RedirectTarget,
    success_html: String,
    error_html: String,
}

impl LocalServer {
    pub fn new(redirect_uri: impl Into<String>) -> Result<Self, OAuthError> {
        let redirect_uri = redirect_uri.into();
        Ok(Self {
            target: RedirectTarget::parse(&redirect_uri)?,
            success_html: DEFAULT_SUCCESS_HTML.to_string(),
            error_html: DEFAULT_ERROR_HTML.to_string(),
        })
    }

    pub fn with_success_html(mut self, html: impl Into<String>) -> Self {
        self.success_html = html.into();
        self
    }

    pub fn with_error_html(mut self, html: impl Into<String>) -> Self {
        self.error_html = html.into();
        self
    }

    pub fn bind(&self) -> Result<TcpListener, OAuthError> {
        TcpListener::bind((self.target.host.as_str(), self.target.port)).map_err(OAuthError::from)
    }

    pub fn listen_with(&self, listener: TcpListener) -> Result<AuthorizationResponse, OAuthError> {
        loop {
            let (mut stream, _) = listener.accept()?;
            match self.handle_request(&mut stream)? {
                Some(response) => return Ok(response),
                None => continue,
            }
        }
    }

    pub fn listen_once(&self) -> Result<AuthorizationResponse, OAuthError> {
        let listener = self.bind()?;
        self.listen_with(listener)
    }

    fn handle_request(
        &self,
        stream: &mut TcpStream,
    ) -> Result<Option<AuthorizationResponse>, OAuthError> {
        let mut buffer = [0u8; 8192];
        let bytes = stream.read(&mut buffer)?;
        if bytes == 0 {
            return Ok(None);
        }

        let request = String::from_utf8_lossy(&buffer[..bytes]);
        let request_line = match request.lines().next() {
            Some(line) => line,
            None => {
                write_response(stream, "400 Bad Request", &self.error_html)?;
                return Ok(None);
            }
        };

        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let target = parts.next().unwrap_or("");

        if method != "GET" {
            write_response(stream, "405 Method Not Allowed", &self.error_html)?;
            return Ok(None);
        }

        let (path, query) = target.split_once('?').unwrap_or((target, ""));
        if path != self.target.path {
            write_response(stream, "404 Not Found", &self.error_html)?;
            return Ok(None);
        }

        let callback_url = self.target.build_callback_url(query)?;
        match AuthorizationResponse::from_url(&callback_url) {
            Ok(response) => {
                write_response(stream, "200 OK", &self.success_html)?;
                Ok(Some(response))
            }
            Err(OAuthError::MissingAuthorizationCode) => {
                write_response(stream, "400 Bad Request", &self.error_html)?;
                Ok(None)
            }
            Err(error) => {
                write_response(stream, "500 Internal Server Error", &self.error_html)?;
                Err(error)
            }
        }
    }
}

#[derive(Debug, Clone)]
struct RedirectTarget {
    scheme: String,
    host: String,
    port: u16,
    path: String,
}

impl RedirectTarget {
    fn parse(redirect_uri: &str) -> Result<Self, OAuthError> {
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

    fn build_callback_url(&self, query: &str) -> Result<String, OAuthError> {
        let base = format!("{}://{}:{}{}", self.scheme, self.host, self.port, self.path);

        if query.is_empty() {
            return Ok(base);
        }

        let url = Url::parse(&format!("{base}?{query}"))?;
        Ok(url.to_string())
    }
}

fn write_response(stream: &mut TcpStream, status: &str, body: &str) -> Result<(), OAuthError> {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
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
