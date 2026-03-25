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
    fs::write(
        root.join("big.py"),
        big_py_lines(450),
    )
    .expect("write");

    let mut config = Config::default();
    config.exclude_dirs = vec![]; // allow scanning everything under temp dir
    config.include_extensions = vec![".py".into()];
    config.large_file_extensions = [".py".into()].into_iter().collect();
    config.max_file_lines_warning = 400;
    config.max_file_lines_high = 800;
    config.fail_threshold = 100;

    let result = scan(root, &config).expect("scan");
    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.findings[0].rule_id, "large-file");
    assert_eq!(result.findings[0].severity, "medium");
}

#[test]
fn unknown_include_rule_errors() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(root.join("a.py"), "x = 1\n").expect("write");

    let mut config = Config::default();
    config.exclude_dirs = vec![];
    config.include_extensions = vec![".py".into()];
    config.include_rules = Some(vec!["large-file".into(), "not-a-rule".into()]);

    let err = scan(root, &config).expect_err("expected unknown rule");
    assert!(
        err.to_string().contains("not-a-rule"),
        "got {}",
        err
    );
}
