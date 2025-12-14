// tests/tool_integration.rs
//! Integration tests for MCP tool execution
//! These tests require the CLI tools to be available in PATH (run via `nix develop`)

use serde_json::Value;
use std::process::Command;

/// Helper to check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Test eza produces valid output
#[test]
#[ignore = "requires eza in PATH"]
fn test_eza_execution() {
    if !command_exists("eza") {
        eprintln!("Skipping: eza not in PATH");
        return;
    }

    let output = Command::new("eza")
        .args(["--oneline", "."])
        .output()
        .expect("Failed to execute eza");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
}

/// Test fd produces valid output
#[test]
#[ignore = "requires fd in PATH"]
fn test_fd_execution() {
    if !command_exists("fd") {
        eprintln!("Skipping: fd not in PATH");
        return;
    }

    let output = Command::new("fd")
        .args(["--type", "f", "--max-depth", "1", "."])
        .output()
        .expect("Failed to execute fd");

    assert!(output.status.success());
}

/// Test ripgrep with JSON output
#[test]
#[ignore = "requires rg in PATH"]
fn test_rg_json_output() {
    if !command_exists("rg") {
        eprintln!("Skipping: rg not in PATH");
        return;
    }

    let output = Command::new("rg")
        .args(["--json", "fn", "src/"])
        .output()
        .expect("Failed to execute rg");

    // rg returns exit code 1 if no matches, which is fine
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Each line should be valid JSON (JSONL format)
    for line in stdout.lines() {
        if !line.is_empty() {
            let parsed: Result<Value, _> = serde_json::from_str(line);
            assert!(parsed.is_ok(), "Invalid JSON line: {}", line);
        }
    }
}

/// Test jq processes JSON correctly
#[test]
#[ignore = "requires jq in PATH"]
fn test_jq_processing() {
    if !command_exists("jq") {
        eprintln!("Skipping: jq not in PATH");
        return;
    }

    let output = Command::new("jq")
        .args(["-n", r#"{"test": "value"}"#])
        .output()
        .expect("Failed to execute jq");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).expect("Invalid JSON from jq");
    assert_eq!(parsed["test"], "value");
}

/// Test procs with JSON output
#[test]
#[ignore = "requires procs in PATH"]
fn test_procs_json_output() {
    if !command_exists("procs") {
        eprintln!("Skipping: procs not in PATH");
        return;
    }

    let output = Command::new("procs")
        .args(["--json"])
        .output()
        .expect("Failed to execute procs");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<Value, _> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "procs --json should produce valid JSON");
}

/// Test tokei with JSON output
#[test]
#[ignore = "requires tokei in PATH"]
fn test_tokei_json_output() {
    if !command_exists("tokei") {
        eprintln!("Skipping: tokei not in PATH");
        return;
    }

    let output = Command::new("tokei")
        .args(["--output", "json", "."])
        .output()
        .expect("Failed to execute tokei");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<Value, _> = serde_json::from_str(&stdout);
    assert!(
        parsed.is_ok(),
        "tokei --output json should produce valid JSON"
    );
}

/// Test duf with JSON output
#[test]
#[ignore = "requires duf in PATH"]
fn test_duf_json_output() {
    if !command_exists("duf") {
        eprintln!("Skipping: duf not in PATH");
        return;
    }

    let output = Command::new("duf")
        .args(["--json"])
        .output()
        .expect("Failed to execute duf");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<Value, _> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "duf --json should produce valid JSON");
}

/// Test bat execution (no JSON, but should work)
#[test]
#[ignore = "requires bat in PATH"]
fn test_bat_execution() {
    if !command_exists("bat") {
        eprintln!("Skipping: bat not in PATH");
        return;
    }

    let output = Command::new("bat")
        .args(["--color=never", "--paging=never", "Cargo.toml"])
        .output()
        .expect("Failed to execute bat");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[package]"),
        "bat should output Cargo.toml content"
    );
}

/// Test fzf filter mode
#[test]
#[ignore = "requires fzf in PATH"]
fn test_fzf_filter() {
    if !command_exists("fzf") {
        eprintln!("Skipping: fzf not in PATH");
        return;
    }

    let mut child = Command::new("fzf")
        .args(["--filter", "main"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn fzf");

    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(b"main.rs\nlib.rs\nmod.rs\n")
            .expect("Failed to write");
    }

    let output = child.wait_with_output().expect("Failed to wait");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("main.rs"), "fzf should match main.rs");
}

/// Test file command execution
#[test]
#[ignore = "requires file in PATH"]
fn test_file_command() {
    if !command_exists("file") {
        eprintln!("Skipping: file not in PATH");
        return;
    }

    let output = Command::new("file")
        .args(["-b", "Cargo.toml"])
        .output()
        .expect("Failed to execute file");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("text"),
        "file should identify Cargo.toml as text"
    );
}

/// Test diff command execution
#[test]
#[ignore = "requires diff in PATH"]
fn test_diff_command() {
    if !command_exists("diff") {
        eprintln!("Skipping: diff not in PATH");
        return;
    }

    // Create temp files for diff
    let temp_dir = std::env::temp_dir();
    let file_a = temp_dir.join("diff_test_a.txt");
    let file_b = temp_dir.join("diff_test_b.txt");

    std::fs::write(&file_a, "line1\nline2\n").expect("Failed to write file_a");
    std::fs::write(&file_b, "line1\nmodified\n").expect("Failed to write file_b");

    let output = Command::new("diff")
        .args(["-u", file_a.to_str().unwrap(), file_b.to_str().unwrap()])
        .output()
        .expect("Failed to execute diff");

    // diff returns 1 when files differ, which is expected
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("@@"),
        "diff should produce unified diff output"
    );
    assert!(stdout.contains("-line2"), "diff should show removed line");
    assert!(stdout.contains("+modified"), "diff should show added line");

    // Cleanup
    let _ = std::fs::remove_file(file_a);
    let _ = std::fs::remove_file(file_b);
}
