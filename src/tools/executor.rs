// modern-cli-mcp/src/tools/executor.rs
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Clone, Default)]
pub struct CommandExecutor;

/// Options for command execution
#[derive(Debug, Default)]
pub struct ExecOptions<'a> {
    pub working_dir: Option<&'a str>,
    pub timeout_secs: Option<u64>,
    pub env: Option<&'a HashMap<String, String>>,
    pub clear_env: bool,
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self, cmd: &str, args: &[&str]) -> Result<CommandOutput, String> {
        self.run_with_options(cmd, args, ExecOptions::default())
            .await
    }

    pub async fn run_in_dir(
        &self,
        cmd: &str,
        args: &[&str],
        working_dir: Option<&str>,
    ) -> Result<CommandOutput, String> {
        self.run_with_options(
            cmd,
            args,
            ExecOptions {
                working_dir,
                ..Default::default()
            },
        )
        .await
    }

    pub async fn run_with_options(
        &self,
        cmd: &str,
        args: &[&str],
        opts: ExecOptions<'_>,
    ) -> Result<CommandOutput, String> {
        let cmd_path =
            which::which(cmd).map_err(|_| format!("Command '{}' not found in PATH", cmd))?;

        let mut command = Command::new(&cmd_path);
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(dir) = opts.working_dir {
            command.current_dir(dir);
        }

        if opts.clear_env {
            command.env_clear();
        }

        if let Some(env) = opts.env {
            for (k, v) in env {
                command.env(k, v);
            }
        }

        let output_future = command.output();

        let output = if let Some(timeout_secs) = opts.timeout_secs {
            match tokio::time::timeout(Duration::from_secs(timeout_secs), output_future).await {
                Ok(result) => result.map_err(|e| format!("Failed to execute {}: {}", cmd, e))?,
                Err(_) => {
                    return Err(format!(
                        "Command '{}' timed out after {} seconds",
                        cmd, timeout_secs
                    ))
                }
            }
        } else {
            output_future
                .await
                .map_err(|e| format!("Failed to execute {}: {}", cmd, e))?
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(CommandOutput {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout,
            stderr,
        })
    }

    pub async fn run_with_stdin(
        &self,
        cmd: &str,
        args: &[&str],
        stdin_data: &str,
    ) -> Result<CommandOutput, String> {
        use tokio::io::AsyncWriteExt;

        let cmd_path =
            which::which(cmd).map_err(|_| format!("Command '{}' not found in PATH", cmd))?;

        let mut child = Command::new(&cmd_path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", cmd, e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(stdin_data.as_bytes())
                .await
                .map_err(|e| format!("Failed to write stdin: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| format!("Failed to wait for {}: {}", cmd, e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(CommandOutput {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout,
            stderr,
        })
    }
}

#[derive(Debug)]
pub struct CommandOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl CommandOutput {
    pub fn to_result_string(&self) -> String {
        if self.success {
            if self.stdout.is_empty() {
                "(no output)".to_string()
            } else {
                self.stdout.clone()
            }
        } else {
            let mut result = String::new();
            if !self.stderr.is_empty() {
                result.push_str(&self.stderr);
            }
            if !self.stdout.is_empty() {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&self.stdout);
            }
            if result.is_empty() {
                format!(
                    "Command failed with exit code: {}",
                    self.exit_code.unwrap_or(-1)
                )
            } else {
                result
            }
        }
    }

    /// Convert output to JSON, returning the raw JSON if stdout is valid JSON,
    /// otherwise wrapping in a structured response
    pub fn to_json_string(&self) -> String {
        if self.success {
            // Try to parse as JSON first
            if let Ok(json) = serde_json::from_str::<Value>(&self.stdout) {
                return serde_json::to_string_pretty(&json).unwrap_or_else(|_| self.stdout.clone());
            }
            // Return as-is if already valid JSON array/object
            self.stdout.clone()
        } else {
            json!({
                "success": false,
                "error": if self.stderr.is_empty() { &self.stdout } else { &self.stderr },
                "exit_code": self.exit_code
            })
            .to_string()
        }
    }
}

// ============================================================================
// JSON OUTPUT HELPERS
// ============================================================================

/// File entry for JSON output (reserved for future enhanced parsing)
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_status: Option<String>,
}

/// Search match for JSON output (reserved for future enhanced parsing)
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct SearchMatch {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_text: Option<String>,
}

/// Diff entry for JSON output
#[derive(Debug, Serialize)]
pub struct DiffEntry {
    pub line_number_old: Option<u32>,
    pub line_number_new: Option<u32>,
    pub change_type: String, // "add", "remove", "context"
    pub content: String,
}

/// Diff result for JSON output
#[derive(Debug, Serialize)]
pub struct DiffResult {
    pub file_a: String,
    pub file_b: String,
    pub hunks: Vec<DiffHunk>,
}

/// Diff hunk for JSON output
#[derive(Debug, Serialize)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    pub lines: Vec<DiffEntry>,
}

/// Trash item for JSON output (reserved for future enhanced parsing)
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct TrashItem {
    pub path: String,
    pub original_path: String,
    pub deletion_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// File info for JSON output (reserved for future enhanced parsing)
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub path: String,
    pub mime_type: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
}

/// Parse eza long format output to JSON
pub fn parse_eza_to_json(output: &str, path: &str) -> String {
    let entries: Vec<Value> = output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts.last().unwrap_or(&"");
                json!({
                    "name": name,
                    "raw": line.trim()
                })
            } else {
                json!({ "name": line.trim() })
            }
        })
        .collect();

    json!({
        "path": path,
        "entries": entries,
        "count": entries.len()
    })
    .to_string()
}

/// Parse fd output to JSON
pub fn parse_fd_to_json(output: &str) -> String {
    let files: Vec<Value> = output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|path| {
            let name = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string());
            json!({
                "path": path,
                "name": name
            })
        })
        .collect();

    json!({
        "files": files,
        "count": files.len()
    })
    .to_string()
}

/// Parse unified diff output to JSON
pub fn parse_diff_to_json(output: &str, file_a: &str, file_b: &str) -> String {
    let mut hunks = Vec::new();
    let mut current_hunk: Option<DiffHunk> = None;
    let mut old_line = 0u32;
    let mut new_line = 0u32;

    for line in output.lines() {
        if line.starts_with("@@") {
            // Parse hunk header: @@ -start,count +start,count @@
            if let Some(hunk) = current_hunk.take() {
                hunks.push(hunk);
            }
            // Simple parse of @@ -a,b +c,d @@
            let parts: Vec<&str> = line.split_whitespace().collect();
            let (old_start, old_count) = parse_hunk_range(parts.get(1).unwrap_or(&"-1,1"));
            let (new_start, new_count) = parse_hunk_range(parts.get(2).unwrap_or(&"+1,1"));
            old_line = old_start;
            new_line = new_start;
            current_hunk = Some(DiffHunk {
                old_start,
                old_count,
                new_start,
                new_count,
                lines: Vec::new(),
            });
        } else if let Some(ref mut hunk) = current_hunk {
            let (change_type, content) = if let Some(rest) = line.strip_prefix('+') {
                let entry = DiffEntry {
                    line_number_old: None,
                    line_number_new: Some(new_line),
                    change_type: "add".to_string(),
                    content: rest.to_string(),
                };
                new_line += 1;
                (entry.change_type.clone(), entry)
            } else if let Some(rest) = line.strip_prefix('-') {
                let entry = DiffEntry {
                    line_number_old: Some(old_line),
                    line_number_new: None,
                    change_type: "remove".to_string(),
                    content: rest.to_string(),
                };
                old_line += 1;
                (entry.change_type.clone(), entry)
            } else if let Some(rest) = line.strip_prefix(' ') {
                let entry = DiffEntry {
                    line_number_old: Some(old_line),
                    line_number_new: Some(new_line),
                    change_type: "context".to_string(),
                    content: rest.to_string(),
                };
                old_line += 1;
                new_line += 1;
                (entry.change_type.clone(), entry)
            } else {
                continue;
            };
            let _ = change_type;
            hunk.lines.push(content);
        }
    }

    if let Some(hunk) = current_hunk {
        hunks.push(hunk);
    }

    let result = DiffResult {
        file_a: file_a.to_string(),
        file_b: file_b.to_string(),
        hunks,
    };

    serde_json::to_string_pretty(&result).unwrap_or_else(|_| {
        json!({
            "file_a": file_a,
            "file_b": file_b,
            "raw": output
        })
        .to_string()
    })
}

fn parse_hunk_range(s: &str) -> (u32, u32) {
    let s = s.trim_start_matches(['-', '+']);
    let parts: Vec<&str> = s.split(',').collect();
    let start = parts.first().and_then(|s| s.parse().ok()).unwrap_or(1);
    let count = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    (start, count)
}
/// Parse file command output to JSON
pub fn parse_file_to_json(output: &str, path: &str) -> String {
    // Format: "path: description"
    let description = output
        .strip_prefix(&format!("{}: ", path))
        .unwrap_or(output)
        .trim();

    // Try to extract MIME type if present
    let (mime_type, desc) = if description.contains(';') {
        let parts: Vec<&str> = description.splitn(2, ';').collect();
        (parts[0].trim(), description)
    } else {
        ("unknown", description)
    };

    json!({
        "path": path,
        "mime_type": mime_type,
        "description": desc,
        "type": categorize_file_type(description)
    })
    .to_string()
}

fn categorize_file_type(description: &str) -> &'static str {
    let desc_lower = description.to_lowercase();
    if desc_lower.contains("text") {
        "text"
    } else if desc_lower.contains("executable") || desc_lower.contains("elf") {
        "executable"
    } else if desc_lower.contains("image") {
        "image"
    } else if desc_lower.contains("audio") {
        "audio"
    } else if desc_lower.contains("video") {
        "video"
    } else if desc_lower.contains("archive")
        || desc_lower.contains("compressed")
        || desc_lower.contains("zip")
        || desc_lower.contains("tar")
    {
        "archive"
    } else if desc_lower.contains("directory") {
        "directory"
    } else {
        "other"
    }
}

/// Parse fzf filter output to JSON
pub fn parse_fzf_to_json(output: &str, query: &str) -> String {
    let matches: Vec<Value> = output
        .lines()
        .filter(|line| !line.is_empty())
        .enumerate()
        .map(|(i, line)| {
            json!({
                "rank": i + 1,
                "match": line.trim()
            })
        })
        .collect();

    json!({
        "query": query,
        "matches": matches,
        "count": matches.len()
    })
    .to_string()
}

/// Parse dust output to JSON
pub fn parse_dust_to_json(output: &str, path: &str) -> String {
    let entries: Vec<Value> = output
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with("Total:"))
        .filter_map(|line| {
            // dust output: "  1.2G ├── directory"
            let trimmed = line.trim();
            let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
            if parts.len() >= 2 {
                let size = parts[0];
                let name = parts[1].trim_start_matches(['├', '└', '─', '│', ' ']);
                Some(json!({
                    "size": size,
                    "name": name.trim()
                }))
            } else {
                None
            }
        })
        .collect();

    json!({
        "path": path,
        "entries": entries,
        "count": entries.len()
    })
    .to_string()
}

/// Wrap any output as JSON with optional metadata (reserved for future use)
#[allow(dead_code)]
pub fn wrap_as_json(output: &str, tool: &str, args: &[&str]) -> String {
    json!({
        "tool": tool,
        "args": args,
        "output": output.trim(),
        "lines": output.lines().count()
    })
    .to_string()
}
