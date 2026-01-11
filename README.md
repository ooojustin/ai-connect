# ai-connect

A Rust library for authenticating with your AI provider subscriptions using OAuth 2.0 + PKCE.

## Supported Providers

- Anthropic (Claude)
- OpenAI

## Installation

```toml
[dependencies]
ai-connect = { version = "0.1", features = ["local-server"] }
```

## Usage

```rust
use ai_connect::{AnthropicProvider, OAuthError};

#[tokio::main]
async fn main() -> Result<(), OAuthError> {
    let auth = AnthropicProvider::authorize(|req| {
        println!("Authorization URL: {}", req.url);
        Ok(())
    })?;

    let response = auth.wait().await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}
```
