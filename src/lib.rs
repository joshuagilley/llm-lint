//! Core scan API for llm-lint.

pub mod config;
pub mod models;
mod pragma;
pub mod parsers;
pub mod rules;
pub mod scanner;
pub mod scoring;
pub mod walker;

pub use config::Config;
pub use models::{FileContext, Finding, FunctionInfo, ScanResult};
pub use scanner::scan;
