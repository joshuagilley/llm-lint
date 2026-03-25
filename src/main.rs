use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Row, Table};
use crossterm::style::Stylize;
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

#[derive(Args)]
#[group(id = "git_diff", multiple = false)]
struct GitDiffOpts {
    /// Only files changed vs `main` (same as `--changed-since main`).
    #[arg(long, group = "git_diff")]
    branch: bool,
    /// Only files changed vs `REF` (`git diff REF --name-only --diff-filter=ACMR`).
    #[arg(long = "changed-since", value_name = "REF", group = "git_diff")]
    changed_since: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan files under PATH (default: current directory)
    Scan {
        #[command(flatten)]
        git_diff: GitDiffOpts,
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

fn resolve_git_diff_ref(opts: &GitDiffOpts) -> Result<Option<String>, String> {
    if opts.branch {
        return Ok(Some("main".into()));
    }
    if let Some(r) = &opts.changed_since {
        let t = r.trim();
        if t.is_empty() {
            return Err("--changed-since requires a non-empty ref".into());
        }
        return Ok(Some(t.into()));
    }
    Ok(None)
}

fn finding_loc(f: &llm_lint::Finding) -> String {
    match (f.line_start, f.line_end) {
        (None, _) => "file".into(),
        (Some(a), None) => a.to_string(),
        (Some(a), Some(b)) if a == b => a.to_string(),
        (Some(a), Some(b)) => format!("{a}-{b}"),
    }
}

fn severity_color(sev: &str) -> Color {
    match sev.to_lowercase().as_str() {
        "high" => Color::Red,
        "medium" => Color::Yellow,
        "low" => Color::DarkGrey,
        _ => Color::Reset,
    }
}

fn status_color(status: &str) -> Color {
    match status {
        "healthy" => Color::Green,
        "warning" => Color::Yellow,
        _ => Color::Red,
    }
}

fn message_wrap_width() -> usize {
    let w = crossterm::terminal::size()
        .map(|(cols, _)| cols as usize)
        .unwrap_or(100);
    w.saturating_sub(36).clamp(40, 120)
}

fn print_terminal(result: &llm_lint::ScanResult, verbose: bool) {
    let status = grade(result.total_score);
    let n = result.findings.len();
    let issues_label = if n == 1 { "issue" } else { "issues" };

    let mut summary = Table::new();
    summary
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(vec![
            Cell::new("Tool").add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Files").add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Issues").add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Score").add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Status").add_attribute(comfy_table::Attribute::Bold),
        ]);
    let status_upper = status.to_uppercase();
    summary.add_row(Row::from(vec![
        Cell::new("llm-lint"),
        Cell::new(result.files_scanned),
        Cell::new(format!("{n} {issues_label}")),
        Cell::new(result.total_score),
        Cell::new(&status_upper)
            .fg(status_color(status))
            .add_attribute(comfy_table::Attribute::Bold),
    ]));
    println!("{summary}");

    if result.findings.is_empty() {
        println!();
        let mut t = Table::new();
        t.load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .add_row(Row::from(vec![Cell::new("No issues.").fg(Color::Green)]));
        println!("{t}");
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

    let wrap_w = message_wrap_width();

    let use_color = io::stdout().is_terminal();
    let mut out = io::stdout().lock();

    for (dir, group) in by_dir {
        writeln!(out).ok();
        if use_color {
            writeln!(out, "{}", format!("{dir}/").cyan().bold()).ok();
        } else {
            writeln!(out, "{dir}/").ok();
        }

        let mut by_file: BTreeMap<String, Vec<&llm_lint::Finding>> = BTreeMap::new();
        for f in group {
            by_file.entry(f.file_path.clone()).or_default().push(f);
        }
        for (fp, mut findings) in by_file {
            let name = std::path::Path::new(&fp)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&fp);

            if use_color {
                writeln!(out, "  {}", name.white().bold()).ok();
            } else {
                writeln!(out, "  {name}").ok();
            }

            let mut ft = Table::new();
            ft.load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::DynamicFullWidth);

            if verbose {
                ft.set_header(vec![
                    Cell::new("Location").add_attribute(comfy_table::Attribute::Bold),
                    Cell::new("Severity").add_attribute(comfy_table::Attribute::Bold),
                    Cell::new("Rule").add_attribute(comfy_table::Attribute::Bold),
                    Cell::new("Pts").add_attribute(comfy_table::Attribute::Bold),
                    Cell::new("Message").add_attribute(comfy_table::Attribute::Bold),
                ]);
            } else {
                ft.set_header(vec![
                    Cell::new("Location").add_attribute(comfy_table::Attribute::Bold),
                    Cell::new("Severity").add_attribute(comfy_table::Attribute::Bold),
                    Cell::new("Rule").add_attribute(comfy_table::Attribute::Bold),
                    Cell::new("Message").add_attribute(comfy_table::Attribute::Bold),
                ]);
            }

            findings.sort_by(|a, b| {
                let sa = severity_order(&a.severity);
                let sb = severity_order(&b.severity);
                sa.cmp(&sb)
                    .then(a.line_start.unwrap_or(0).cmp(&b.line_start.unwrap_or(0)))
                    .then(a.rule_id.cmp(&b.rule_id))
            });

            for f in findings {
                let loc = finding_loc(f);
                let msg = textwrap::fill(&f.message, wrap_w);
                let sev_upper = f.severity.to_uppercase();
                if verbose {
                    ft.add_row(Row::from(vec![
                        Cell::new(loc),
                        Cell::new(&sev_upper).fg(severity_color(&f.severity)),
                        Cell::new(&f.rule_id),
                        Cell::new(f.score),
                        Cell::new(msg),
                    ]));
                } else {
                    ft.add_row(Row::from(vec![
                        Cell::new(loc),
                        Cell::new(&sev_upper).fg(severity_color(&f.severity)),
                        Cell::new(&f.rule_id),
                        Cell::new(msg),
                    ]));
                }
            }
            writeln!(out, "{ft}").ok();
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
        git_diff,
        path,
        fail_threshold,
        format,
        verbose,
        max_file_lines,
        max_function_lines,
    } = cli.command;

    let changed_since = match resolve_git_diff_ref(&git_diff) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {e}");
            return std::process::ExitCode::from(1);
        }
    };

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

    let changed = changed_since.as_deref();
    let result = match scan(&scan_root, &config, changed) {
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
