use clap::Parser;
use cli::{Cli, Commands};
use commands::{
    add::handle_add, generate::handle_generate, init::handle_init_command, list::handle_list,
    remove::handle_remove, show::handle_show,
};
use eyre::Result;

mod cli;
mod commands;
mod gpg;
mod store;
mod utils;

fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init {} => {
            handle_init_command(cli.key_path)?;
        }
        Commands::Add {
            path,
            force,
            generate,
            length,
            no_symbols,
        } => {
            handle_add(&path, force, generate, length, no_symbols, cli.key_path)?;
        }
        Commands::Show { path } => {
            handle_show(&path, cli.key_path)?;
        }
        Commands::List { subfolder } => {
            handle_list(subfolder.as_deref())?;
        }
        Commands::Remove { path, recursive } => {
            handle_remove(&path, recursive)?;
        }
        Commands::Generate { length, no_symbols } => {
            handle_generate(length, no_symbols)?;
        }
    }

    Ok(())
}
