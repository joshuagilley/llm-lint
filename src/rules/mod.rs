mod duplicate_functions;
mod exposed_secrets;
mod fallback_defaults;
mod helper_sprawl;
mod large_file;
mod large_function;

pub use duplicate_functions::DuplicateFunctionsRule;
pub use exposed_secrets::ExposedSecretsRule;
pub use fallback_defaults::FallbackDefaultsRule;
pub use helper_sprawl::HelperSprawlRule;
pub use large_file::LargeFileRule;
pub use large_function::LargeFunctionRule;

/// All rule ids that exist in this build (for validating `include` in config).
pub fn registered_rule_ids() -> &'static [&'static str] {
    &[
        LargeFileRule::RULE_ID,
        LargeFunctionRule::RULE_ID,
        DuplicateFunctionsRule::RULE_ID,
        HelperSprawlRule::RULE_ID,
        ExposedSecretsRule::RULE_ID,
        FallbackDefaultsRule::RULE_ID,
    ]
}
