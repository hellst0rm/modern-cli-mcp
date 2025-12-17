// modern-cli-mcp/src/format.rs
//! Summary formatters for dual-response mode.
//!
//! When dual-response is enabled, tools return both:
//! 1. A human-readable formatted summary (this module)
//! 2. Raw structured data (JSON/JSONL) for LLM processing

use serde_json::Value;

/// Format eza directory listing summary
pub fn format_eza_summary(json: &str, path: &str) -> String {
    if let Ok(v) = serde_json::from_str::<Value>(json) {
        let count = v.get("count").and_then(|c| c.as_u64()).unwrap_or(0);
        let entries = v.get("entries").and_then(|e| e.as_array());

        let (files, dirs) = entries
            .map(|arr| {
                arr.iter().fold((0, 0), |(f, d), e| {
                    if e.get("type").and_then(|t| t.as_str()) == Some("directory") {
                        (f, d + 1)
                    } else {
                        (f + 1, d)
                    }
                })
            })
            .unwrap_or((0, 0));

        format!(
            "Listed {} items in {}\n\n{} files, {} directories",
            count, path, files, dirs
        )
    } else {
        format!("Listed directory: {}", path)
    }
}

/// Format fd file search summary
pub fn format_fd_summary(json: &str, pattern: &str) -> String {
    if let Ok(v) = serde_json::from_str::<Value>(json) {
        let count = v.get("count").and_then(|c| c.as_u64()).unwrap_or(0);
        format!("Found {} matches for pattern '{}'", count, pattern)
    } else {
        format!("Search completed for '{}'", pattern)
    }
}

/// Format ripgrep search summary
pub fn format_rg_summary(output: &str, pattern: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let match_count = lines.len();

    // Count unique files
    let files: std::collections::HashSet<&str> = lines
        .iter()
        .filter_map(|line| line.split(':').next())
        .collect();

    format!(
        "Found {} matches in {} files for '{}'",
        match_count,
        files.len(),
        pattern
    )
}

/// Format git status summary
pub fn format_git_status_summary(json: &str) -> String {
    if let Ok(v) = serde_json::from_str::<Value>(json) {
        let branch = v
            .get("branch")
            .and_then(|b| b.get("head"))
            .and_then(|h| h.as_str())
            .unwrap_or("unknown");

        let files = v.get("files").and_then(|f| f.as_array());
        let (modified, added, deleted, untracked) = files
            .map(|arr| {
                arr.iter().fold((0, 0, 0, 0), |(m, a, d, u), f| {
                    let status = f.get("status").and_then(|s| s.as_str()).unwrap_or("");
                    match status.chars().next() {
                        Some('M') | Some('.') if status.contains('M') => (m + 1, a, d, u),
                        Some('A') => (m, a + 1, d, u),
                        Some('D') => (m, a, d + 1, u),
                        Some('?') => (m, a, d, u + 1),
                        _ => (m, a, d, u),
                    }
                })
            })
            .unwrap_or((0, 0, 0, 0));

        let ahead = v
            .get("branch")
            .and_then(|b| b.get("ahead"))
            .and_then(|a| a.as_u64())
            .unwrap_or(0);
        let behind = v
            .get("branch")
            .and_then(|b| b.get("behind"))
            .and_then(|a| a.as_u64())
            .unwrap_or(0);

        let mut summary = format!("Branch: {}\n", branch);

        if modified + added + deleted + untracked > 0 {
            summary.push_str("\nChanges:\n");
            if modified > 0 {
                summary.push_str(&format!("  {} modified\n", modified));
            }
            if added > 0 {
                summary.push_str(&format!("  {} added\n", added));
            }
            if deleted > 0 {
                summary.push_str(&format!("  {} deleted\n", deleted));
            }
            if untracked > 0 {
                summary.push_str(&format!("  {} untracked\n", untracked));
            }
        } else {
            summary.push_str("\nWorking tree clean");
        }

        if ahead > 0 || behind > 0 {
            summary.push('\n');
            if ahead > 0 {
                summary.push_str(&format!("Ahead by {} commits", ahead));
            }
            if behind > 0 {
                if ahead > 0 {
                    summary.push_str(", ");
                }
                summary.push_str(&format!("Behind by {} commits", behind));
            }
        }

        summary
    } else {
        "Git status retrieved".to_string()
    }
}

/// Format git diff summary
pub fn format_git_diff_summary(diff: &str) -> String {
    let lines: Vec<&str> = diff.lines().collect();
    let additions = lines
        .iter()
        .filter(|l| l.starts_with('+') && !l.starts_with("+++"))
        .count();
    let deletions = lines
        .iter()
        .filter(|l| l.starts_with('-') && !l.starts_with("---"))
        .count();
    let files: std::collections::HashSet<&str> = lines
        .iter()
        .filter(|l| l.starts_with("diff --git"))
        .filter_map(|l| l.split(' ').nth(2))
        .collect();

    format!(
        "{} files changed, {} insertions(+), {} deletions(-)",
        files.len(),
        additions,
        deletions
    )
}

/// Format git log summary
pub fn format_git_log_summary(output: &str) -> String {
    let commit_count = output
        .lines()
        .filter(|l| l.starts_with("commit ") || l.len() == 40)
        .count();
    format!("Showing {} commits", commit_count)
}

/// Format bat/file view summary
pub fn format_bat_summary(path: &str, line_count: usize) -> String {
    format!("Viewing {} ({} lines)", path, line_count)
}

/// Format dust disk usage summary
pub fn format_dust_summary(json: &str) -> String {
    if let Ok(v) = serde_json::from_str::<Value>(json) {
        let total = v
            .get("total_size")
            .and_then(|t| t.as_str())
            .unwrap_or("unknown");
        let count = v
            .get("entries")
            .and_then(|e| e.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        format!("Analyzed {} items, total size: {}", count, total)
    } else {
        "Disk usage analysis complete".to_string()
    }
}

/// Format generic command output summary
pub fn format_generic_summary(tool: &str, success: bool, output_lines: usize) -> String {
    if success {
        format!(
            "{} completed successfully ({} lines of output)",
            tool, output_lines
        )
    } else {
        format!("{} completed with errors", tool)
    }
}

/// Format file read summary
pub fn format_file_read_summary(path: &str, line_count: usize) -> String {
    format!("Read {} ({} lines)", path, line_count)
}

/// Format file write summary
pub fn format_file_write_summary(path: &str, bytes: usize) -> String {
    format!("Wrote {} ({} bytes)", path, bytes)
}

/// Format file edit summary
pub fn format_file_edit_summary(path: &str, replacements: usize) -> String {
    format!("Edited {} ({} replacements)", path, replacements)
}

/// Format kubectl/k8s summary
#[allow(dead_code)]
pub fn format_kubectl_summary(resource: &str, count: usize) -> String {
    format!("Retrieved {} {} resources", count, resource)
}

/// Format container/podman summary
#[allow(dead_code)]
pub fn format_container_summary(action: &str, target: &str) -> String {
    format!("{} on {}", action, target)
}

/// Format HTTP request summary
#[allow(dead_code)]
pub fn format_http_summary(method: &str, url: &str, status: u16) -> String {
    format!("{} {} -> {}", method, url, status)
}

/// Format SQL query summary
#[allow(dead_code)]
pub fn format_sql_summary(rows: usize) -> String {
    format!("Query returned {} rows", rows)
}

/// Format text processing summary (jq, yq, etc.)
pub fn format_text_summary(tool: &str, input_lines: usize, output_lines: usize) -> String {
    format!(
        "{}: {} lines in, {} lines out",
        tool, input_lines, output_lines
    )
}
