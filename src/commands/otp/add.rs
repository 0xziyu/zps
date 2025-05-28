use std::path::PathBuf;

use eyre::{Result, eyre};
use totp_rs::TOTP;
use tracing::info;

use crate::{
    gpg::encrypt_data,
    store::{ensure_store_directory_exists, get_password_file_path, get_password_store_path},
    utils::determine_key,
};

/// Validate and normalize otpauth URI
fn normalize_otpauth_uri(uri: &str) -> Result<String> {
    let uri = uri.trim();

    if !uri.starts_with("otpauth://") {
        return Err(eyre!("Invalid otpauth URI - must start with 'otpauth://'"));
    }

    let _ = TOTP::from_url_unchecked(uri).map_err(|e| eyre!("Invalid otpauth URI: {}", e))?;

    Ok(uri.to_string())
}

/// Create new OTP entry
pub fn handle_otp_add(path: &str, uri: &str, key_path: Option<String>) -> Result<()> {
    let home_dir_str = std::env::var("HOME")?;
    let home_dir = PathBuf::from(home_dir_str);
    let (cert, _) = determine_key(&home_dir, key_path)?;

    let store_path = get_password_store_path()?;

    ensure_store_directory_exists(&store_path)?;

    let otp_file_path = get_password_file_path(&store_path, path)?;

    let uri = normalize_otpauth_uri(uri)?;
    let content = uri.clone();

    let encrypted = encrypt_data(content.as_bytes(), &cert)?;
    if let Some(parent) = otp_file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&otp_file_path, encrypted)?;

    info!("OTP entry created at {}", path);

    Ok(())
}
