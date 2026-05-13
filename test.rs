use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use std::io;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    port: u16,
}

fn main() {
    let mut cmd = Cli::command();
    
    // We must call build() to instantiate help and version flags
    cmd.build();

    let arg_ids: Vec<_> = cmd.get_arguments().map(|a| a.get_id().clone()).collect();
    for id in arg_ids {
        cmd = cmd.mut_arg(id, |a| a.short(None));
    }

    generate(Shell::Bash, &mut cmd, "testcmd".to_string(), &mut io::stdout());
}
