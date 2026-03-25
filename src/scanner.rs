use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::models::{FileContext, ScanResult};
use crate::parsers::{parse_python, parse_text};
use crate::rules::{
    registered_rule_ids, DuplicateFunctionsRule, ExposedSecretsRule, FallbackDefaultsRule,
    HelperSprawlRule, LargeFileRule, LargeFunctionRule,
};
use crate::scoring::compute_score;
use crate::walker::walk_repo;

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("unknown rule id(s) in config include list: {0}")]
    UnknownRules(String),
    #[error("git: {0}")]
    Git(String),
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

fn detect_language(path: &Path) -> String {
    if path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("py"))
    {
        "python".into()
    } else {
        "text".into()
    }
}

fn build_file_context(path: PathBuf, lines: Vec<String>) -> FileContext {
    let language = detect_language(&path);
    let functions = match language.as_str() {
        "python" => parse_python(&path, &lines),
        _ => parse_text(&path, &lines),
    };
    FileContext {
        path,
        lines,
        language,
        functions,
    }
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
    Ok(Some(build_file_context(path.to_path_buf(), lines)))
}

/// Walk `root`, apply enabled rules, return aggregated result.
///
/// When `changed_since` is `Some(ref)`, only files that appear in
/// `git diff ref --name-only --diff-filter=ACMR` under `root` are scanned (same idea as slopsniff `--branch` / `--changed-since`).
pub fn scan(
    root: &Path,
    config: &Config,
    changed_since: Option<&str>,
) -> Result<ScanResult, ScanError> {
    let enabled = resolve_enabled_rule_ids(config)?;
    let files = if let Some(ref_name) = changed_since {
        crate::git_scope::scan_paths_from_git_diff(root, config, ref_name)
            .map_err(ScanError::Git)?
    } else {
        walk_repo(root, config)
    };

    let mut contexts = Vec::new();
    for path in files {
        if let Some(ctx) = read_file_context(&path)? {
            contexts.push(ctx);
        }
    }

    let mut findings = Vec::new();
    let large_file = LargeFileRule::new(config);
    let large_fn = LargeFunctionRule::new(config);

    for ctx in &contexts {
        if enabled.contains(LargeFileRule::RULE_ID) {
            findings.extend(large_file.run(ctx));
        }
        if enabled.contains(LargeFunctionRule::RULE_ID) {
            findings.extend(large_fn.run(ctx));
        }
        if enabled.contains(HelperSprawlRule::RULE_ID) {
            findings.extend(HelperSprawlRule::run(ctx));
        }
        if enabled.contains(ExposedSecretsRule::RULE_ID) {
            findings.extend(ExposedSecretsRule::run(ctx));
        }
        if enabled.contains(FallbackDefaultsRule::RULE_ID) {
            findings.extend(FallbackDefaultsRule::run(ctx));
        }
    }

    if enabled.contains(DuplicateFunctionsRule::RULE_ID) {
        findings.extend(DuplicateFunctionsRule::run_cross_file(&contexts));
    }
    if enabled.contains(HelperSprawlRule::RULE_ID) {
        findings.extend(HelperSprawlRule::run_cross_file(&contexts));
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
