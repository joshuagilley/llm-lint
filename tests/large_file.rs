use std::collections::HashSet;
use std::fs;

use llm_lint::config::Config;
use llm_lint::scanner::scan;

fn big_py_lines(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str(&format!("# line {i}\n"));
    }
    s
}

#[test]
fn large_file_rule_warns_over_threshold() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(root.join("big.py"), big_py_lines(450)).expect("write");

    let config = Config {
        exclude_dirs: vec![], // allow scanning everything under temp dir
        include_extensions: vec![".py".into()],
        large_file_extensions: [".py".into()].into_iter().collect(),
        max_file_lines_warning: 400,
        max_file_lines_high: 800,
        fail_threshold: 100,
        ..Default::default()
    };

    let result = scan(root, &config, None).expect("scan");
    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.findings[0].rule_id, "large-file");
    assert_eq!(result.findings[0].severity, "medium");
}

#[test]
fn unknown_include_rule_errors() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(root.join("a.py"), "x = 1\n").expect("write");

    let config = Config {
        exclude_dirs: vec![],
        include_extensions: vec![".py".into()],
        include_rules: Some(vec!["large-file".into(), "not-a-rule".into()]),
        ..Default::default()
    };

    let err = scan(root, &config, None).expect_err("expected unknown rule");
    assert!(err.to_string().contains("not-a-rule"), "got {}", err);
}

#[test]
fn exclude_severities_drops_high_findings() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let token = format!("ghp_{}", "a".repeat(36));
    fs::write(
        root.join("app.py"),
        format!("api_key = \"{token}\"\ntimeout = os.getenv(\"TIMEOUT\", 0)\n"),
    )
    .expect("write");

    let config = Config {
        exclude_dirs: vec![],
        include_extensions: vec![".py".into()],
        exclude_severities: HashSet::from(["high".to_string()]),
        ..Default::default()
    };

    let result = scan(root, &config, None).expect("scan");
    assert!(result
        .findings
        .iter()
        .all(|f| f.severity.to_lowercase() != "high"));
    assert!(result
        .findings
        .iter()
        .any(|f| f.rule_id == "fallback-defaults"));
    assert!(!result
        .findings
        .iter()
        .any(|f| f.rule_id == "exposed-secrets"));
    let expected: i32 = result.findings.iter().map(|f| f.score).sum();
    assert_eq!(result.total_score, expected);
}
