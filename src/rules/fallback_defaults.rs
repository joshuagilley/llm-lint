use std::sync::OnceLock;

use regex::Regex;

use crate::models::{FileContext, Finding};
use crate::pragma::line_ignores_rule;

fn fallback_patterns() -> &'static [(String, Regex)] {
    static P: OnceLock<Vec<(String, Regex)>> = OnceLock::new();
    P.get_or_init(|| {
        vec![
            (
                "Python env fallback to primitive sentinel".into(),
                Regex::new(
                    r#"\bos\.(?:getenv|environ\.get|getenv)\s*\(\s*['"][A-Z0-9_]+['"]\s*,\s*(?:None|True|False|0|1|['"]{2}|\[\]|\{\})\s*\)"#,
                )
                .unwrap(),
            ),
            (
                "dotenv env fallback to primitive sentinel".into(),
                Regex::new(
                    r#"\bdotenv\.get\s*\(\s*['"][A-Z0-9_]+['"]\s*,\s*(?:None|True|False|0|1|['"]{2}|\[\]|\{\})\s*\)"#,
                )
                .unwrap(),
            ),
            (
                "Node process.env logical-or primitive fallback".into(),
                Regex::new(
                    r#"\bprocess\.env\.[A-Z0-9_]+\s*\|\|\s*(?:null|undefined|true|false|0|1|['"]{2}|\[\]|\{\})"#,
                )
                .unwrap(),
            ),
            (
                "Node process.env nullish primitive fallback".into(),
                Regex::new(
                    r#"\bprocess\.env\.[A-Z0-9_]+\s*\?\?\s*(?:null|undefined|true|false|0|1|['"]{2}|\[\]|\{\})"#,
                )
                .unwrap(),
            ),
            (
                "Python catch-all returns primitive".into(),
                Regex::new(
                    r#"\bexcept(?:\s+(?:Exception|BaseException))?(?:\s+as\s+\w+)?\s*:\s*return\s+(?:None|True|False|0|1|['"]{2}|\[\]|\{\})"#,
                )
                .unwrap(),
            ),
            (
                "JS catch-all returns primitive".into(),
                Regex::new(
                    r#"\bcatch\s*(?:\(\s*\w+\s*\))?\s*\{\s*return\s*(?:null|undefined|true|false|0|1|['"]{2}|\[\]|\{\})\s*;?\s*\}"#,
                )
                .unwrap(),
            ),
        ]
    })
}

pub struct FallbackDefaultsRule;

impl FallbackDefaultsRule {
    pub const RULE_ID: &'static str = "fallback-defaults";

    pub fn run(ctx: &FileContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        let mut seen_lines = std::collections::HashSet::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_no = (i + 1) as u32;
            if seen_lines.contains(&line_no) {
                continue;
            }
            let mut kinds = Vec::new();
            for (label, pattern) in fallback_patterns() {
                if pattern.is_match(line) {
                    kinds.push(label.as_str());
                }
            }
            if kinds.is_empty() {
                continue;
            }
            if line_ignores_rule(line, Self::RULE_ID) {
                continue;
            }
            seen_lines.insert(line_no);
            let kinds_str = if kinds.len() > 1 {
                kinds.join("; ")
            } else {
                kinds[0].to_string()
            };
            findings.push(Finding {
                rule_id: Self::RULE_ID.to_string(),
                severity: "medium".into(),
                file_path: ctx.path.display().to_string(),
                line_start: Some(line_no),
                line_end: Some(line_no),
                message: format!(
                    "Potential slop fallback detected ({kinds_str}). \
                     Avoid silent primitive defaults; \
                     preserve failure semantics or add observability."
                ),
                score: 4,
            });
        }
        findings
    }
}
