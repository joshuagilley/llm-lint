use std::path::Path;

use regex::Regex;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

use crate::models::FunctionInfo;

fn patterns() -> &'static [Regex] {
    static P: OnceLock<Vec<Regex>> = OnceLock::new();
    P.get_or_init(|| {
        vec![
            Regex::new(r"^\s*(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*\(").unwrap(),
            Regex::new(r"^\s*(?:export\s+)?(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?\(")
                .unwrap(),
            Regex::new(r"^\s*(?:export\s+)?(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?function")
                .unwrap(),
            Regex::new(r"^\s*(\w+)\s*=\s*(?:async\s+)?\(").unwrap(),
        ]
    })
}

fn sha256_lines(slice: &[String]) -> String {
    let normalized: String = slice
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn find_block_end(lines: &[String], start_idx: usize) -> usize {
    let mut depth = 0i32;
    for (i, line) in lines.iter().enumerate().skip(start_idx) {
        depth += line.matches('{').count() as i32;
        depth -= line.matches('}').count() as i32;
        if depth <= 0 {
            return i + 1;
        }
    }
    lines.len()
}

pub fn parse_text(path: &Path, lines: &[String]) -> Vec<FunctionInfo> {
    let mut functions = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let mut matched = false;
        for pattern in patterns() {
            if let Some(caps) = pattern.captures(&lines[i]) {
                let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                if name.is_empty() {
                    continue;
                }
                let end_idx = find_block_end(lines, i);
                let body_hash = sha256_lines(&lines[i..end_idx]);
                functions.push(FunctionInfo {
                    name: name.to_string(),
                    file_path: path.display().to_string(),
                    line_start: (i + 1) as u32,
                    line_end: end_idx as u32,
                    body_hash,
                });
                i = end_idx;
                matched = true;
                break;
            }
        }
        if !matched {
            i += 1;
        }
    }
    functions
}
