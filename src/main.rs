use clap::Parser;
use colored::*;
use doit_cli::*;
use std::env;
/// A command line tool to run commands defined in a do.yaml file
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<Vec<String>>,

    #[clap(long, short, action)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    env::set_var("DOIT_PROD", "true");
    let conf = get_dofiles(None).expect("No do.yaml found");
    if cli.verbose {
        println!("Verbose mode enabled");
        std::env::set_var("RUST_LOG", "debug");
    }

    if let Some(name) = cli.name {
        let print_name = name.join(" ");
        println!("------------------------------------");
        println!("Name: {}", print_name.green());
        conf.exec(name.iter().map(|s| &**s).collect())?;
    } else {
        conf.list_commands();
    }

    // search recursively for do.yaml files in parent directories
    Ok(())
}
