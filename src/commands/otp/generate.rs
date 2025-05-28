use std::path::PathBuf;

use eyre::{Result, eyre};
use totp_rs::TOTP;
use tracing::info;

use crate::{
    gpg::decrypt_data,
    store::{ensure_store_directory_exists, get_password_file_path, get_password_store_path},
    utils::determine_key,
};

/// Generate OTP code from entry
pub fn handle_otp_generate(path: &str, key_path: Option<String>) -> Result<()> {
    let home_dir_str = std::env::var("HOME")?;
    let home_dir = PathBuf::from(home_dir_str);
    let (cert, _) = determine_key(&home_dir, key_path)?;

    let store_path = get_password_store_path()?;

    ensure_store_directory_exists(&store_path)?;
    let otp_file_path = get_password_file_path(&store_path, path)?;
    if !otp_file_path.is_file() {
        if otp_file_path.is_dir() {
            return Err(eyre!(
                "Error: '{}' is a directory. Use 'list {}' to list contents.",
                path,
                path
            ));
        }
        return Err(eyre!("Error: OTP entry '{}' not found.", path));
    }

    let encrypted_data = std::fs::read(&otp_file_path)?;
    let decrypted_data_bytes = decrypt_data(&cert, &encrypted_data)?;
    let decrypted_content = String::from_utf8(decrypted_data_bytes)?;

    let first_line = decrypted_content.lines().next().unwrap_or("").trim_end();

    let totp = TOTP::from_url_unchecked(first_line)
        .map_err(|e| eyre!("Invalid OTP configuration: {}", e))?;

    let code = totp.generate_current()?;
    info!("{}", code);
    Ok(())
}
