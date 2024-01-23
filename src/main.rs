use clap::{Parser, Subcommand};
use colored::*;
use std::env;
mod init;
use init::*;
mod exec;
use exec::*;
mod list;
use list::List;
mod config;
use config::Conf;
/// A command line tool to run commands defined in a do.yaml file
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<Vec<String>>,

    #[clap(long, short, action)]
    verbose: bool,

    #[clap(subcommand)]
    command: Option<Command>
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Help message for init.
    Init,
    /// Help message for list.
    List,
    // ...other commands (can #[clap(flatten)] other enum variants here)
    Run,
}

fn run_cli() -> anyhow::Result<()> {
    let cli = Cli::parse();
    env::set_var("DOIT_PROD", "true");

    let conf = get_dofiles(None)?;

    if cli.verbose {
        println!("Verbose mode enabled");
        std::env::set_var("RUST_LOG", "debug");
    }

    fn execute(name: Vec<String>, conf: &Conf) -> anyhow::Result<()> {
        let print_name = name.join(" ");
        println!("------------------------------------");
        println!("Name: {}", print_name.green());
        conf.exec(name.iter().map(|s| &**s).collect())?;
        Ok(())
    }

    match cli.command {
        Some(Command::Init) => init(),
        Some(Command::Run) => {
            let name = cli.name.unwrap();
            execute(name, &conf)?;
        }
        // Command::List => conf.list_commands(),
        Some(Command::List) => conf.list_commands(),
        None => {
            if let Some(name) = cli.name {
                execute(name, &conf)?;
            } else {
                conf.list_commands();
            }
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = run_cli() {
        println!("{}", e)
    }
}
