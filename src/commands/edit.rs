use eyre::{Result, eyre};
use std::{fs, path::PathBuf};
use tracing::info;

use crate::{
    gpg::{decrypt_data, encrypt_data},
    store::{ensure_store_directory_exists, get_password_file_path, get_password_store_path},
    utils::{determine_key, edit_content_in_editor},
    vcs::jj_commit_changes,
};

pub fn handle_edit(path: &str, key_path: Option<String>) -> Result<()> {
    let home_dir_str = std::env::var("HOME")?;
    let home_dir = PathBuf::from(home_dir_str);
    let (cert, _) = determine_key(&home_dir, key_path)?;

    let store_path = get_password_store_path()?;
    ensure_store_directory_exists(&store_path)?;

    let password_file_path = get_password_file_path(&store_path, path)?;
    let file_existed_before_edit = password_file_path.exists();

    let initial_content = if file_existed_before_edit {
        info!("Editing existing entry: {}", path);
        let encrypted_data = fs::read(&password_file_path)?;
        let decrypted_bytes = decrypt_data(&cert, &encrypted_data)?;
        String::from_utf8(decrypted_bytes).map_err(|e| {
            eyre!(
                "Failed to decode decrypted content for '{}' as UTF-8: {}",
                path,
                e
            )
        })?
    } else {
        info!("Creating new entry via edit: {}", path);
        "\n".to_string()
    };

    let modified_content = edit_content_in_editor(&initial_content)?;

    if modified_content.trim().is_empty() {
        if file_existed_before_edit {
            info!("Content is empty after editing. Removing entry '{}'.", path);
            fs::remove_file(&password_file_path)?;
            let commit_message = format!("Remove entry {} (edited to empty)", path);
            jj_commit_changes(&store_path, &commit_message)?;
            info!("Entry '{}' removed as it was saved empty.", path);
        } else {
            info!("New entry '{}' was saved empty. No file created.", path);
        }
        return Ok(());
    }

    if let Some(parent_dir) = password_file_path.parent() {
        fs::create_dir_all(parent_dir)?;
    }

    let encrypted_data = encrypt_data(modified_content.as_bytes(), &cert)?;
    fs::write(&password_file_path, encrypted_data)?;

    let (action_message, commit_action_prefix) = if !file_existed_before_edit {
        (format!("Added entry for '{}'", path), "Add")
    } else {
        (format!("Updated entry for '{}'", path), "Edit")
    };
    info!("{}", action_message);

    let commit_message = format!("{} entry {}", commit_action_prefix, path);
    jj_commit_changes(&store_path, &commit_message)?;

    Ok(())
}
