use crate::config::Config;
use crate::models::{FileContext, Finding};

pub struct LargeFunctionRule<'a> {
    config: &'a Config,
}

impl<'a> LargeFunctionRule<'a> {
    pub const RULE_ID: &'static str = "large-function";

    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn run(&self, ctx: &FileContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        for fn_info in &ctx.functions {
            let length = fn_info.line_end as i32 - fn_info.line_start as i32 + 1;

            if length >= self.config.max_function_lines_high {
                findings.push(Finding {
                    rule_id: Self::RULE_ID.to_string(),
                    severity: "high".into(),
                    file_path: ctx.path.display().to_string(),
                    line_start: Some(fn_info.line_start),
                    line_end: Some(fn_info.line_end),
                    message: format!(
                        "Function '{}' is {length} lines long (high threshold: {})",
                        fn_info.name, self.config.max_function_lines_high
                    ),
                    score: 10,
                });
            } else if length >= self.config.max_function_lines_warning {
                findings.push(Finding {
                    rule_id: Self::RULE_ID.to_string(),
                    severity: "medium".into(),
                    file_path: ctx.path.display().to_string(),
                    line_start: Some(fn_info.line_start),
                    line_end: Some(fn_info.line_end),
                    message: format!(
                        "Function '{}' is {length} lines long (warning threshold: {})",
                        fn_info.name, self.config.max_function_lines_warning
                    ),
                    score: 5,
                });
            }
        }
        findings
    }
}
