use std::sync::OnceLock;

use regex::Regex;

use crate::models::{FileContext, Finding};
use crate::pragma::line_ignores_rule;

fn secret_patterns() -> &'static [(String, Regex)] {
    static P: OnceLock<Vec<(String, Regex)>> = OnceLock::new();
    P.get_or_init(|| {
        vec![
            (
                "PEM private key header".into(),
                Regex::new(r"-----BEGIN [A-Z0-9 +\-]+PRIVATE KEY-----").unwrap(),
            ),
            (
                "AWS access key id".into(),
                Regex::new(r"\b(?:AKIA|ASIA|AROA)[0-9A-Z]{16}\b").unwrap(),
            ),
            (
                "GitHub personal access token (classic)".into(),
                Regex::new(r"\bghp_[a-zA-Z0-9]{36,}\b").unwrap(),
            ),
            (
                "GitHub fine-grained PAT".into(),
                Regex::new(r"github_pat_[a-zA-Z0-9_]{20,}").unwrap(),
            ),
            (
                "Slack bot/user token".into(),
                Regex::new(r"xox[bpa]-[0-9]{10,13}-[0-9]{10,13}-[a-zA-Z0-9]{24,}").unwrap(),
            ),
            (
                "Stripe secret key".into(),
                Regex::new(r"\bsk_(?:live|test)_[0-9a-zA-Z]{20,}\b").unwrap(),
            ),
            (
                "OpenAI API key (sk-proj-)".into(),
                Regex::new(r"\bsk-proj-[a-zA-Z0-9_-]{20,}\b").unwrap(),
            ),
            (
                "OpenAI-style API key (long sk- prefix)".into(),
                Regex::new(r"\bsk-[a-zA-Z0-9]{45,}\b").unwrap(),
            ),
            (
                "Anthropic API key".into(),
                Regex::new(r"\bsk-ant-api03-[a-zA-Z0-9_-]{20,}\b").unwrap(),
            ),
            (
                "Google API key".into(),
                Regex::new(r"\bAIza[0-9A-Za-z\-_]{35}\b").unwrap(),
            ),
            (
                "SendGrid API key".into(),
                Regex::new(r"SG\.[a-zA-Z0-9_-]{22}\.[a-zA-Z0-9_-]{40,}").unwrap(),
            ),
        ]
    })
}

pub struct ExposedSecretsRule;

impl ExposedSecretsRule {
    pub const RULE_ID: &'static str = "exposed-secrets";

    pub fn run(ctx: &FileContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        let mut seen_lines = std::collections::HashSet::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_no = (i + 1) as u32;
            if seen_lines.contains(&line_no) {
                continue;
            }
            let mut kinds = Vec::new();
            for (label, pattern) in secret_patterns() {
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
                severity: "high".into(),
                file_path: ctx.path.display().to_string(),
                line_start: Some(line_no),
                line_end: Some(line_no),
                message: format!(
                    "Possible exposed secret ({kinds_str}). \
                     Remove or rotate the credential; use env vars or a secret manager."
                ),
                score: 10,
            });
        }
        findings
    }
}
