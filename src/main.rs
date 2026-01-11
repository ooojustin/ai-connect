use ai_connect::{AnthropicProvider, OAuthError, OpenAIProvider};
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
    let auth = AnthropicProvider::authorize(|req| {
        eprintln!("Authorization URL:\n{}", req.url);
        webbrowser::open(&req.url).ok();
        Ok(())
    })?;

    let tokens = auth.wait().await?;

    let output =
        serde_json::to_string_pretty(&tokens).map_err(|err| OAuthError::InvalidResponse {
            message: err.to_string(),
            body: String::new(),
        })?;

    println!("{output}");
    Ok(())
}

async fn run_openai() -> Result<(), OAuthError> {
    let auth = OpenAIProvider::authorize(|req| {
        eprintln!("Authorization URL:\n{}", req.url);
        webbrowser::open(&req.url).ok();
        Ok(())
    })?;

    let tokens = auth.wait().await?;

    let output =
        serde_json::to_string_pretty(&tokens).map_err(|err| OAuthError::InvalidResponse {
            message: err.to_string(),
            body: String::new(),
        })?;

    println!("{output}");
    Ok(())
}
