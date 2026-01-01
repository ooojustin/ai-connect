use ai_connect::{
    AnthropicProvider, OAuthClient, OAuthClientConfig, OAuthError, OAuthProvider, OpenAIProvider,
};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "ai-connect",
    about = "Connect to AI provider accounts via OAuth and print access tokens as JSON."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Anthropic,
    Openai,
}

#[tokio::main]
async fn main() -> Result<(), OAuthError> {
    let cli = Cli::parse();
    match cli.command {
        Command::Anthropic => run_anthropic().await,
        Command::Openai => run_openai().await,
    }
}

async fn run_anthropic() -> Result<(), OAuthError> {
    let provider = AnthropicProvider;
    let config = OAuthClientConfig::new(
        AnthropicProvider::default_client_id(),
        AnthropicProvider::default_redirect_uri(),
    )
    .with_scope(provider.default_scope());

    let client = OAuthClient::new(provider, config)?;

    let tokens = client
        .run_local_flow(|auth| {
            eprintln!("Authorization URL:\n{}", auth.authorization_url);
            if let Err(err) = webbrowser::open(&auth.authorization_url) {
                eprintln!("Failed to open browser automatically: {err}");
            }
            Ok(())
        })
        .await?;

    let output =
        serde_json::to_string_pretty(&tokens).map_err(|err| OAuthError::InvalidResponse {
            message: err.to_string(),
            body: String::new(),
        })?;

    println!("{output}");
    Ok(())
}

async fn run_openai() -> Result<(), OAuthError> {
    let provider = OpenAIProvider::new();
    let config = OAuthClientConfig::new(
        OpenAIProvider::default_client_id(),
        OpenAIProvider::default_redirect_uri(),
    )
    .with_scope(provider.default_scope());

    let client = OAuthClient::new(provider, config)?;

    let tokens = client
        .run_local_flow(|auth| {
            eprintln!("Authorization URL:\n{}", auth.authorization_url);
            if let Err(err) = webbrowser::open(&auth.authorization_url) {
                eprintln!("Failed to open browser automatically: {err}");
            }
            Ok(())
        })
        .await?;

    let output =
        serde_json::to_string_pretty(&tokens).map_err(|err| OAuthError::InvalidResponse {
            message: err.to_string(),
            body: String::new(),
        })?;

    println!("{output}");
    Ok(())
}
