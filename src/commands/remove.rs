use eyre::{Result, eyre};
use tracing::info;

use crate::store::{
    ensure_store_directory_exists, get_password_file_path, get_password_store_path,
};

pub fn handle_remove(path: &str, recursive: bool) -> Result<()> {
    let store_path = get_password_store_path()?;
    ensure_store_directory_exists(&store_path)?;

    let potential_gpg_file_path = get_password_file_path(&store_path, path)?;
    let potential_dir_path = store_path.join(path);

    let (path_to_remove_fs, is_dir_removal) = if potential_dir_path.is_dir() {
        if !recursive {
            return Err(eyre!(
                "Error: '{}' is a directory. Use --recursive (-r) to remove.",
                path
            ));
        }
        (potential_dir_path, true)
    } else if potential_gpg_file_path.is_file() {
        (potential_gpg_file_path, false)
    } else {
        return Err(eyre!(
            "Error: '{}' not found as a password or directory.",
            path
        ));
    };

    if is_dir_removal {
        std::fs::remove_dir_all(&path_to_remove_fs)?;
        info!("Directory '{}' removed.", path);
    } else {
        std::fs::remove_file(&path_to_remove_fs)?;
        println!("Password '{}' removed.", path);
    }

    Ok(())
}
