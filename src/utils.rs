use std::{
    io::Write,
    path::{Path, PathBuf},
};

use eyre::{Result, eyre};
use sequoia_openpgp::{Cert, serialize::MarshalInto};
use tracing::info;

use crate::{
    gpg::{generate_key_with_password, load_and_validate_key_from_file},
    store::{CONFIG_DIR_NAME, DEFAULT_KEY_FILE_NAME},
};

/// Helper for getting trimmed user input from stdin.
fn get_trimmed_user_input(prompt_message: &str) -> Result<String> {
    info!("{}", prompt_message);
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Handles the case where a key path is provided by the user.
fn handle_user_provided_key(key_path_str: &str) -> Result<(Cert, PathBuf)> {
    let user_provided_key_path = PathBuf::from(key_path_str);
    if !user_provided_key_path.exists() {
        return Err(eyre!(
            "Specified key file not found: {:?}",
            user_provided_key_path
        ));
    }
    info!("Using specified key file: {:?}", user_provided_key_path);
    let cert = load_and_validate_key_from_file(&user_provided_key_path)?;
    Ok((cert, user_provided_key_path))
}

/// Generates a new PGP key, saves it to the default path, and then loads/validates it.
fn generate_and_save_new_key(default_key_path: &Path, default_key_app_dir: &Path) -> Result<Cert> {
    info!(
        "No PGP key found at default location: {:?}, generating new key.",
        default_key_path
    );

    if !default_key_app_dir.exists() {
        std::fs::create_dir_all(default_key_app_dir)?;
        info!(
            "Created directory for PGP key storage: {:?}",
            default_key_app_dir
        );
    }

    let name = get_trimmed_user_input("Enter your full name (e.g., John Doe): ")?;
    if name.is_empty() {
        return Err(eyre!("Name cannot be empty. Key generation aborted."));
    }

    let email = get_trimmed_user_input("Enter your email address (e.g., john.doe@example.com): ")?;
    if email.is_empty() {
        return Err(eyre!("Email cannot be empty. Key generation aborted."));
    }

    let user_id_str = format!("{} <{}>", name, email);
    info!("The User ID for the new key will be: {}", user_id_str);

    let password = rpassword::prompt_password("Enter a strong password for your new PGP key: ")?;
    let password_confirm = rpassword::prompt_password("Confirm PGP key password: ")?;
    if password != password_confirm {
        return Err(eyre!("Passwords do not match. Key generation aborted."));
    }

    info!("Generating new PGP key for '{}'...", user_id_str);
    let new_secret_cert = generate_key_with_password(user_id_str, &password)?;
    info!("PGP key generation successful.");

    let armored_key_bytes = new_secret_cert
        .as_tsk()
        .armored()
        .to_vec()
        .map_err(|e| eyre!("Failed to serialize PGP key to armored format: {}", e))?;
    std::fs::write(default_key_path, armored_key_bytes)?;
    info!("New PGP secret key saved to: {:?}", default_key_path);

    info!("Validating newly generated key...");
    load_and_validate_key_from_file(default_key_path)
}

/// Determines which PGP key to use (user-provided, default, or new), and loads its identity.
/// Returns the Cert, GpgIdentity, and the PathBuf of the key file used.
pub fn determine_key(home_dir: &Path, key_path_opt: Option<String>) -> Result<(Cert, PathBuf)> {
    let default_key_app_dir = home_dir.join(CONFIG_DIR_NAME);
    let default_key_path = default_key_app_dir.join(DEFAULT_KEY_FILE_NAME);

    if let Some(kp_str) = key_path_opt {
        handle_user_provided_key(&kp_str)
    } else if default_key_path.exists() {
        info!(
            "Found existing PGP key at default location: {:?}",
            default_key_path
        );
        let cert = load_and_validate_key_from_file(&default_key_path)?;
        Ok((cert, default_key_path))
    } else {
        let cert = generate_and_save_new_key(&default_key_path, &default_key_app_dir)?;
        Ok((cert, default_key_path))
    }
}
