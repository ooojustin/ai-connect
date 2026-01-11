use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::{OAuthError, TokenResponse};

/// Handle to a running authorization flow.
///
/// Returned by [`OAuthClient::authorize`]. Supports cancellation, polling, and async wait.
pub struct AuthHandle {
    result_rx: oneshot::Receiver<Result<TokenResponse, OAuthError>>,
    cancel_token: CancellationToken,
}

impl AuthHandle {
    pub(crate) fn new(
        result_rx: oneshot::Receiver<Result<TokenResponse, OAuthError>>,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            result_rx,
            cancel_token,
        }
    }

    /// Cancel the authorization flow. Returns [`OAuthError::Cancelled`].
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// Wait for the authorization flow to complete.
    pub async fn wait(self) -> Result<TokenResponse, OAuthError> {
        self.result_rx
            .await
            .map_err(|e| OAuthError::Internal(format!("OAuth flow channel closed: {e}")))?
    }

    /// Non-blocking check if a result is ready.
    pub fn try_result(&mut self) -> Option<Result<TokenResponse, OAuthError>> {
        match self.result_rx.try_recv() {
            Ok(result) => Some(result),
            Err(oneshot::error::TryRecvError::Empty) => None,
            Err(oneshot::error::TryRecvError::Closed) => Some(Err(OAuthError::Internal(
                "OAuth flow channel closed".into(),
            ))),
        }
    }
}
