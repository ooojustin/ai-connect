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

    let response = auth.wait().await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn run_openai() -> Result<(), OAuthError> {
    let auth = OpenAIProvider::authorize(|req| {
        eprintln!("Authorization URL:\n{}", req.url);
        webbrowser::open(&req.url).ok();
        Ok(())
    })?;

    let response = auth.wait().await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}
