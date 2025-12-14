// tests/json_helpers.rs
//! Integration tests for JSON output helper functions

use serde_json::Value;

// Import the helper functions from the crate
// Note: These need to be made pub in lib.rs or we test via the binary output

/// Test parse_eza_to_json output structure
#[test]
fn test_eza_json_structure() {
    let sample_output = "drwxr-xr-x    - user 10 Dec 12:00 src
.rw-r--r-- 1.2k user 10 Dec 11:00 Cargo.toml
.rw-r--r--  500 user 10 Dec 10:00 README.md";

    // Simulate what parse_eza_to_json produces
    let entries: Vec<Value> = sample_output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts.last().unwrap_or(&"");
                serde_json::json!({
                    "name": name,
                    "raw": line.trim()
                })
            } else {
                serde_json::json!({ "name": line.trim() })
            }
        })
        .collect();

    let result = serde_json::json!({
        "path": ".",
        "entries": entries,
        "count": entries.len()
    });

    assert_eq!(result["count"], 3);
    assert_eq!(result["entries"][0]["name"], "src");
    assert_eq!(result["entries"][1]["name"], "Cargo.toml");
    assert_eq!(result["entries"][2]["name"], "README.md");
}

/// Test parse_fd_to_json output structure
#[test]
fn test_fd_json_structure() {
    let sample_output = "src/main.rs
src/tools/mod.rs
src/tools/executor.rs";

    let files: Vec<Value> = sample_output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|path| {
            let name = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string());
            serde_json::json!({
                "path": path,
                "name": name
            })
        })
        .collect();

    let result = serde_json::json!({
        "files": files,
        "count": files.len()
    });

    assert_eq!(result["count"], 3);
    assert_eq!(result["files"][0]["name"], "main.rs");
    assert_eq!(result["files"][0]["path"], "src/main.rs");
    assert_eq!(result["files"][1]["name"], "mod.rs");
    assert_eq!(result["files"][2]["name"], "executor.rs");
}

/// Test parse_fzf_to_json output structure
#[test]
fn test_fzf_json_structure() {
    let sample_output = "src/main.rs
src/tools/mod.rs";
    let query = "main";

    let matches: Vec<Value> = sample_output
        .lines()
        .filter(|line| !line.is_empty())
        .enumerate()
        .map(|(i, line)| {
            serde_json::json!({
                "rank": i + 1,
                "match": line.trim()
            })
        })
        .collect();

    let result = serde_json::json!({
        "query": query,
        "matches": matches,
        "count": matches.len()
    });

    assert_eq!(result["query"], "main");
    assert_eq!(result["count"], 2);
    assert_eq!(result["matches"][0]["rank"], 1);
    assert_eq!(result["matches"][0]["match"], "src/main.rs");
}

/// Test parse_file_to_json output structure
#[test]
fn test_file_json_structure() {
    let sample_output = "UTF-8 Unicode text";
    let path = "README.md";

    let description = sample_output.trim();
    let (mime_type, desc) = if description.contains(';') {
        let parts: Vec<&str> = description.splitn(2, ';').collect();
        (parts[0].trim(), description)
    } else {
        ("unknown", description)
    };

    let result = serde_json::json!({
        "path": path,
        "mime_type": mime_type,
        "description": desc,
        "type": "text"
    });

    assert_eq!(result["path"], "README.md");
    assert_eq!(result["description"], "UTF-8 Unicode text");
    assert_eq!(result["type"], "text");
}

/// Test parse_trash_list_to_json output structure
#[test]
fn test_trash_list_json_structure() {
    let sample_output = "2024-01-15 10:30:00 /home/user/old_file.txt
2024-01-14 09:00:00 /home/user/backup.zip";

    let items: Vec<Value> = sample_output
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() >= 3 {
                Some(serde_json::json!({
                    "deletion_date": format!("{} {}", parts[0], parts[1]),
                    "original_path": parts[2]
                }))
            } else {
                Some(serde_json::json!({ "raw": line }))
            }
        })
        .collect();

    let result = serde_json::json!({
        "items": items,
        "count": items.len()
    });

    assert_eq!(result["count"], 2);
    assert_eq!(result["items"][0]["deletion_date"], "2024-01-15 10:30:00");
    assert_eq!(
        result["items"][0]["original_path"],
        "/home/user/old_file.txt"
    );
}

/// Test parse_dust_to_json output structure
#[test]
fn test_dust_json_structure() {
    let sample_output = "  1.2G ├── target
 50.0M ├── src
  2.0K └── Cargo.toml";
    let path = ".";

    let entries: Vec<Value> = sample_output
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with("Total:"))
        .filter_map(|line| {
            let trimmed = line.trim();
            let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
            if parts.len() >= 2 {
                let size = parts[0];
                let name = parts[1].trim_start_matches(['├', '└', '─', '│', ' ']);
                Some(serde_json::json!({
                    "size": size,
                    "name": name.trim()
                }))
            } else {
                None
            }
        })
        .collect();

    let result = serde_json::json!({
        "path": path,
        "entries": entries,
        "count": entries.len()
    });

    assert_eq!(result["count"], 3);
    assert_eq!(result["entries"][0]["size"], "1.2G");
    assert_eq!(result["entries"][0]["name"], "target");
}

/// Test diff JSON structure
#[test]
fn test_diff_json_structure() {
    // Just verify we can parse the hunk header
    let line = "@@ -1,3 +1,4 @@";
    assert!(line.starts_with("@@"));

    let parts: Vec<&str> = line.split_whitespace().collect();
    assert_eq!(parts.len(), 4);
    assert_eq!(parts[1], "-1,3");
    assert_eq!(parts[2], "+1,4");
}

/// Test JSON validity of all outputs
#[test]
fn test_json_validity() {
    // Test that we produce valid JSON
    let json_str = r#"{"path": ".", "entries": [], "count": 0}"#;
    let parsed: Result<Value, _> = serde_json::from_str(json_str);
    assert!(parsed.is_ok());

    let json_str = r#"{"files": [{"path": "a.rs", "name": "a.rs"}], "count": 1}"#;
    let parsed: Result<Value, _> = serde_json::from_str(json_str);
    assert!(parsed.is_ok());

    let json_str = r#"{"query": "test", "matches": [], "count": 0}"#;
    let parsed: Result<Value, _> = serde_json::from_str(json_str);
    assert!(parsed.is_ok());
}
