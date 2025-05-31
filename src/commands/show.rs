use eyre::{Result, eyre};
use std::{fs, io::Write, path::PathBuf};
use tracing::info;

use crate::{
    gpg::decrypt_data,
    store::{ensure_store_directory_exists, get_password_file_path, get_password_store_path},
    utils::determine_key,
};

pub fn handle_show(path: &str, show_all: bool, key_path: Option<String>) -> Result<()> {
    let home_dir_str = std::env::var("HOME")?;
    let home_dir = PathBuf::from(home_dir_str);
    let (cert, _) = determine_key(&home_dir, key_path)?;

    let store_path = get_password_store_path()?;

    ensure_store_directory_exists(&store_path)?;

    let password_file_path = get_password_file_path(&store_path, path)?;
    if !password_file_path.is_file() {
        if password_file_path.is_dir() {
            return Err(eyre!(
                "Error: '{}' is a directory. Use 'list {}' to list contents.",
                path,
                path
            ));
        }
        return Err(eyre!("Error: Password entry '{}' not found.", path));
    }

    let encrypted_data = fs::read(&password_file_path)?;
    let decrypted_data_bytes = decrypt_data(&cert, &encrypted_data)?;
    let decrypted_content = String::from_utf8(decrypted_data_bytes)?;

    if show_all {
        info!("\n{}", decrypted_content);
    } else {
        let first_line = decrypted_content.lines().next().unwrap_or("").trim_end();
        info!("{}", first_line);
    }
    std::io::stdout().flush()?;

    Ok(())
}
