use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// Path to the shader file
    pub file: Option<PathBuf>,

    /// The amount of time (in milliseconds) under which consecutives file events are combined
    #[arg(long, default_value_t = 250)]
    pub debouncer_ms: u32,

    /// Print all debug logs to the terminal
    #[arg(long, default_value_t = false)]
    pub debug: bool,
}
// #[command(subcommand)]
// pub command: Option<Commands>,
