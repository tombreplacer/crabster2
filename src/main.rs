mod cli;
mod file_info;
mod handlers;
mod server;

use clap::Parser;
use cli::Cli;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    // Generate shell completions and exit
    if let Some(shell) = args.completions {
        cli::print_completions(shell);
        return Ok(());
    }

    // Validate directory
    let dir = args.dir.canonicalize().unwrap_or_else(|_| {
        eprintln!(
            "\x1b[31mError:\x1b[0m Directory '{}' does not exist",
            args.dir.display()
        );
        std::process::exit(1);
    });

    if !dir.is_dir() {
        eprintln!(
            "\x1b[31mError:\x1b[0m '{}' is not a directory",
            dir.display()
        );
        std::process::exit(1);
    }

    server::start_server(
        &args.bind,
        args.port,
        dir,
        args.readonly,
        args.hidden,
        args.no_delete,
        args.auth,
    )
    .await
}
