//! Provider-agnostic OAuth 2.0 + PKCE helpers.
//!
//! This crate focuses on public-client flows (no client secret required) and
//! provides a small provider trait so new services can be added later.

mod client;
mod error;
#[cfg(feature = "local-server")]
mod local_server;
mod pkce;
mod providers;
mod types;

pub use client::{OAuthClient, OAuthClientConfig};
pub use error::OAuthError;
#[cfg(feature = "local-server")]
pub use local_server::{LocalServer, LocalServerConfig};
pub use pkce::PkcePair;
pub use providers::{AnthropicProvider, OAuthProvider, OpenAIProvider, TokenRequestFormat};
pub use types::{AuthorizationRequest, AuthorizationResponse, TokenResponse};
