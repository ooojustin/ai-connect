//! Authenticate with Anthropic and print the token response.
//!
//! Run with:
//!   cargo run --features cli --example anthropic

use ai_connect::{AnthropicProvider, OAuthError};

#[tokio::main]
async fn main() -> Result<(), OAuthError> {
    let auth = AnthropicProvider::authorize(|req| {
        println!("Open this URL to authorize:\n{}\n", req.url);
        webbrowser::open(&req.url).ok();
        Ok(())
    })?;

    println!("Waiting for authorization...");
    let response = auth.wait().await?;

    println!("Access token: {}", response.access_token);
    if let Some(refresh) = &response.refresh_token {
        println!("Refresh token: {}", refresh);
    }

    Ok(())
}
