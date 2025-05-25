use eyre::{Result, eyre};
use rpassword::prompt_password;
use std::fs;
use std::path::PathBuf;
use tracing::info;

use crate::commands::generate::{DEFAULT_PASSWORD_LENGTH, generate_password_internal};
use crate::gpg::encrypt_data;
use crate::store::{
    ensure_store_directory_exists, get_password_file_path, get_password_store_path,
};
use crate::utils::determine_key;

fn get_password_content(
    path_name: &str,
    generate_flag: bool,
    length_opt: Option<usize>,
    no_symbols_opt: bool,
) -> Result<String> {
    let mut lines: Vec<String> = Vec::new();

    if generate_flag {
        let length = length_opt.unwrap_or(DEFAULT_PASSWORD_LENGTH);
        let no_symbols = no_symbols_opt;
        let password = generate_password_internal(length, no_symbols);
        lines.push(password);
    } else {
        let password = prompt_password(format!("Enter password for {}: ", path_name))?;
        if password.is_empty() {
            info!("Empty password, generating one.");
            let gen_length = DEFAULT_PASSWORD_LENGTH;
            let gen_no_symbols = false;
            lines.push(generate_password_internal(gen_length, gen_no_symbols));
        } else {
            lines.push(password);
        }
    }

    if lines.is_empty() || lines[0].is_empty() {
        return Err(eyre!("Password content cannot be empty."));
    }

    Ok(lines.join("\n"))
}

pub fn handle_add(
    path: &str,
    force: bool,
    generate: bool,
    length: Option<usize>,
    no_symbols: bool,
    key_path: Option<String>,
) -> Result<()> {
    let home_dir_str = std::env::var("HOME")?;
    let home_dir = PathBuf::from(home_dir_str);
    let (cert, _) = determine_key(&home_dir, key_path)?;

    let store_path = get_password_store_path()?;

    ensure_store_directory_exists(&store_path)?;

    let password_file_path = get_password_file_path(&store_path, path)?;

    if password_file_path.exists() && !force {
        return Err(eyre!(
            "Password entry '{}' already exists. Use --force to overwrite.",
            path
        ));
    }

    if let Some(parent_dir) = password_file_path.parent() {
        fs::create_dir_all(parent_dir)?;
    }

    let content = get_password_content(path, generate, length, no_symbols)?;

    let encrypted_data = encrypt_data(content.as_bytes(), &cert)?;

    fs::write(&password_file_path, encrypted_data)?;

    println!("Password for '{}' added.", path);
    Ok(())
}
