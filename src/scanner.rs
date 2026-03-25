use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::models::{FileContext, ScanResult};
use crate::rules::{registered_rule_ids, LargeFileRule};
use crate::scoring::compute_score;
use crate::walker::walk_repo;

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("unknown rule id(s) in config include list: {0}")]
    UnknownRules(String),
    #[error("failed to read {path}: {source}")]
    ReadFile {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

fn resolve_enabled_rule_ids(config: &Config) -> Result<HashSet<String>, ScanError> {
    let available: HashSet<String> = registered_rule_ids()
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    let chosen: HashSet<String> = match &config.include_rules {
        None => available.clone(),
        Some(v) if v.is_empty() => available.clone(),
        Some(v) => v.iter().cloned().collect(),
    };

    let unknown: Vec<String> = chosen.difference(&available).cloned().collect();
    if !unknown.is_empty() {
        let mut u = unknown;
        u.sort();
        return Err(ScanError::UnknownRules(u.join(", ")));
    }

    Ok(chosen)
}

fn read_file_context(path: &Path) -> Result<Option<FileContext>, ScanError> {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            return Err(ScanError::ReadFile {
                path: path.display().to_string(),
                source: e,
            })
        }
    };
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    Ok(Some(FileContext {
        path: path.to_path_buf(),
        lines,
    }))
}

/// Walk `root`, apply enabled rules, return aggregated result.
pub fn scan(root: &Path, config: &Config) -> Result<ScanResult, ScanError> {
    let enabled = resolve_enabled_rule_ids(config)?;
    let files = walk_repo(root, config);

    let mut contexts = Vec::new();
    for path in files {
        if let Some(ctx) = read_file_context(&path)? {
            contexts.push(ctx);
        }
    }

    let mut findings = Vec::new();
    let large_file = LargeFileRule::new(config);

    for ctx in &contexts {
        if enabled.contains(LargeFileRule::RULE_ID) {
            findings.extend(large_file.run(ctx));
        }
    }

    if !config.exclude_severities.is_empty() {
        findings.retain(|f| {
            !config
                .exclude_severities
                .contains(&f.severity.to_lowercase())
        });
    }

    let total_score = compute_score(&findings);
    let passed = total_score <= config.fail_threshold;

    Ok(ScanResult {
        findings,
        total_score,
        files_scanned: contexts.len(),
        passed,
    })
}
