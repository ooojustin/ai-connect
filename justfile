# Show available recipes.
help:
    @just --list --unsorted

# Run Clippy with all features enabled.
clippy:
    cargo clippy-all

# Run the Anthropic OAuth flow via the CLI.
anthropic:
    cargo run --features=cli anthropic

# Run the OpenAI OAuth flow via the CLI.
openai:
    cargo run --features=cli openai
