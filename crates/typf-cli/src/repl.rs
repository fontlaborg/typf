//! Interactive text rendering: explore TYPF in real-time
//!
//! Perfect for experimenting with fonts, testing text samples,
//! and understanding how the pipeline works step by step.

#![allow(dead_code)] // Legacy REPL infrastructure - retained for future v2.1 REPL mode

#[cfg(feature = "repl")]
use colored::Colorize;
#[cfg(feature = "repl")]
use rustyline::error::ReadlineError;
#[cfg(feature = "repl")]
use rustyline::DefaultEditor;

#[cfg(feature = "repl")]
pub fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "TYPF Interactive REPL v2.0".bold().cyan());
    println!("{}", "Type 'help' for commands, 'exit' to quit".dimmed());
    println!();

    let mut rl = DefaultEditor::new()?;
    let mut context = ReplContext::default();

    // The main interactive loop
    loop {
        let prompt = format!("{}> ", "typf".green().bold());
        let readline = rl.readline(&prompt);

        match readline {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                // Remember what we typed
                rl.add_history_entry(line)?;

                // Figure out what to do
                match process_command(line, &mut context) {
                    Ok(ControlFlow::Continue) => {},
                    Ok(ControlFlow::Exit) => {
                        println!("{}", "Goodbye!".yellow());
                        break;
                    },
                    Err(e) => {
                        eprintln!("{} {}", "Error:".red().bold(), e);
                    },
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("{}", "^C (use 'exit' or ^D to quit)".yellow());
            },
            Err(ReadlineError::Eof) => {
                println!("{}", "Goodbye!".yellow());
                break;
            },
            Err(err) => {
                eprintln!("{} {:?}", "Error:".red().bold(), err);
                break;
            },
        }
    }

    Ok(())
}

#[cfg(feature = "repl")]
#[derive(Default)]
struct ReplContext {
    font: Option<String>,
    size: f32,
    output_format: String,
}

#[cfg(feature = "repl")]
enum ControlFlow {
    Continue,
    Exit,
}

#[cfg(feature = "repl")]
fn process_command(
    line: &str,
    context: &mut ReplContext,
) -> Result<ControlFlow, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.is_empty() {
        return Ok(ControlFlow::Continue);
    }

    match parts[0] {
        "help" | "h" | "?" => {
            show_help();
            Ok(ControlFlow::Continue)
        },
        "exit" | "quit" | "q" => Ok(ControlFlow::Exit),
        "status" | "s" => {
            show_status(context);
            Ok(ControlFlow::Continue)
        },
        "set" => {
            if parts.len() < 3 {
                println!("{}", "Usage: set <key> <value>".yellow());
                println!("Example: set font NotoSans");
                return Ok(ControlFlow::Continue);
            }
            set_option(parts[1], parts[2], context)?;
            Ok(ControlFlow::Continue)
        },
        "render" | "r" => {
            if parts.len() < 2 {
                println!("{}", "Usage: render <text> [output]".yellow());
                return Ok(ControlFlow::Continue);
            }
            let text = parts[1..].join(" ");
            render_text(&text, context)?;
            Ok(ControlFlow::Continue)
        },
        "info" => {
            show_info();
            Ok(ControlFlow::Continue)
        },
        _ => {
            println!("{} {}", "Unknown command:".red(), parts[0]);
            println!("Type 'help' for available commands");
            Ok(ControlFlow::Continue)
        },
    }
}

#[cfg(feature = "repl")]
fn show_help() {
    println!("{}", "Available Commands:".bold().underline());
    println!();
    println!("  {}  - Show this help", "help, h, ?".cyan());
    println!("  {}     - Show current settings", "status, s".cyan());
    println!("  {}       - Show system information", "info".cyan());
    println!("  {} - Set configuration option", "set <key> <value>".cyan());
    println!("  {} - Render text", "render <text> [output]".cyan());
    println!("  {}  - Exit REPL", "exit, quit, q".cyan());
    println!();
    println!("{}", "Settings:".bold().underline());
    println!("  font       - Font family (e.g., 'NotoSans')");
    println!("  size       - Font size in pixels (e.g., '48')");
    println!("  format     - Output format (png, svg, json)");
    println!();
    println!("{}", "Examples:".bold().underline());
    println!("  set font NotoSans");
    println!("  set size 48");
    println!("  render Hello, World! output.png");
}

#[cfg(feature = "repl")]
fn show_status(context: &ReplContext) {
    println!("{}", "Current Settings:".bold().underline());
    println!("  Font:   {}", context.font.as_deref().unwrap_or("(default)").cyan());
    println!("  Size:   {}", format!("{}", context.size).cyan());
    println!("  Format: {}", context.output_format.cyan());
}

#[cfg(feature = "repl")]
fn show_info() {
    println!("{}", "TYPF System Information:".bold().underline());
    println!("  Version:    {}", env!("CARGO_PKG_VERSION").cyan());
    println!("  Backends:");
    println!("    Shaping:  {}", "none, harfbuzz (if compiled)".dimmed());
    println!("    Rendering: {}", "opixa (if compiled)".dimmed());
    println!("  Formats:   {}", "PNG, SVG, PNM, JSON".dimmed());
}

#[cfg(feature = "repl")]
fn set_option(
    key: &str,
    value: &str,
    context: &mut ReplContext,
) -> Result<(), Box<dyn std::error::Error>> {
    match key {
        "font" => {
            context.font = Some(value.to_string());
            println!("{} font to {}", "Set".green(), value.cyan());
        },
        "size" => {
            context.size = value.parse()?;
            println!("{} size to {}", "Set".green(), value.cyan());
        },
        "format" => {
            context.output_format = value.to_string();
            println!("{} format to {}", "Set".green(), value.cyan());
        },
        _ => {
            println!("{} {}", "Unknown setting:".red(), key);
            println!("Valid settings: font, size, format");
        },
    }
    Ok(())
}

#[cfg(feature = "repl")]
fn render_text(text: &str, _context: &ReplContext) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} '{}'", "Rendering:".green(), text.cyan());
    // TODO: Actually render using the context settings
    println!("{}", "(Rendering not yet implemented in REPL)".yellow());
    Ok(())
}

#[cfg(not(feature = "repl"))]
pub fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("REPL mode not compiled in. Rebuild with --features repl");
    std::process::exit(1);
}
