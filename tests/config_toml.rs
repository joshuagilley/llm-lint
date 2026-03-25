use std::fs;

use llm_lint::config::merge_config_simple;

#[test]
fn loads_fail_threshold_from_llm_lint_toml() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(
        root.join("llm-lint.toml"),
        r#"
fail_threshold = 99
max_file_lines_warning = 111
"#,
    )
    .expect("write toml");

    let cfg = merge_config_simple(root, None, false, None, None).expect("merge");
    assert_eq!(cfg.fail_threshold, 99);
    assert_eq!(cfg.max_file_lines_warning, 111);
}

#[test]
fn loads_kebab_case_keys_in_toml() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(
        root.join("llm-lint.toml"),
        r#""fail-threshold" = 7
"max-file-lines-warning" = 222
"#,
    )
    .expect("write toml");

    let cfg = merge_config_simple(root, None, false, None, None).expect("merge");
    assert_eq!(cfg.fail_threshold, 7);
    assert_eq!(cfg.max_file_lines_warning, 222);
}

#[test]
fn empty_toml_file_falls_back_to_json() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(root.join("llm-lint.toml"), "   \n  \n").expect("empty toml");
    fs::write(root.join("llm-lint.json"), r#"{"fail-threshold": 42}"#).expect("write json");

    let cfg = merge_config_simple(root, None, false, None, None).expect("merge");
    assert_eq!(cfg.fail_threshold, 42);
}

#[test]
fn toml_takes_precedence_over_json_when_nonempty() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(root.join("llm-lint.toml"), "fail_threshold = 1\n").expect("toml");
    fs::write(root.join("llm-lint.json"), r#"{"fail-threshold": 99}"#).expect("json");

    let cfg = merge_config_simple(root, None, false, None, None).expect("merge");
    assert_eq!(cfg.fail_threshold, 1);
}
