//! Resolve paths changed vs a git ref (`git diff <ref> --name-only --diff-filter=ACMR`).

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::walker::path_matches_scan;

fn run_git(workdir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(workdir)
        .args(args)
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let msg = if stderr.is_empty() { stdout } else { stderr };
        return Err(if msg.is_empty() {
            format!("git exited with status {}", output.status)
        } else {
            msg
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Git work tree root containing `start` (must be a directory).
pub fn git_repo_root(start: &Path) -> Result<PathBuf, String> {
    let start = if start.is_file() {
        start
            .parent()
            .ok_or_else(|| "path has no parent directory".to_string())?
    } else {
        start
    };
    let out = run_git(start, &["rev-parse", "--show-toplevel"])?
        .trim()
        .to_string();
    if out.is_empty() {
        return Err("git rev-parse --show-toplevel returned empty output".into());
    }
    Ok(PathBuf::from(out))
}

/// Paths relative to repo root (POSIX lines from git), added/copied/modified/renamed vs current tree.
pub fn git_changed_paths(repo_root: &Path, ref_name: &str) -> Result<Vec<String>, String> {
    let stdout = run_git(
        repo_root,
        &["diff", "--name-only", "--diff-filter=ACMR", ref_name, "--"],
    )?;
    Ok(stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect())
}

/// Files under `scan_root` that match config include rules and appear in the diff vs `ref_name`.
pub fn scan_paths_from_git_diff(
    scan_root: &Path,
    config: &crate::config::Config,
    ref_name: &str,
) -> Result<Vec<PathBuf>, String> {
    let repo_root = git_repo_root(scan_root)?;
    let rel_lines = git_changed_paths(&repo_root, ref_name)?;
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for rel in rel_lines {
        let candidate = repo_root.join(&rel);
        let candidate = match candidate.canonicalize() {
            Ok(p) => p,
            Err(_) => continue,
        };
        if !seen.insert(candidate.clone()) {
            continue;
        }
        if path_matches_scan(scan_root, &candidate, config) {
            out.push(candidate);
        }
    }
    out.sort();
    Ok(out)
}
