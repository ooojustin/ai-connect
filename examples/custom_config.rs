//! Example showing how to customize the local OAuth server and client.
//!
//! Demonstrates:
//! - Custom success HTML page
//! - Disabling the default timeout
//! - Custom OAuth scope (more restrictive than default)
//!
//! Run with:
//!   cargo run --features cli --example custom_cfg

use ai_connect::{
    AnthropicProvider, LocalServerConfig, OAuthClient, OAuthClientConfig, OAuthError,
};

#[tokio::main]
async fn main() -> Result<(), OAuthError> {
    // Create custom local server config
    let redirect_uri = AnthropicProvider::default_redirect_uri();
    let server_config = LocalServerConfig::from_redirect_uri(redirect_uri)?
        // Customize: remove timeout
        .without_timeout()
        // Customize: change success page HTML
        .with_success_html("<html><h1>!! it worked !!</h1></html>");

    // Create custom client config
    let config = OAuthClientConfig::new(
        AnthropicProvider::default_client_id(),
        server_config.redirect_uri(),
    )
    // Customize: apply local server config
    .with_local_server_config(server_config)
    // Customize: apply custom scope
    .with_scope("user:inference");

    // Create client, perform authorization
    let client = OAuthClient::new(AnthropicProvider, config)?;
    let auth = client.authorize(|req| {
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
