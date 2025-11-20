///! TYPF CLI - Professional text rendering from the command line
///!
///! A unified command-line interface for text shaping and rendering
///! with support for multiple backends and output formats.

mod cli;
mod commands;

// Keep legacy modules for REPL and advanced batch
mod batch;
mod jsonl;
mod repl;

use clap::Parser;
use cli::{Cli, Commands};
use typf::error::Result;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Parse command-line arguments
    let cli = Cli::parse();

    // Dispatch to appropriate command handler
    match cli.command {
        Commands::Info(args) => commands::info::run(&args),
        Commands::Render(args) => commands::render::run(&args),
        Commands::Batch(args) => commands::batch::run(&args),
    }
}
