// modern-cli-mcp/src/state.rs
//! Internal state management using SQLite for operational data.
//! Handles auth state, caching, tasks, and context storage.

use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// State manager for MCP operational data
#[derive(Debug, Clone)]
pub struct StateManager {
    conn: Arc<Mutex<Connection>>,
}

/// Authentication state for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub provider: String,
    pub authenticated: bool,
    pub last_check: i64,
    pub metadata: Option<serde_json::Value>,
}

/// Cached value with TTL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub value: String,
    pub created_at: i64,
    pub ttl_secs: Option<i64>,
}

/// Task item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub content: String,
    pub status: TaskStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::InProgress => write!(f, "in_progress"),
            TaskStatus::Completed => write!(f, "completed"),
        }
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(TaskStatus::Pending),
            "in_progress" => Ok(TaskStatus::InProgress),
            "completed" => Ok(TaskStatus::Completed),
            _ => Err(format!("Unknown status: {}", s)),
        }
    }
}

/// Context scope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContextScope {
    Session,
    Project,
    Global,
}

impl std::fmt::Display for ContextScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextScope::Session => write!(f, "session"),
            ContextScope::Project => write!(f, "project"),
            ContextScope::Global => write!(f, "global"),
        }
    }
}

impl std::str::FromStr for ContextScope {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "session" => Ok(ContextScope::Session),
            "project" => Ok(ContextScope::Project),
            "global" => Ok(ContextScope::Global),
            _ => Err(format!("Unknown scope: {}", s)),
        }
    }
}

/// Context entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    pub key: String,
    pub value: String,
    pub scope: ContextScope,
}

impl StateManager {
    /// Create a new state manager, initializing the database
    pub fn new() -> Result<Self, String> {
        let db_path = Self::get_db_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create state directory: {}", e))?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open state database: {}", e))?;

        let manager = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        manager.init_schema()?;
        Ok(manager)
    }

    /// Create an in-memory state manager (for testing)
    #[allow(dead_code)]
    pub fn new_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to open in-memory database: {}", e))?;

        let manager = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        manager.init_schema()?;
        Ok(manager)
    }

    fn get_db_path() -> Result<PathBuf, String> {
        let data_dir = dirs::data_dir()
            .or_else(dirs::home_dir)
            .ok_or_else(|| "Could not determine data directory".to_string())?;

        Ok(data_dir.join("modern-cli-mcp").join("state.db"))
    }

    fn init_schema(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute_batch(
            r#"
            -- Auth state for git forges and other services
            CREATE TABLE IF NOT EXISTS auth_state (
                provider TEXT PRIMARY KEY,
                authenticated INTEGER NOT NULL DEFAULT 0,
                last_check INTEGER NOT NULL,
                metadata TEXT
            );

            -- Tool cache with TTL
            CREATE TABLE IF NOT EXISTS tool_cache (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                ttl_secs INTEGER
            );

            -- Session tasks
            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            -- Key-value context storage
            CREATE TABLE IF NOT EXISTS context (
                key TEXT NOT NULL,
                scope TEXT NOT NULL DEFAULT 'session',
                value TEXT NOT NULL,
                PRIMARY KEY (key, scope)
            );

            -- Index for cache cleanup
            CREATE INDEX IF NOT EXISTS idx_cache_expiry
                ON tool_cache(created_at, ttl_secs);

            -- Index for task status queries
            CREATE INDEX IF NOT EXISTS idx_task_status
                ON tasks(status);
            "#,
        )
        .map_err(|e| format!("Failed to initialize schema: {}", e))?;

        Ok(())
    }

    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    // ========================================================================
    // AUTH STATE
    // ========================================================================

    /// Get auth state for a provider
    pub fn get_auth_state(&self, provider: &str) -> Result<Option<AuthState>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT provider, authenticated, last_check, metadata FROM auth_state WHERE provider = ?")
            .map_err(|e| e.to_string())?;

        let result = stmt
            .query_row(params![provider], |row| {
                let metadata_str: Option<String> = row.get(3)?;
                let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

                Ok(AuthState {
                    provider: row.get(0)?,
                    authenticated: row.get::<_, i64>(1)? != 0,
                    last_check: row.get(2)?,
                    metadata,
                })
            })
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(result)
    }

    /// Set auth state for a provider
    pub fn set_auth_state(&self, state: &AuthState) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let metadata_str = state
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default());

        conn.execute(
            "INSERT OR REPLACE INTO auth_state (provider, authenticated, last_check, metadata) VALUES (?, ?, ?, ?)",
            params![
                state.provider,
                state.authenticated as i64,
                state.last_check,
                metadata_str
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Get all auth states
    pub fn get_all_auth_states(&self) -> Result<Vec<AuthState>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT provider, authenticated, last_check, metadata FROM auth_state")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let metadata_str: Option<String> = row.get(3)?;
                let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

                Ok(AuthState {
                    provider: row.get(0)?,
                    authenticated: row.get::<_, i64>(1)? != 0,
                    last_check: row.get(2)?,
                    metadata,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<SqliteResult<Vec<_>>>()
            .map_err(|e| e.to_string())
    }

    // ========================================================================
    // CACHE
    // ========================================================================

    /// Get cached value (returns None if expired)
    pub fn cache_get(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = Self::now();

        let mut stmt = conn
            .prepare("SELECT value, created_at, ttl_secs FROM tool_cache WHERE key = ?")
            .map_err(|e| e.to_string())?;

        let result: Option<(String, i64, Option<i64>)> = stmt
            .query_row(params![key], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .optional()
            .map_err(|e| e.to_string())?;

        match result {
            Some((value, created_at, ttl_secs)) => {
                if let Some(ttl) = ttl_secs {
                    if now > created_at + ttl {
                        // Expired, delete and return None
                        drop(stmt);
                        conn.execute("DELETE FROM tool_cache WHERE key = ?", params![key])
                            .ok();
                        return Ok(None);
                    }
                }
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set cached value with optional TTL
    pub fn cache_set(&self, key: &str, value: &str, ttl_secs: Option<i64>) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT OR REPLACE INTO tool_cache (key, value, created_at, ttl_secs) VALUES (?, ?, ?, ?)",
            params![key, value, Self::now(), ttl_secs],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Delete cached value
    pub fn cache_delete(&self, key: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM tool_cache WHERE key = ?", params![key])
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Clean up expired cache entries
    pub fn cache_cleanup(&self) -> Result<u64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = Self::now();

        let deleted = conn
            .execute(
                "DELETE FROM tool_cache WHERE ttl_secs IS NOT NULL AND created_at + ttl_secs < ?",
                params![now],
            )
            .map_err(|e| e.to_string())?;

        Ok(deleted as u64)
    }

    // ========================================================================
    // TASKS
    // ========================================================================

    /// Create a new task
    pub fn task_create(&self, content: &str) -> Result<Task, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = Self::now();

        conn.execute(
            "INSERT INTO tasks (content, status, created_at, updated_at) VALUES (?, 'pending', ?, ?)",
            params![content, now, now],
        )
        .map_err(|e| e.to_string())?;

        let id = conn.last_insert_rowid();

        Ok(Task {
            id,
            content: content.to_string(),
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update task status
    pub fn task_update_status(&self, id: i64, status: TaskStatus) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let affected = conn
            .execute(
                "UPDATE tasks SET status = ?, updated_at = ? WHERE id = ?",
                params![status.to_string(), Self::now(), id],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("Task {} not found", id));
        }

        Ok(())
    }

    /// Get all tasks
    pub fn task_list(&self, status_filter: Option<TaskStatus>) -> Result<Vec<Task>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let (query, status_str);
        let params: Vec<&dyn rusqlite::ToSql> = if let Some(ref status) = status_filter {
            status_str = status.to_string();
            query = "SELECT id, content, status, created_at, updated_at FROM tasks WHERE status = ? ORDER BY id";
            vec![&status_str as &dyn rusqlite::ToSql]
        } else {
            query = "SELECT id, content, status, created_at, updated_at FROM tasks ORDER BY id";
            vec![]
        };

        let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params.as_slice(), |row| {
                let status_str: String = row.get(2)?;
                Ok(Task {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    status: status_str.parse().unwrap_or(TaskStatus::Pending),
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<SqliteResult<Vec<_>>>()
            .map_err(|e| e.to_string())
    }

    /// Delete a task
    pub fn task_delete(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let affected = conn
            .execute("DELETE FROM tasks WHERE id = ?", params![id])
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("Task {} not found", id));
        }

        Ok(())
    }

    /// Clear all tasks
    pub fn task_clear(&self) -> Result<u64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let deleted = conn
            .execute("DELETE FROM tasks", [])
            .map_err(|e| e.to_string())?;

        Ok(deleted as u64)
    }

    // ========================================================================
    // CONTEXT
    // ========================================================================

    /// Get context value
    pub fn context_get(&self, key: &str, scope: ContextScope) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let result: Option<String> = conn
            .query_row(
                "SELECT value FROM context WHERE key = ? AND scope = ?",
                params![key, scope.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(result)
    }

    /// Set context value
    pub fn context_set(&self, key: &str, value: &str, scope: ContextScope) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT OR REPLACE INTO context (key, scope, value) VALUES (?, ?, ?)",
            params![key, scope.to_string(), value],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Delete context value
    pub fn context_delete(&self, key: &str, scope: ContextScope) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "DELETE FROM context WHERE key = ? AND scope = ?",
            params![key, scope.to_string()],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// List all context entries for a scope
    pub fn context_list(&self, scope: Option<ContextScope>) -> Result<Vec<ContextEntry>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let (query, scope_str);
        let params: Vec<&dyn rusqlite::ToSql> = if let Some(ref s) = scope {
            scope_str = s.to_string();
            query = "SELECT key, scope, value FROM context WHERE scope = ?";
            vec![&scope_str as &dyn rusqlite::ToSql]
        } else {
            query = "SELECT key, scope, value FROM context";
            vec![]
        };

        let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params.as_slice(), |row| {
                let scope_str: String = row.get(1)?;
                Ok(ContextEntry {
                    key: row.get(0)?,
                    scope: scope_str.parse().unwrap_or(ContextScope::Session),
                    value: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<SqliteResult<Vec<_>>>()
            .map_err(|e| e.to_string())
    }

    /// Clear session-scoped context
    pub fn context_clear_session(&self) -> Result<u64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let deleted = conn
            .execute("DELETE FROM context WHERE scope = 'session'", [])
            .map_err(|e| e.to_string())?;

        Ok(deleted as u64)
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new().expect("Failed to create state manager")
    }
}

// Trait for optional query results
trait OptionalExt<T> {
    fn optional(self) -> SqliteResult<Option<T>>;
}

impl<T> OptionalExt<T> for SqliteResult<T> {
    fn optional(self) -> SqliteResult<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_state() {
        let mgr = StateManager::new_in_memory().unwrap();

        let state = AuthState {
            provider: "gh:github.com".to_string(),
            authenticated: true,
            last_check: StateManager::now(),
            metadata: Some(serde_json::json!({"user": "test"})),
        };

        mgr.set_auth_state(&state).unwrap();

        let retrieved = mgr.get_auth_state("gh:github.com").unwrap().unwrap();
        assert_eq!(retrieved.authenticated, true);
        assert_eq!(retrieved.provider, "gh:github.com");
    }

    #[test]
    fn test_cache() {
        let mgr = StateManager::new_in_memory().unwrap();

        mgr.cache_set("test_key", "test_value", Some(3600)).unwrap();

        let value = mgr.cache_get("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        mgr.cache_delete("test_key").unwrap();
        let value = mgr.cache_get("test_key").unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn test_tasks() {
        let mgr = StateManager::new_in_memory().unwrap();

        let task = mgr.task_create("Test task").unwrap();
        assert_eq!(task.content, "Test task");
        assert_eq!(task.status, TaskStatus::Pending);

        mgr.task_update_status(task.id, TaskStatus::InProgress)
            .unwrap();

        let tasks = mgr.task_list(Some(TaskStatus::InProgress)).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::InProgress);
    }

    #[test]
    fn test_context() {
        let mgr = StateManager::new_in_memory().unwrap();

        mgr.context_set("key1", "value1", ContextScope::Session)
            .unwrap();
        mgr.context_set("key2", "value2", ContextScope::Project)
            .unwrap();

        let value = mgr.context_get("key1", ContextScope::Session).unwrap();
        assert_eq!(value, Some("value1".to_string()));

        let entries = mgr.context_list(Some(ContextScope::Session)).unwrap();
        assert_eq!(entries.len(), 1);

        mgr.context_clear_session().unwrap();
        let value = mgr.context_get("key1", ContextScope::Session).unwrap();
        assert!(value.is_none());
    }
}
