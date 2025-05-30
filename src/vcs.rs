use std::{
    path::Path,
    process::{Command, Stdio},
};

use eyre::{Result, eyre};
use tracing::{error, info};

fn run_jj_command(store_path: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("jj")
        .current_dir(store_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        if !stdout_str.is_empty() {
            info!("jj stdout: {}", stdout_str.trim());
        }
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        if !stderr_str.is_empty() {
            info!("jj stderr: {}", stderr_str.trim());
        }
        Ok(())
    } else {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        error!("jj command failed with status: {}", output.status);
        error!("jj stderr: {}", stderr_str.trim());
        Err(eyre!(
            "jj command `jj {}` failed in {:?}: {}",
            args.join(" "),
            store_path,
            stderr_str.trim()
        ))
    }
}

/// Initializes a Jujutsu repository in the given path and creates an initial commit.
/// Skips if a .jj or .git directory already exists.
pub fn jj_init_repository(store_path: &Path) -> Result<()> {
    if store_path.join(".jj").exists() || store_path.join(".git").exists() {
        info!(
            "Jujutsu (.jj) or Git (.git) repository already exists in {:?}. Skipping initialization.",
            store_path
        );
        return Ok(());
    }

    run_jj_command(store_path, &["git", "init"])?;
    info!(
        "Jujutsu repository initialized with Git backend in {:?}",
        store_path
    );

    // Create an initial commit for the new repository
    run_jj_command(
        store_path,
        &["commit", "-m", "Initial commit: Initialize password store"],
    )?;
    Ok(())
}

/// Commits changes in the Jujutsu repository with the given message.
/// This assumes that file system changes have already been made.
pub fn jj_commit_changes(store_path: &Path, message: &str) -> Result<()> {
    if !store_path.join(".jj").exists() {
        return Err(eyre!(
            "Not a Jujutsu repository ('.jj' directory not found) in {:?}. Cannot commit.",
            store_path
        ));
    }

    run_jj_command(store_path, &["commit", "-m", message])
}
