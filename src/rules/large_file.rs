use crate::config::Config;
use crate::models::{FileContext, Finding};

pub struct LargeFileRule<'a> {
    config: &'a Config,
}

impl<'a> LargeFileRule<'a> {
    pub const RULE_ID: &'static str = "large-file";

    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn run(&self, ctx: &FileContext) -> Vec<Finding> {
        let suffix = ctx
            .path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| format!(".{}", s.to_lowercase()))
            .unwrap_or_default();

        if !self
            .config
            .large_file_extensions
            .iter()
            .any(|ext| ext.eq_ignore_ascii_case(&suffix))
        {
            return Vec::new();
        }

        let count = ctx.lines.len() as i32;

        if count >= self.config.max_file_lines_high {
            return vec![Finding {
                rule_id: Self::RULE_ID.to_string(),
                severity: "high".into(),
                file_path: ctx.path.display().to_string(),
                line_start: None,
                line_end: None,
                message: format!(
                    "File is {count} lines long (high threshold: {})",
                    self.config.max_file_lines_high
                ),
                score: 10,
            }];
        }

        if count >= self.config.max_file_lines_warning {
            return vec![Finding {
                rule_id: Self::RULE_ID.to_string(),
                severity: "medium".into(),
                file_path: ctx.path.display().to_string(),
                line_start: None,
                line_end: None,
                message: format!(
                    "File is {count} lines long (warning threshold: {})",
                    self.config.max_file_lines_warning
                ),
                score: 5,
            }];
        }

        Vec::new()
    }
}
