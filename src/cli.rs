use clap::{Parser, Subcommand};

use crate::commands::generate::DEFAULT_PASSWORD_LENGTH;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    /// Optional: Path to your PGP secret key file (e.g., ~/.config/your-app/key.pgp).
    /// If not provided, attempts to use a default key (in ~/.config/pass-rs/default_key.pgp)
    /// or prompts to create one if the default does not exist.
    #[arg(long)]
    pub key_path: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new password store
    Init {},
    /// Add a new password to the store
    Add {
        /// Path to the password entry (e.g., "work/email" or "social/twitter.com")
        #[clap(value_name = "PATH")]
        path: String,

        /// Overwrite existing password if it exists.
        #[clap(long, short)]
        force: bool,

        /// Generate a new password instead of prompting.
        #[clap(long, short = 'g')] // Added short flag -g
        generate: bool,

        /// Optional length for the generated password if --generate is used.
        /// Defaults to the same default as the 'generate' command.
        #[clap(long, requires = "generate")]
        length: Option<usize>,

        /// Optional: do not use symbols if --generate is used with --length.
        #[clap(long, requires = "generate")] // Technically, only requires `generate`
        no_symbols: bool,
    },
    /// Show (decrypt and print) an existing password
    Show {
        /// Path to the password entry
        #[clap(value_name = "PATH")]
        path: String,
    },
    /// List passwords
    List {
        /// Optional subfolder to list
        #[clap(value_name = "SUBFOLDER")]
        subfolder: Option<String>,
    },
    /// Remove a password or folder
    Rm {
        /// Path to the password entry or folder
        #[clap(value_name = "PATH")]
        path: String,

        /// Remove recursively (required for folders).
        #[clap(long, short, alias = "force")]
        recursive: bool,
    },
    /// Generate a new password
    Generate {
        /// Length of the generated password.
        #[clap(long, short = 'l', default_value_t = DEFAULT_PASSWORD_LENGTH)]
        length: usize,

        /// Exclude symbols from the generated password.
        #[clap(long, short = 'n', alias = "no-symbols")]
        no_symbols: bool,
    },
}
