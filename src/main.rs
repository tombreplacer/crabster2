mod cli;
mod daemon;
mod file_info;
mod handlers;
mod server;

use clap::Parser;
use cli::{Cli, Commands};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    // Generate shell completions and exit
    if let Some(shell) = args.completions {
        cli::print_completions(shell);
        return Ok(());
    }

    // Handle subcommands
    if let Some(cmd) = &args.command {
        match cmd {
            Commands::Ps => {
                let instances = daemon::list_instances();
                if instances.is_empty() {
                    println!("No running instances");
                } else {
                    println!("{:<10} {:<10} {:<15} {:<20} {:<15}", "ID", "PID", "PORT", "STARTED", "DIRECTORY");
                    println!("{}", "─".repeat(70));
                    for inst in instances {
                        println!(
                            "{:<10} {:<10} {:<15} {:<20} {:<15}",
                            inst.id,
                            inst.pid,
                            inst.port,
                            inst.started.format("%Y-%m-%d %H:%M").to_string(),
                            inst.dir.display()
                        );
                    }
                }
                return Ok(());
            }
            Commands::Stop { id } => {
                match daemon::stop_instance(id) {
                    Ok(_) => println!("Stopped instance {}", id),
                    Err(e) => eprintln!("\x1b[31mError:\x1b[0m {}", e),
                }
                return Ok(());
            }
        }
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

    // Handle daemonization
    if args.daemon {
        match daemon::daemonize(args.port, &args.bind, dir.clone()) {
            Ok(id) => {
                if id.is_empty() {
                    // We are in the child process
                } else {
                    // We are in the parent process
                    println!("🦀 Crabster started in background");
                    println!("   ID:   \x1b[33m{}\x1b[0m", id);
                    println!("   URL:  \x1b[32mhttp://{}:{}\x1b[0m", if args.bind == "0.0.0.0" { "localhost" } else { &args.bind }, args.port);
                    return Ok(());
                }
            }
            Err(e) => {
                eprintln!("\x1b[31mError:\x1b[0m Failed to daemonize: {}", e);
                std::process::exit(1);
            }
        }
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
