use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use crate::{AuthorizationResponse, OAuthError};

use super::config::{DEFAULT_ERROR_HTML, DEFAULT_SUCCESS_HTML, LocalServerConfig};
use super::target::RedirectTarget;

#[derive(Debug, Clone)]
pub struct LocalServer {
    target: RedirectTarget,
    success_html: String,
    error_html: String,
    timeout: Option<Duration>,
}

impl LocalServer {
    pub fn new(redirect_uri: impl Into<String>) -> Result<Self, OAuthError> {
        let redirect_uri = redirect_uri.into();
        Ok(Self {
            target: RedirectTarget::parse(&redirect_uri)?,
            success_html: DEFAULT_SUCCESS_HTML.to_string(),
            error_html: DEFAULT_ERROR_HTML.to_string(),
            timeout: None,
        })
    }

    pub fn from_config(config: LocalServerConfig) -> Result<Self, OAuthError> {
        let redirect_uri = config.redirect_uri();
        Ok(Self {
            target: RedirectTarget::parse(&redirect_uri)?,
            success_html: config.success_html,
            error_html: config.error_html,
            timeout: config.timeout,
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

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn bind(&self) -> Result<TcpListener, OAuthError> {
        TcpListener::bind((self.target.host.as_str(), self.target.port)).map_err(OAuthError::from)
    }

    pub fn listen_with(&self, listener: TcpListener) -> Result<AuthorizationResponse, OAuthError> {
        if let Some(timeout) = self.timeout {
            return self.listen_with_timeout(listener, timeout);
        }

        self.listen_blocking(listener)
    }

    fn listen_blocking(&self, listener: TcpListener) -> Result<AuthorizationResponse, OAuthError> {
        loop {
            let (mut stream, _) = listener.accept()?;
            match self.handle_request(&mut stream)? {
                Some(response) => return Ok(response),
                None => continue,
            }
        }
    }

    fn listen_with_timeout(
        &self,
        listener: TcpListener,
        timeout: Duration,
    ) -> Result<AuthorizationResponse, OAuthError> {
        listener.set_nonblocking(true)?;
        let deadline = Instant::now() + timeout;

        loop {
            if Instant::now() >= deadline {
                return Err(OAuthError::LocalServerTimeout { timeout });
            }

            match listener.accept() {
                Ok((mut stream, _)) => {
                    let now = Instant::now();
                    if now >= deadline {
                        return Err(OAuthError::LocalServerTimeout { timeout });
                    }
                    let remaining = deadline.duration_since(now);
                    stream.set_read_timeout(Some(remaining))?;
                    if let Some(response) = self.handle_request(&mut stream)? {
                        return Ok(response);
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(err) => return Err(err.into()),
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

fn write_response(stream: &mut TcpStream, status: &str, body: &str) -> Result<(), OAuthError> {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}
