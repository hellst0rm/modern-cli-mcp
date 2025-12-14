// src/ignore.rs
//! .agentignore pattern matching and path validation
//!
//! Loads ignore patterns from:
//! 1. ~/.config/agent/ignore (global)
//! 2. Walk up directory tree looking for .agentignore files
//!
//! Tools should NOT respect .gitignore, ONLY .agentignore.

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Compiled ignore patterns with caching
#[derive(Debug)]
pub struct AgentIgnore {
    /// Global ignore patterns (~/.config/agent/ignore)
    global: Option<Gitignore>,
    /// Per-directory cache of compiled patterns
    cache: RwLock<HashMap<PathBuf, Arc<Gitignore>>>,
}

impl AgentIgnore {
    /// Create new AgentIgnore, loading global patterns
    pub fn new() -> Result<Self, String> {
        let global = Self::load_global_ignore()?;
        Ok(Self {
            global,
            cache: RwLock::new(HashMap::new()),
        })
    }

    /// Load ~/.config/agent/ignore if exists
    fn load_global_ignore() -> Result<Option<Gitignore>, String> {
        let config_dir = match dirs::config_dir() {
            Some(dir) => dir,
            None => return Ok(None),
        };

        let ignore_path = config_dir.join("agent").join("ignore");

        if ignore_path.exists() {
            let mut builder = GitignoreBuilder::new(&config_dir);
            if let Some(err) = builder.add(&ignore_path) {
                return Err(format!("Failed to parse global ignore file: {}", err));
            }
            let gitignore = builder.build().map_err(|e| e.to_string())?;
            Ok(Some(gitignore))
        } else {
            Ok(None)
        }
    }

    /// Check if path should be ignored
    pub fn is_ignored(&self, path: &Path) -> bool {
        let path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };

        let is_dir = path.is_dir();

        // Check global ignore first
        if let Some(ref global) = self.global {
            if global.matched(&path, is_dir).is_ignore() {
                return true;
            }
        }

        // Walk up directory tree looking for .agentignore
        let mut current = path.parent();
        while let Some(dir) = current {
            let ignore_file = dir.join(".agentignore");
            if ignore_file.exists() {
                let patterns = self.get_or_load_patterns(dir);
                if let Some(patterns) = patterns {
                    if patterns.matched(&path, is_dir).is_ignore() {
                        return true;
                    }
                }
            }
            current = dir.parent();
        }

        false
    }

    /// Get ignore file paths and flags for use with fd/rg
    /// Returns args like: ["--no-ignore", "--ignore-file=/path/.agentignore", ...]
    pub fn get_ignore_file_args(&self, working_dir: &Path) -> Vec<String> {
        let mut args = Vec::new();

        // First, disable .gitignore processing
        args.push("--no-ignore".to_string());

        // Add global ignore file
        if let Some(config_dir) = dirs::config_dir() {
            let global_ignore = config_dir.join("agent").join("ignore");
            if global_ignore.exists() {
                args.push(format!("--ignore-file={}", global_ignore.display()));
            }
        }

        // Walk up from working_dir adding .agentignore files
        let working_dir = match working_dir.canonicalize() {
            Ok(p) => p,
            Err(_) => working_dir.to_path_buf(),
        };

        let mut current = Some(working_dir.as_path());
        while let Some(dir) = current {
            let ignore_file = dir.join(".agentignore");
            if ignore_file.exists() {
                args.push(format!("--ignore-file={}", ignore_file.display()));
            }
            current = dir.parent();
        }

        args
    }

    /// Filter a list of paths, removing ignored ones
    #[allow(dead_code)]
    pub fn filter_paths<P: AsRef<Path>>(&self, paths: Vec<P>) -> Vec<P> {
        paths
            .into_iter()
            .filter(|p| !self.is_ignored(p.as_ref()))
            .collect()
    }

    /// Validate path is not ignored, return error if it is
    pub fn validate_path(&self, path: &Path) -> Result<(), String> {
        if self.is_ignored(path) {
            Err(format!(
                "Path is blocked by .agentignore: {}",
                path.display()
            ))
        } else {
            Ok(())
        }
    }

    /// Load and cache patterns for a directory's .agentignore
    fn get_or_load_patterns(&self, dir: &Path) -> Option<Arc<Gitignore>> {
        let dir_path = dir.to_path_buf();

        // Check cache first
        {
            let cache = self.cache.read();
            if let Some(patterns) = cache.get(&dir_path) {
                return Some(Arc::clone(patterns));
            }
        }

        // Load and cache
        let ignore_file = dir.join(".agentignore");
        if !ignore_file.exists() {
            return None;
        }

        let mut builder = GitignoreBuilder::new(dir);
        if builder.add(&ignore_file).is_some() {
            return None; // Parse error
        }

        match builder.build() {
            Ok(gitignore) => {
                let arc = Arc::new(gitignore);
                let mut cache = self.cache.write();
                cache.insert(dir_path, Arc::clone(&arc));
                Some(arc)
            }
            Err(_) => None,
        }
    }

    /// Clear the pattern cache (useful for testing or after file changes)
    #[allow(dead_code)]
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write();
        cache.clear();
    }
}

impl Default for AgentIgnore {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            global: None,
            cache: RwLock::new(HashMap::new()),
        })
    }
}

impl Clone for AgentIgnore {
    fn clone(&self) -> Self {
        // Create a new instance with fresh cache but same global patterns
        Self {
            global: self.global.clone(),
            cache: RwLock::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_no_ignore_files() {
        let ignore = AgentIgnore::default();
        let temp = TempDir::new().unwrap();
        let test_file = temp.path().join("test.txt");
        fs::write(&test_file, "content").unwrap();

        assert!(!ignore.is_ignored(&test_file));
    }

    #[test]
    fn test_agentignore_file() {
        let temp = TempDir::new().unwrap();

        // Create .agentignore
        let ignore_file = temp.path().join(".agentignore");
        fs::write(&ignore_file, "*.secret\nsecrets/\n").unwrap();

        // Create test files
        let normal_file = temp.path().join("normal.txt");
        let secret_file = temp.path().join("test.secret");
        fs::write(&normal_file, "content").unwrap();
        fs::write(&secret_file, "secret").unwrap();

        let ignore = AgentIgnore::default();

        assert!(!ignore.is_ignored(&normal_file));
        assert!(ignore.is_ignored(&secret_file));
    }

    #[test]
    fn test_nested_agentignore() {
        let temp = TempDir::new().unwrap();

        // Create nested structure
        let subdir = temp.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        // Root .agentignore
        fs::write(temp.path().join(".agentignore"), "*.root\n").unwrap();

        // Subdir .agentignore
        fs::write(subdir.join(".agentignore"), "*.sub\n").unwrap();

        let root_file = temp.path().join("test.root");
        let sub_file = subdir.join("test.sub");
        let normal_file = subdir.join("normal.txt");

        fs::write(&root_file, "").unwrap();
        fs::write(&sub_file, "").unwrap();
        fs::write(&normal_file, "").unwrap();

        let ignore = AgentIgnore::default();

        assert!(ignore.is_ignored(&root_file));
        assert!(ignore.is_ignored(&sub_file));
        assert!(!ignore.is_ignored(&normal_file));
    }

    #[test]
    fn test_get_ignore_file_args() {
        let temp = TempDir::new().unwrap();

        // Create .agentignore
        let ignore_file = temp.path().join(".agentignore");
        fs::write(&ignore_file, "*.txt\n").unwrap();

        let ignore = AgentIgnore::default();
        let args = ignore.get_ignore_file_args(temp.path());

        // Should contain --no-ignore
        assert!(args.iter().any(|a| a == "--no-ignore"));

        // Should contain the .agentignore path
        assert!(args
            .iter()
            .any(|a| a.contains(".agentignore") && a.starts_with("--ignore-file=")));
    }

    #[test]
    fn test_filter_paths() {
        let temp = TempDir::new().unwrap();

        // Create .agentignore
        fs::write(temp.path().join(".agentignore"), "*.ignored\n").unwrap();

        let file1 = temp.path().join("keep.txt");
        let file2 = temp.path().join("remove.ignored");
        fs::write(&file1, "").unwrap();
        fs::write(&file2, "").unwrap();

        let ignore = AgentIgnore::default();
        let paths = vec![file1.clone(), file2.clone()];
        let filtered = ignore.filter_paths(paths);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], file1);
    }

    #[test]
    fn test_validate_path() {
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join(".agentignore"), "blocked.txt\n").unwrap();

        let allowed = temp.path().join("allowed.txt");
        let blocked = temp.path().join("blocked.txt");
        fs::write(&allowed, "").unwrap();
        fs::write(&blocked, "").unwrap();

        let ignore = AgentIgnore::default();

        assert!(ignore.validate_path(&allowed).is_ok());
        assert!(ignore.validate_path(&blocked).is_err());
    }
}
