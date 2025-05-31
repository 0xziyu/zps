use std::path::PathBuf;

use eyre::Result;
use tracing::info;

use crate::{
    store::{ensure_store_directory_exists, get_password_store_path},
    utils::determine_key,
    vcs::jj_init_repository,
};

pub fn handle_init_command(key_path: Option<String>) -> Result<()> {
    let home_dir_str = std::env::var("HOME")?;
    let home_dir = PathBuf::from(home_dir_str);

    let store_path = get_password_store_path()?;

    ensure_store_directory_exists(&store_path)?;

    jj_init_repository(&store_path)?;

    let (_, used_key_file_path) = determine_key(&home_dir, key_path)?;

    info!("Password store initialized successfully");
    info!("The PGP key file being used is: {:?}", used_key_file_path);

    Ok(())
}
