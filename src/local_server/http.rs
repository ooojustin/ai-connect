use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::{
    extract::{RawQuery, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use tokio::sync::oneshot;

use crate::{AuthorizationResponse, OAuthError};

use super::target::RedirectTarget;

type ResponseResult = Result<AuthorizationResponse, OAuthError>;
type ResponseSender = oneshot::Sender<ResponseResult>;
type ResponseReceiver = oneshot::Receiver<ResponseResult>;
type SharedResponseSender = Arc<Mutex<Option<ResponseSender>>>;

#[derive(Clone)]
pub(super) struct LocalServerState {
    pub(super) target: RedirectTarget,
    pub(super) success_html: String,
    pub(super) error_html: String,
    pub(super) response_tx: SharedResponseSender,
}

pub(super) fn send_response(response_tx: &SharedResponseSender, response: ResponseResult) {
    if let Ok(mut guard) = response_tx.lock() {
        if let Some(sender) = guard.take() {
            let _ = sender.send(response);
        }
    }
}

pub(super) async fn callback_handler(
    State(state): State<LocalServerState>,
    RawQuery(query): RawQuery,
) -> impl IntoResponse {
    let LocalServerState {
        target,
        success_html,
        error_html,
        response_tx,
    } = state;

    let query = query.unwrap_or_default();
    let callback_url = match target.build_callback_url(&query) {
        Ok(url) => url,
        Err(error) => {
            send_response(&response_tx, Err(error));
            return (StatusCode::INTERNAL_SERVER_ERROR, Html(error_html));
        }
    };

    match AuthorizationResponse::from_url(&callback_url) {
        Ok(response) => {
            send_response(&response_tx, Ok(response));
            (StatusCode::OK, Html(success_html))
        }
        Err(OAuthError::MissingAuthorizationCode) => (StatusCode::BAD_REQUEST, Html(error_html)),
        Err(error) => {
            send_response(&response_tx, Err(error));
            (StatusCode::INTERNAL_SERVER_ERROR, Html(error_html))
        }
    }
}

pub(super) async fn fallback_handler(State(state): State<LocalServerState>) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Html(state.error_html))
}

pub(super) async fn wait_for_response(
    response_rx: ResponseReceiver,
    timeout: Option<Duration>,
) -> Result<AuthorizationResponse, OAuthError> {
    if let Some(timeout) = timeout {
        let result = tokio::time::timeout(timeout, response_rx)
            .await
            .map_err(|_| OAuthError::LocalServerTimeout { timeout })?;
        result.map_err(|_| OAuthError::InvalidResponse {
            message: "local server response channel closed".to_string(),
            body: String::new(),
        })?
    } else {
        response_rx.await.map_err(|_| OAuthError::InvalidResponse {
            message: "local server response channel closed".to_string(),
            body: String::new(),
        })?
    }
}
