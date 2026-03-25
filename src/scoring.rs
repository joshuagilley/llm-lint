use crate::models::Finding;

pub fn compute_score(findings: &[Finding]) -> i32 {
    findings.iter().map(|f| f.score).sum()
}

pub fn grade(score: i32) -> &'static str {
    if score < 10 {
        "healthy"
    } else if score < 20 {
        "warning"
    } else {
        "fail"
    }
}
