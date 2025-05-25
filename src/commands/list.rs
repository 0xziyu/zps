use std::{collections::BTreeMap, path::PathBuf};

use eyre::{Result, eyre};
use termtree::Tree;
use tracing::info;
use walkdir::WalkDir;

use crate::store::{ensure_store_directory_exists, get_password_store_path};

// Helper to recursively build the tree structure for termtree
// `entries` should be a collection of paths relative to the listing_base_path, with .gpg removed.
fn build_display_tree(base_display_name: String, relative_entry_paths: &[PathBuf]) -> Tree<String> {
    let mut root = Tree::new(base_display_name);
    let mut children_map = BTreeMap::<String, Vec<PathBuf>>::new();

    for rel_path in relative_entry_paths {
        let mut components = rel_path.components();
        if let Some(first_comp) = components.next() {
            let first_comp_str = first_comp.as_os_str().to_string_lossy().into_owned();
            let remaining_path: PathBuf = components.collect();
            children_map
                .entry(first_comp_str)
                .or_default()
                .push(remaining_path);
        }
    }

    for (name, sub_paths) in children_map {
        if sub_paths.iter().all(|p| p.as_os_str().is_empty()) {
            root.push(Tree::new(name));
        } else {
            let dir_sub_paths: Vec<PathBuf> = sub_paths
                .into_iter()
                .filter(|p| !p.as_os_str().is_empty())
                .collect();
            root.push(build_display_tree(name, &dir_sub_paths));
        }
    }
    root
}

pub fn handle_list(subfolder: Option<&str>) -> Result<()> {
    let store_path = get_password_store_path()?;

    ensure_store_directory_exists(&store_path)?;

    let listing_base_path = match subfolder {
        Some(sf) if !sf.is_empty() => store_path.join(sf),
        _ => store_path.clone(),
    };

    if !listing_base_path.is_dir() {
        let display_path = subfolder.unwrap_or("Password Store root");
        return Err(eyre!(
            "Error: '{}' is not a directory or does not exist.",
            display_path
        ));
    }

    let mut relative_entry_paths: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(&listing_base_path)
        .min_depth(1)
        .sort_by_file_name()
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "gpg") {
            if let Ok(rel_path) = path.strip_prefix(&listing_base_path) {
                relative_entry_paths.push(rel_path.with_extension(""));
            }
        }
    }

    if relative_entry_paths.is_empty() {
        let display_base = if listing_base_path == store_path {
            "Password Store".to_string()
        } else {
            listing_base_path
                .strip_prefix(&store_path)
                .unwrap_or(&listing_base_path)
                .to_string_lossy()
                .into_owned()
        };
        info!("{} is empty.", display_base);
        return Ok(());
    }

    let tree_root_name = if listing_base_path == store_path {
        "Password Store".to_string()
    } else {
        listing_base_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    };

    let tree = build_display_tree(tree_root_name, &relative_entry_paths);
    info!("{}", tree);

    Ok(())
}
