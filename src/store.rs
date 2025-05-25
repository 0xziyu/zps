use eyre::Result;
use eyre::eyre;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

pub const PASSWORD_STORE_DIR_NAME: &str = ".zps";
pub const CONFIG_DIR_NAME: &str = ".config";
pub const DEFAULT_KEY_FILE_NAME: &str = "key.pgp";

/// Returns the path to the password store directory.
/// If not set, it defaults to `$HOME/.zps`.
pub fn get_password_store_path() -> Result<PathBuf> {
    // Determine home directory based on OS
    let home_dir_str = env::var("HOME");

    match home_dir_str {
        Ok(home) => Ok(PathBuf::from(home).join(PASSWORD_STORE_DIR_NAME)),
        Err(e) => Err(eyre!(
            "Failed to determine home directory (HOME or USERPROFILE not set): {}",
            e
        )),
    }
}

/// Ensures the main password store directory exists.
/// Creates it if it doesn't.
pub fn ensure_store_directory_exists(store_path: &Path) -> Result<()> {
    if !store_path.exists() {
        fs::create_dir_all(store_path)?;
        info!("Created password store directory: {:?}", store_path);
    }
    Ok(())
}

/// Returns the path to a password file within the store.
/// E.g., for "work/email", returns ~/.password-store/work/email.gpg
pub fn get_password_file_path(store_path: &Path, entry_name: &str) -> Result<PathBuf> {
    if entry_name.is_empty()
        || entry_name.contains("..")
        || entry_name.starts_with('/')
        || entry_name.starts_with('\\')
    {
        return Err(eyre!(
            "Invalid password entry name (cannot be empty, contain '..', or be an absolute path): '{}'",
            entry_name
        ));
    }
    // Normalize path separators for consistency, though join handles this.
    let mut path = store_path.to_path_buf();
    for component in entry_name.split(['/', '\\']) {
        if component == "." || component.is_empty() {
            // Allow empty components from "foo//bar" -> "foo/bar"
            continue;
        }
        path.push(component);
    }

    Ok(path.with_extension("gpg"))
}
