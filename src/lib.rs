//! Core scan API for llm-lint.

pub mod config;
mod git_scope;
pub mod models;
pub mod parsers;
mod pragma;
pub mod rules;
pub mod scanner;
pub mod scoring;
pub mod walker;

pub use config::Config;
pub use models::{FileContext, Finding, FunctionInfo, ScanResult};
pub use scanner::scan;
