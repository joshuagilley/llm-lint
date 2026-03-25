use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub rule_id: String,
    pub severity: String,
    pub file_path: String,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub message: String,
    pub score: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    pub findings: Vec<Finding>,
    pub total_score: i32,
    pub files_scanned: usize,
    pub passed: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub file_path: String,
    pub line_start: u32,
    pub line_end: u32,
    pub body_hash: String,
}

/// Per-file content for rules (lines + parsed functions where applicable).
#[derive(Debug, Clone)]
pub struct FileContext {
    pub path: PathBuf,
    pub lines: Vec<String>,
    pub language: String,
    pub functions: Vec<FunctionInfo>,
}
