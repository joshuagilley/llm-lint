use std::sync::OnceLock;

use regex::Regex;

fn ignore_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)(?:slopsniff|llm-lint):\s*ignore(?:\s+([\w\-]+(?:\s*,\s*[\w\-]+)*))?")
            .expect("pragma regex")
    })
}

/// True if `line` contains a pragma suppressing `rule_id`, or all rules (bare `ignore`).
pub fn line_ignores_rule(line: &str, rule_id: &str) -> bool {
    for cap in ignore_re().captures_iter(line) {
        let rest = cap.get(1).map(|g| g.as_str().trim()).unwrap_or("").trim();
        if rest.is_empty() {
            return true;
        }
        for part in rest.split(',') {
            let t = part.trim();
            if t == rule_id {
                return true;
            }
        }
    }
    false
}
