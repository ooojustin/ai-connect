//! Authenticate with OpenAI and print the token response.
//!
//! Run with:
//!   cargo run --features cli --example openai

use ai_connect::{OAuthError, OpenAIProvider};

#[tokio::main]
async fn main() -> Result<(), OAuthError> {
    let auth = OpenAIProvider::authorize(|req| {
        println!("Open this URL to authorize:\n{}\n", req.url);
        webbrowser::open(&req.url).ok();
        Ok(())
    })?;

    println!("Waiting for authorization...");
    let tokens = auth.wait().await?;

    println!("Access token: {}", tokens.access_token);
    if let Some(refresh) = &tokens.refresh_token {
        println!("Refresh token: {}", refresh);
    }

    Ok(())
}
