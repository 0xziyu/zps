use std::path::PathBuf;

use eyre::{Result, eyre};
use tracing::info;

use crate::{store::ensure_store_directory_exists, utils::determine_key};

pub fn handle_init_command(key_path_opt: Option<String>) -> Result<()> {
    let home_dir_str = std::env::var("HOME").map_err(|_| {
        eyre!("Could not find HOME environment variable. Please ensure one is set.")
    })?;
    let home_dir = PathBuf::from(home_dir_str);

    ensure_store_directory_exists(&home_dir)?;

    let (_, used_key_file_path) = determine_key(&home_dir, key_path_opt)?;

    info!("Password store initialized successfully");
    info!("The PGP key file being used is: {:?}", used_key_file_path);

    Ok(())
}
