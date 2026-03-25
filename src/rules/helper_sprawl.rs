use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use regex::Regex;

use crate::models::{FileContext, Finding};

const HELPER_FILENAMES: &[&str] = &[
    "utils", "helpers", "common", "shared", "misc", "util", "helper",
];

fn version_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(_v\d+\w*|_safe|_new|_old|_legacy|_fixed|_updated|_temp|_tmp|_copy)$")
            .expect("version suffix regex")
    })
}

fn base_name(name: &str) -> String {
    version_re().replace(name, "").to_string()
}

pub struct HelperSprawlRule;

impl HelperSprawlRule {
    pub const RULE_ID: &'static str = "helper-sprawl";

    pub fn run(ctx: &FileContext) -> Vec<Finding> {
        let stem = ctx
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if HELPER_FILENAMES.contains(&stem.as_str()) {
            vec![Finding {
                rule_id: Self::RULE_ID.to_string(),
                severity: "low".into(),
                file_path: ctx.path.display().to_string(),
                line_start: None,
                line_end: None,
                message: format!(
                    "Generic helper filename '{}' suggests a low-cohesion catch-all module",
                    ctx.path.file_name().and_then(|n| n.to_str()).unwrap_or("")
                ),
                score: 2,
            }]
        } else {
            Vec::new()
        }
    }

    pub fn run_cross_file(contexts: &[FileContext]) -> Vec<Finding> {
        let mut base_map: HashMap<String, Vec<(String, String, u32)>> = HashMap::new();
        for ctx in contexts {
            for fn_info in &ctx.functions {
                let b = base_name(&fn_info.name);
                base_map.entry(b).or_default().push((
                    fn_info.name.clone(),
                    fn_info.file_path.clone(),
                    fn_info.line_start,
                ));
            }
        }

        let mut findings = Vec::new();
        for (base, variants) in base_map {
            let unique_names: HashSet<&str> = variants.iter().map(|(n, _, _)| n.as_str()).collect();
            if unique_names.len() < 2 {
                continue;
            }
            let locations: String = variants
                .iter()
                .map(|(name, path, line)| format!("'{name}' at {path}:{line}"))
                .collect::<Vec<_>>()
                .join(", ");
            let (_primary_name, primary_path, primary_line) = &variants[0];
            findings.push(Finding {
                rule_id: Self::RULE_ID.to_string(),
                severity: "medium".into(),
                file_path: primary_path.clone(),
                line_start: Some(*primary_line),
                line_end: None,
                message: format!(
                    "Versioned function name variants detected for '{base}': {locations}"
                ),
                score: 5,
            });
        }
        findings
    }
}
