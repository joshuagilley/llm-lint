use std::path::Path;

use sha2::{Digest, Sha256};
use textwrap::dedent;
use tree_sitter::Parser;
use tree_sitter_python::LANGUAGE;

use crate::models::FunctionInfo;

pub fn parse_python(path: &Path, lines: &[String]) -> Vec<FunctionInfo> {
    let source = lines.join("\n");
    let mut parser = Parser::new();
    let lang: tree_sitter::Language = LANGUAGE.into();
    if parser.set_language(&lang).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source.as_bytes(), None) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    collect_functions(tree.root_node(), source.as_bytes(), path, lines, &mut out);
    out
}

fn normalize_python_body_hash(lines: &[String], line_start: u32, line_end: u32) -> String {
    let start_idx = line_start.saturating_sub(1) as usize;
    let end_idx = line_end as usize;
    if start_idx >= lines.len() {
        return String::new();
    }
    let end_idx = end_idx.min(lines.len());
    if start_idx >= end_idx {
        return String::new();
    }
    let slice = &lines[start_idx..end_idx];
    let joined = slice.join("\n");
    let dedented = dedent(&joined);
    let normalized: String = dedented
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn collect_functions(
    node: tree_sitter::Node,
    source: &[u8],
    path: &Path,
    lines: &[String],
    out: &mut Vec<FunctionInfo>,
) {
    if node.kind() == "function_definition" {
        if let Some(name_node) = node.child_by_field_name("name") {
            if let Ok(name) = name_node.utf8_text(source) {
                let start_row = node.start_position().row;
                let end_row = node.end_position().row;
                let line_start = (start_row + 1) as u32;
                let line_end = (end_row + 1) as u32;
                let body_hash = normalize_python_body_hash(lines, line_start, line_end);
                out.push(FunctionInfo {
                    name: name.to_string(),
                    file_path: path.display().to_string(),
                    line_start,
                    line_end,
                    body_hash,
                });
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_functions(child, source, path, lines, out);
    }
}
