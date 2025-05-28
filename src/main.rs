use clap::Parser;
use cli::{Cli, Commands, OtpCommands, PassCommands};
use commands::{
    init::handle_init_command,
    list::handle_list,
    otp::{add::handle_otp_add, generate::handle_otp_generate},
    pass::{add::handle_pass_add, generate::handle_pass_generate},
    remove::handle_remove,
    show::handle_show,
};
use eyre::Result;

mod cli;
mod commands;
mod constants;
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
        Commands::Pass { command } => match command {
            PassCommands::Add {
                path,
                force,
                generate,
                length,
                no_symbols,
            } => {
                handle_pass_add(&path, force, generate, length, no_symbols, cli.key_path)?;
            }
            PassCommands::Generate { length, no_symbols } => {
                handle_pass_generate(length, no_symbols)?;
            }
        },
        Commands::Show { path } => {
            handle_show(&path, cli.key_path)?;
        }
        Commands::List { subfolder } => {
            handle_list(subfolder.as_deref())?;
        }
        Commands::Remove { path, recursive } => {
            handle_remove(&path, recursive)?;
        }
        Commands::Otp { command } => match command {
            OtpCommands::Add { path, uri } => {
                handle_otp_add(&path, &uri, cli.key_path)?;
            }
            OtpCommands::Generate { path } => {
                handle_otp_generate(&path, cli.key_path)?;
            }
        },
    }

    Ok(())
}
