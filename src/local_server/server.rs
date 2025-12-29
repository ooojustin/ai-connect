use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use axum::{Router, routing::get};
use tokio::net::TcpListener as TokioTcpListener;
use tokio::runtime::Builder;
use tokio::sync::oneshot;

use crate::{AuthorizationResponse, OAuthError};

use super::config::{DEFAULT_ERROR_HTML, DEFAULT_SUCCESS_HTML, LocalServerConfig};
use super::http::{
    LocalServerState, callback_handler, fallback_handler, send_response, wait_for_response,
};
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
        let server = self.clone();
        let handle = thread::spawn(move || -> Result<AuthorizationResponse, OAuthError> {
            let runtime = Builder::new_current_thread().enable_all().build()?;
            runtime.block_on(server.listen_with_async(listener))
        });

        match handle.join() {
            Ok(result) => result,
            Err(_) => Err(OAuthError::InvalidResponse {
                message: "local server thread panicked".to_string(),
                body: String::new(),
            }),
        }
    }

    pub fn listen_once(&self) -> Result<AuthorizationResponse, OAuthError> {
        let listener = self.bind()?;
        self.listen_with(listener)
    }

    pub async fn listen_with_async(
        &self,
        listener: TcpListener,
    ) -> Result<AuthorizationResponse, OAuthError> {
        let (response_tx, response_rx) =
            oneshot::channel::<Result<AuthorizationResponse, OAuthError>>();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let response_tx = Arc::new(Mutex::new(Some(response_tx)));

        let state = LocalServerState {
            target: self.target.clone(),
            success_html: self.success_html.clone(),
            error_html: self.error_html.clone(),
            response_tx: response_tx.clone(),
        };

        let app = Router::new()
            .route(&state.target.path, get(callback_handler))
            .fallback(fallback_handler)
            .with_state(state);

        listener.set_nonblocking(true)?;
        let listener = TokioTcpListener::from_std(listener)?;

        let server = axum::serve(listener, app).with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        });

        let response_tx_for_server = response_tx.clone();
        let server_handle = tokio::spawn(async move {
            if let Err(err) = server.await {
                let error = OAuthError::InvalidResponse {
                    message: err.to_string(),
                    body: String::new(),
                };
                send_response(&response_tx_for_server, Err(error));
            }
        });

        let response = wait_for_response(response_rx, self.timeout).await;

        let _ = shutdown_tx.send(());
        let _ = server_handle.await;

        response
    }

    pub async fn listen_once_async(&self) -> Result<AuthorizationResponse, OAuthError> {
        let listener = self.bind()?;
        self.listen_with_async(listener).await
    }
}
