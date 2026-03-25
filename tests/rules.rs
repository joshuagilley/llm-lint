use std::fs;

use llm_lint::config::Config;
use llm_lint::scanner::scan;

fn py_only_scan_config() -> Config {
    Config {
        exclude_dirs: vec![],
        include_extensions: vec![".py".into()],
        fail_threshold: 10_000,
        ..Default::default()
    }
}

#[test]
fn large_function_rule_warns_when_body_exceeds_threshold() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let mut body = String::from("def big():\n");
    for i in 0..55 {
        body.push_str(&format!("    _ = {i}\n"));
    }
    body.push_str("    return 0\n");
    fs::write(root.join("mod.py"), body).expect("write");

    let mut config = py_only_scan_config();
    config.max_function_lines_warning = 50;
    config.max_function_lines_high = 200;
    config.include_rules = Some(vec!["large-function".into()]);

    let result = scan(root, &config, None).expect("scan");
    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.findings[0].rule_id, "large-function");
    assert_eq!(result.findings[0].severity, "medium");
}

#[test]
fn exposed_secrets_detects_sk_proj_style_key() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let secret = format!("API_KEY = \"sk-proj-{}\"", "x".repeat(24));
    fs::write(root.join("keys.py"), secret).expect("write");

    let mut config = py_only_scan_config();
    config.include_rules = Some(vec!["exposed-secrets".into()]);

    let result = scan(root, &config, None).expect("scan");
    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.findings[0].rule_id, "exposed-secrets");
    assert_eq!(result.findings[0].severity, "high");
}

#[test]
fn fallback_defaults_detects_os_getenv_sentinel() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(root.join("cfg.py"), "timeout = os.getenv(\"TIMEOUT\", 0)\n").expect("write");

    let mut config = py_only_scan_config();
    config.include_rules = Some(vec!["fallback-defaults".into()]);

    let result = scan(root, &config, None).expect("scan");
    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.findings[0].rule_id, "fallback-defaults");
    assert_eq!(result.findings[0].severity, "medium");
}

#[test]
fn helper_sprawl_flags_generic_utils_filename() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(root.join("utils.py"), "# helpers\nx = 1\n").expect("write");

    let mut config = py_only_scan_config();
    config.include_rules = Some(vec!["helper-sprawl".into()]);

    let result = scan(root, &config, None).expect("scan");
    let per_file: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.line_start.is_none())
        .collect();
    assert_eq!(per_file.len(), 1);
    assert_eq!(per_file[0].rule_id, "helper-sprawl");
    assert_eq!(per_file[0].severity, "low");
}

#[test]
fn helper_sprawl_cross_file_versioned_function_names() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(root.join("a.py"), "def foo_v1():\n    return 1\n").expect("write");
    fs::write(root.join("b.py"), "def foo_v2():\n    return 2\n").expect("write");

    let mut config = py_only_scan_config();
    config.include_rules = Some(vec!["helper-sprawl".into()]);

    let result = scan(root, &config, None).expect("scan");
    let cross: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.message.contains("Versioned function name variants"))
        .collect();
    assert_eq!(cross.len(), 1);
    assert_eq!(cross[0].severity, "medium");
}

#[test]
fn duplicate_functions_finds_identical_bodies_across_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let body = "def shared():\n    a = 1\n    b = 2\n    c = 3\n    d = 4\n    return a + b\n";
    fs::write(root.join("one.py"), body).expect("write");
    fs::write(root.join("two.py"), body).expect("write");

    let mut config = py_only_scan_config();
    config.include_rules = Some(vec!["duplicate-functions".into()]);

    let result = scan(root, &config, None).expect("scan");
    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.findings[0].rule_id, "duplicate-functions");
    assert_eq!(result.findings[0].severity, "high");
}
