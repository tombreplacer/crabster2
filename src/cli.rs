use clap::{CommandFactory, Parser, ValueHint};
use clap_complete::{generate, Shell};
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "crabster",
    version,
    about = "🦀 Crabster — a beautiful file server with web UI",
    long_about = "Crabster serves files from a directory via HTTP with a stunning web interface.\nUpload, download, browse and manage files from any browser."
)]
pub struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value = "8080", env = "CRABSTER_PORT")]
    pub port: u16,

    /// Address to bind to
    #[arg(short, long, default_value = "0.0.0.0", env = "CRABSTER_BIND")]
    pub bind: String,

    /// Directory to serve
    #[arg(short, long, default_value = "./", value_hint = ValueHint::DirPath, env = "CRABSTER_DIR")]
    pub dir: PathBuf,

    /// Read-only mode (disable upload and delete)
    #[arg(short, long)]
    pub readonly: bool,

    /// Show hidden files
    #[arg(long)]
    pub hidden: bool,

    /// Disable file deletion (uploads still allowed)
    #[arg(long)]
    pub no_delete: bool,

    /// Generate shell completions
    #[arg(long, value_name = "SHELL")]
    pub completions: Option<Shell>,
}

pub fn print_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
}
