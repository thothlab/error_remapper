mod config;
mod input;
mod matcher;
mod output;
mod settings;

use clap::Parser;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

/// CLI utility for fuzzy matching and remapping backend error codes
/// using a YAML dictionary.
#[derive(Parser, Debug)]
#[command(name = "error-remapper", version, about)]
struct Cli {
    /// JSON string with the error (reads from stdin if omitted)
    input_json: Option<String>,

    /// Path to settings.toml
    #[arg(short, long, default_value = "config/settings.toml")]
    config: PathBuf,

    /// Path to errors.yaml (overrides settings.toml value)
    #[arg(short, long)]
    errors: Option<PathBuf>,

    /// Pretty-print output JSON
    #[arg(short, long)]
    pretty: bool,

    /// Verbose output (enable debug logging)
    #[arg(short, long)]
    verbose: bool,
}

fn run(cli: Cli) -> Result<(), String> {
    // Load settings
    let mut settings = settings::load_settings(&cli.config)?;

    // CLI flag overrides config
    if cli.pretty {
        settings.output.pretty = true;
    }

    // Determine errors YAML path
    let errors_path = cli
        .errors
        .unwrap_or_else(|| PathBuf::from(&settings.files.errors_yaml));

    // Load error dictionary
    let entries = config::load_error_config(&errors_path)?;

    // Get input JSON
    let json_str = match cli.input_json {
        Some(s) => s,
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("Failed to read stdin: {}", e))?;
            buf
        }
    };

    if json_str.trim().is_empty() {
        return Err("No input JSON provided".into());
    }

    // Parse input as raw JSON value (for passthrough fields)
    let input_value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse input JSON: {}", e))?;

    // Parse input error fields
    let parsed = input::parse_error_json(
        &json_str,
        &settings.input.code_fields,
        &settings.input.message_fields,
    )?;

    // Find match
    let result = matcher::find_match(&parsed, &entries, settings.matching.fuzzy_threshold);

    // Output result using template
    let json_output = output::format_result(&result, &input_value, &settings.output);
    println!("{}", json_output);

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "warn" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp(None)
        .init();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
