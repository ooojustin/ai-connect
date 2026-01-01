//! Connect to AI provider accounts via OAuth 2.0 + PKCE.
//!
//! This crate simplifies authentication with AI providers using secure public-client
//! OAuth flows. No client secrets required—just PKCE for security. Supports Anthropic,
//! OpenAI, and can be extended to other providers via the `OAuthProvider` trait.

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
