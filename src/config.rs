use std::collections::HashSet;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

const CONFIG_TOML: &str = "llm-lint.toml";
const CONFIG_JSON: &str = "llm-lint.json";

#[derive(Debug, Clone)]
pub struct Config {
    pub max_file_lines_warning: i32,
    pub max_file_lines_high: i32,
    pub max_function_lines_warning: i32,
    pub max_function_lines_high: i32,
    pub fail_threshold: i32,
    pub include_extensions: Vec<String>,
    pub large_file_extensions: HashSet<String>,
    pub exclude_dirs: Vec<String>,
    pub exclude_files: Vec<String>,
    pub exclude_severities: HashSet<String>,
    pub verbose: bool,
    /// If `None`, all known rules run. If set, only these rule ids.
    pub include_rules: Option<Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_file_lines_warning: 400,
            max_file_lines_high: 800,
            max_function_lines_warning: 50,
            max_function_lines_high: 100,
            fail_threshold: 20,
            include_extensions: vec![
                ".py".into(),
                ".js".into(),
                ".ts".into(),
                ".tsx".into(),
                ".jsx".into(),
                ".vue".into(),
                ".html".into(),
            ],
            large_file_extensions: [".py", ".js", ".ts", ".tsx", ".jsx", ".vue"]
                .into_iter()
                .map(String::from)
                .collect(),
            exclude_dirs: vec![
                ".git".into(),
                "node_modules".into(),
                ".nuxt".into(),
                "dist".into(),
                "build".into(),
                ".venv".into(),
                "coverage".into(),
                "tests".into(),
                "__pycache__".into(),
                ".pytest_cache".into(),
                ".mypy_cache".into(),
                ".ruff_cache".into(),
            ],
            exclude_files: Vec::new(),
            exclude_severities: HashSet::new(),
            verbose: false,
            include_rules: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read {0}: {1}")]
    Io(String, #[source] std::io::Error),
    #[error("invalid JSON in {0}: {1}")]
    Json(String, #[source] serde_json::Error),
    #[error("invalid TOML in {0}: {1}")]
    Toml(String, #[source] toml::de::Error),
    #[error("{0}")]
    Invalid(String),
}

/// File overlay for `llm-lint.toml` or `llm-lint.json`. Kebab-case (JSON) and snake_case (TOML) keys are accepted.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfigFile {
    #[serde(default, rename = "include")]
    include: Option<Vec<String>>,
    #[serde(default, rename = "fail-threshold", alias = "fail_threshold")]
    fail_threshold: Option<i32>,
    #[serde(
        default,
        rename = "max-file-lines-warning",
        alias = "max_file_lines_warning"
    )]
    max_file_lines_warning: Option<i32>,
    #[serde(default, rename = "max-file-lines-high", alias = "max_file_lines_high")]
    max_file_lines_high: Option<i32>,
    #[serde(
        default,
        rename = "max-function-lines-warning",
        alias = "max_function_lines_warning"
    )]
    max_function_lines_warning: Option<i32>,
    #[serde(
        default,
        rename = "max-function-lines-high",
        alias = "max_function_lines_high"
    )]
    max_function_lines_high: Option<i32>,
    #[serde(default, rename = "include-extensions", alias = "include_extensions")]
    include_extensions: Option<Vec<String>>,
    #[serde(
        default,
        rename = "large-file-extensions",
        alias = "large_file_extensions"
    )]
    large_file_extensions: Option<Vec<String>>,
    #[serde(default, rename = "exclude-dirs", alias = "exclude_dirs")]
    exclude_dirs: Option<Vec<String>>,
    #[serde(default, rename = "exclude-files", alias = "exclude_files")]
    exclude_files: Option<Vec<String>>,
    #[serde(default, rename = "exclude-severities", alias = "exclude_severities")]
    exclude_severities: Option<Vec<String>>,
    #[serde(default)]
    verbose: Option<bool>,
}

fn normalize_str_list(values: Vec<String>, _key: &str) -> Result<Vec<String>, ConfigError> {
    Ok(values
        .into_iter()
        .map(|v| v.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

const VALID_SEVERITIES: &[&str] = &["low", "medium", "high"];

/// Load `llm-lint.toml` when present and non-empty; otherwise `llm-lint.json`.
pub(crate) fn load_config_file(scan_root: &Path) -> Result<Option<ConfigFile>, ConfigError> {
    let toml_path = scan_root.join(CONFIG_TOML);
    if toml_path.exists() {
        let text = std::fs::read_to_string(&toml_path)
            .map_err(|e| ConfigError::Io(toml_path.display().to_string(), e))?;
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            let data: ConfigFile = toml::from_str(trimmed)
                .map_err(|e| ConfigError::Toml(toml_path.display().to_string(), e))?;
            return Ok(Some(data));
        }
    }

    let json_path = scan_root.join(CONFIG_JSON);
    if !json_path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&json_path)
        .map_err(|e| ConfigError::Io(json_path.display().to_string(), e))?;
    let data: ConfigFile = serde_json::from_str(&text)
        .map_err(|e| ConfigError::Json(json_path.display().to_string(), e))?;
    Ok(Some(data))
}

pub(crate) fn merge_config(
    mut base: Config,
    file: Option<ConfigFile>,
    cli_fail_threshold: Option<i32>,
    cli_verbose: Option<bool>,
    cli_max_file_lines: Option<i32>,
    cli_max_function_lines: Option<i32>,
) -> Result<Config, ConfigError> {
    if let Some(f) = file {
        if let Some(v) = f.fail_threshold {
            base.fail_threshold = v;
        }
        if let Some(v) = f.max_file_lines_warning {
            base.max_file_lines_warning = v;
        }
        if let Some(v) = f.max_file_lines_high {
            base.max_file_lines_high = v;
        }
        if let Some(v) = f.max_function_lines_warning {
            base.max_function_lines_warning = v;
        }
        if let Some(v) = f.max_function_lines_high {
            base.max_function_lines_high = v;
        }
        if let Some(v) = f.include_extensions {
            base.include_extensions = normalize_str_list(v, "include-extensions")?;
        }
        if let Some(v) = f.large_file_extensions {
            base.large_file_extensions = normalize_str_list(v, "large-file-extensions")?
                .into_iter()
                .collect();
        }
        if let Some(v) = f.exclude_dirs {
            base.exclude_dirs = normalize_str_list(v, "exclude-dirs")?;
        }
        if let Some(v) = f.exclude_files {
            base.exclude_files = normalize_str_list(v, "exclude-files")?;
        }
        if let Some(v) = f.exclude_severities {
            let lower: Vec<String> = v.iter().map(|s| s.to_lowercase()).collect();
            for s in &lower {
                if !VALID_SEVERITIES.contains(&s.as_str()) {
                    return Err(ConfigError::Invalid(format!(
                        "Config 'exclude-severities' must be only low, medium, high; invalid: {s}"
                    )));
                }
            }
            base.exclude_severities = lower.into_iter().collect();
        }
        if let Some(v) = f.verbose {
            base.verbose = v;
        }
        base.include_rules = f.include;
    }

    if let Some(t) = cli_fail_threshold {
        base.fail_threshold = t;
    }
    if let Some(v) = cli_verbose {
        base.verbose = v;
    }
    if let Some(m) = cli_max_file_lines {
        base.max_file_lines_warning = m;
    }
    if let Some(m) = cli_max_function_lines {
        base.max_function_lines_warning = m;
    }

    Ok(base)
}

/// When `verbose_cli` is true, scanning/reporting should show per-finding scores (CLI `-v`).
pub fn merge_config_simple(
    scan_root: &std::path::Path,
    cli_fail_threshold: Option<i32>,
    verbose_cli: bool,
    cli_max_file_lines: Option<i32>,
    cli_max_function_lines: Option<i32>,
) -> Result<Config, ConfigError> {
    let file = load_config_file(scan_root)?;
    let cli_v = if verbose_cli { Some(true) } else { None };
    merge_config(
        Config::default(),
        file,
        cli_fail_threshold,
        cli_v,
        cli_max_file_lines,
        cli_max_function_lines,
    )
}
