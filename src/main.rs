use std::path::PathBuf;

use clap::{Parser, Subcommand};
use llm_lint::config::merge_config_simple;
use llm_lint::scanner::scan;
use llm_lint::scoring::grade;
use serde::Serialize;

#[derive(Parser)]
#[command(
    name = "llm-lint",
    version,
    about = "Catch AI slop and code quality drift before it hardens."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan files under PATH (default: current directory)
    Scan {
        #[arg(default_value = ".")]
        path: String,
        #[arg(
            long = "fail-threshold",
            short = 't',
            help = "Score at which the command exits with failure"
        )]
        fail_threshold: Option<i32>,
        #[arg(long = "format", short = 'f', default_value = "terminal", value_parser = ["terminal", "json"])]
        format: String,
        #[arg(long, short = 'v', help = "Show score per finding")]
        verbose: bool,
        #[arg(long = "max-file-lines", help = "Override file line warning threshold")]
        max_file_lines: Option<i32>,
        #[arg(
            long = "max-function-lines",
            help = "Override function line warning threshold"
        )]
        max_function_lines: Option<i32>,
    },
}

#[derive(Serialize)]
struct JsonReport<'a> {
    files_scanned: usize,
    total_score: i32,
    status: &'a str,
    passed: bool,
    findings: &'a [llm_lint::Finding],
}

fn resolve_scan_root(path: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(path);
    let meta = p
        .metadata()
        .map_err(|_| format!("path '{}' does not exist", path))?;
    if meta.is_dir() {
        Ok(p.canonicalize().unwrap_or(p))
    } else {
        p.parent()
            .map(|parent| parent.canonicalize().unwrap_or(parent.to_path_buf()))
            .ok_or_else(|| format!("path '{}' has no parent directory", path))
    }
}

fn finding_loc(f: &llm_lint::Finding) -> String {
    match (f.line_start, f.line_end) {
        (None, _) => "file".into(),
        (Some(a), None) => a.to_string(),
        (Some(a), Some(b)) if a == b => a.to_string(),
        (Some(a), Some(b)) => format!("{a}-{b}"),
    }
}

fn print_terminal(result: &llm_lint::ScanResult, verbose: bool) {
    let status = grade(result.total_score);
    let n = result.findings.len();
    let issues = if n == 1 { "issue" } else { "issues" };
    println!(
        "llm-lint {} files  {} {}  score {}  {}",
        result.files_scanned,
        n,
        issues,
        result.total_score,
        status.to_uppercase()
    );

    if result.findings.is_empty() {
        println!("No issues.");
        return;
    }

    use std::collections::BTreeMap;
    let mut by_dir: BTreeMap<String, Vec<&llm_lint::Finding>> = BTreeMap::new();
    for f in &result.findings {
        let path = std::path::Path::new(&f.file_path);
        let dir = path
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        by_dir.entry(dir).or_default().push(f);
    }

    for (dir, group) in by_dir {
        println!("{}/", dir);
        let mut by_file: BTreeMap<String, Vec<&llm_lint::Finding>> = BTreeMap::new();
        for f in group {
            by_file.entry(f.file_path.clone()).or_default().push(f);
        }
        for (fp, mut findings) in by_file {
            let name = std::path::Path::new(&fp)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&fp);
            println!("  {}", name);
            findings.sort_by(|a, b| {
                let sa = severity_order(&a.severity);
                let sb = severity_order(&b.severity);
                sa.cmp(&sb)
                    .then(a.line_start.unwrap_or(0).cmp(&b.line_start.unwrap_or(0)))
                    .then(a.rule_id.cmp(&b.rule_id))
            });
            for f in findings {
                let loc = finding_loc(f);
                println!(
                    "    {} [{}] {}  {}",
                    loc,
                    f.severity.to_uppercase(),
                    f.rule_id,
                    f.message
                );
                if verbose {
                    println!("      (+{})", f.score);
                }
            }
        }
    }
}

fn severity_order(s: &str) -> u8 {
    match s.to_lowercase().as_str() {
        "high" => 0,
        "medium" => 1,
        "low" => 2,
        _ => 3,
    }
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    let Commands::Scan {
        path,
        fail_threshold,
        format,
        verbose,
        max_file_lines,
        max_function_lines,
    } = cli.command;

    let scan_root = match resolve_scan_root(&path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {e}");
            return std::process::ExitCode::from(1);
        }
    };

    let config = match merge_config_simple(
        &scan_root,
        fail_threshold,
        verbose,
        max_file_lines,
        max_function_lines,
    ) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            return std::process::ExitCode::from(1);
        }
    };

    let result = match scan(&scan_root, &config) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {e}");
            return std::process::ExitCode::from(1);
        }
    };

    if format == "json" {
        let status = grade(result.total_score);
        let report = JsonReport {
            files_scanned: result.files_scanned,
            total_score: result.total_score,
            status,
            passed: result.passed,
            findings: &result.findings,
        };
        match serde_json::to_string_pretty(&report) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("Error: {e}");
                return std::process::ExitCode::from(1);
            }
        }
    } else {
        print_terminal(&result, config.verbose);
    }

    if result.passed {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::from(1)
    }
}
