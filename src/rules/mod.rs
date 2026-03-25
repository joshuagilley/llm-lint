mod large_file;

pub use large_file::LargeFileRule;

/// All rule ids that exist in this build (for validating `include` in config).
pub fn registered_rule_ids() -> &'static [&'static str] {
    &[LargeFileRule::RULE_ID]
}
