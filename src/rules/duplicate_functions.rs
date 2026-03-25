use std::collections::HashMap;

use crate::models::{FileContext, Finding, FunctionInfo};

const MIN_LINES: i32 = 5;

pub struct DuplicateFunctionsRule;

impl DuplicateFunctionsRule {
    pub const RULE_ID: &'static str = "duplicate-functions";

    pub fn run_cross_file(contexts: &[FileContext]) -> Vec<Finding> {
        let mut hash_map: HashMap<&str, Vec<&FunctionInfo>> = HashMap::new();
        for ctx in contexts {
            for fn_info in &ctx.functions {
                let len = fn_info.line_end as i32 - fn_info.line_start as i32 + 1;
                if len < MIN_LINES {
                    continue;
                }
                hash_map
                    .entry(fn_info.body_hash.as_str())
                    .or_default()
                    .push(fn_info);
            }
        }

        let mut findings = Vec::new();
        for fns in hash_map.values() {
            if fns.len() < 2 {
                continue;
            }
            let locations: String = fns
                .iter()
                .map(|fn_info| {
                    format!(
                        "{}:{}-{}",
                        fn_info.file_path, fn_info.line_start, fn_info.line_end
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let primary = fns[0];
            findings.push(Finding {
                rule_id: Self::RULE_ID.to_string(),
                severity: "high".into(),
                file_path: primary.file_path.clone(),
                line_start: Some(primary.line_start),
                line_end: Some(primary.line_end),
                message: format!(
                    "Duplicate function body found in {} locations: {locations}",
                    fns.len()
                ),
                score: 10,
            });
        }
        findings
    }
}
