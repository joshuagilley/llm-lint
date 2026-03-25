use std::path::{Path, PathBuf};

use crate::config::Config;

pub fn path_matches_scan(root: &Path, path: &Path, config: &Config) -> bool {
    let root = match root.canonicalize() {
        Ok(p) => p,
        Err(_) => root.to_path_buf(),
    };
    let path = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };

    if !path.is_file() {
        return false;
    }
    let rel = match path.strip_prefix(&root) {
        Ok(r) => r,
        Err(_) => return false,
    };

    for part in rel.components() {
        let name = part.as_os_str().to_string_lossy();
        if config.exclude_dirs.iter().any(|ex| ex == name.as_ref()) {
            return false;
        }
    }

    let rel_posix = rel.to_string_lossy().replace('\\', "/");
    let exclude_file: std::collections::HashSet<_> = config.exclude_files.iter().cloned().collect();
    if exclude_file.contains(path.file_name().and_then(|n| n.to_str()).unwrap_or(""))
        || exclude_file.contains(&rel_posix)
    {
        return false;
    }

    let suffix = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| format!(".{}", s.to_lowercase()))
        .unwrap_or_default();
    config
        .include_extensions
        .iter()
        .any(|ext| ext.eq_ignore_ascii_case(&suffix))
}

pub fn walk_repo(root: &Path, config: &Config) -> Vec<PathBuf> {
    let root = match root.canonicalize() {
        Ok(p) => p,
        Err(_) => root.to_path_buf(),
    };

    let mut files = Vec::new();
    walk_recursive(&root, &root, config, &mut files);
    files.sort();
    files
}

fn walk_recursive(root: &Path, dir: &Path, config: &Config, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if config.exclude_dirs.iter().any(|ex| ex == name) {
                continue;
            }
            walk_recursive(root, &path, config, out);
        } else if path_matches_scan(root, &path, config) {
            out.push(path);
        }
    }
}
