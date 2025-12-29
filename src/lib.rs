//! Provider-agnostic OAuth 2.0 + PKCE helpers.
//!
//! This crate focuses on public-client flows (no client secret required) and
//! provides a small provider trait so new services can be added later.

mod client;
mod error;
mod local_server;
mod pkce;
mod provider;
mod providers;
mod types;

pub use client::{OAuthClient, OAuthClientConfig};
pub use error::OAuthError;
pub use local_server::LocalServer;
pub use pkce::PkcePair;
pub use provider::{OAuthProvider, TokenRequestFormat};
pub use providers::AnthropicProvider;
pub use types::{AuthorizationRequest, AuthorizationResponse, TokenResponse};
