# ai-connect

A Rust library for authenticating with AI provider subscriptions using OAuth 2.0 + PKCE.

## Supported Providers

- Anthropic (Claude Code)
- OpenAI (Codex)

## Installation

```toml
[dependencies]
ai-connect = { version = "0.1", features = ["local-server"] }
```

## Usage

Code:

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

Output:

```json
{
  "access_token": "[REDACTED]",
  "refresh_token": "[REDACTED]",
  "token_type": "Bearer",
  "scope": "user:inference user:profile",
  "expires_in": 28800,
  "organization": {
    "name": "My Organization",
    "uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
  },
  "account": {
    "email_address": "my@email.net",
    "uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
  }
}
```
