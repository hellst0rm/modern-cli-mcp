// modern-cli-mcp/src/tools/mod.rs
mod executor;

pub use executor::{
    parse_diff_to_json, parse_dust_to_json, parse_eza_to_json, parse_fd_to_json,
    parse_file_to_json, parse_fzf_to_json, CommandExecutor, ExecOptions,
};

use crate::format;
use crate::groups::{AgentProfile, ToolGroup};
use crate::ignore::AgentIgnore;
use crate::state::{ContextScope, StateManager, TaskStatus};
use parking_lot::RwLock;
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::ToolCallContext, wrapper::Parameters},
    model::{
        CallToolRequestParam, CallToolResult, Content, ListToolsResult, PaginatedRequestParam,
        ServerCapabilities, ServerInfo, Tool,
    },
    schemars,
    service::RequestContext,
    tool, tool_router, ErrorData, RoleServer, ServerHandler,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Configuration for dynamic toolset mode
#[derive(Debug, Clone)]
pub struct DynamicToolsetConfig {
    /// Whether dynamic toolsets mode is enabled
    pub enabled: bool,
    /// Currently enabled tool groups
    pub enabled_groups: Arc<RwLock<HashSet<ToolGroup>>>,
}

impl Default for DynamicToolsetConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            enabled_groups: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModernCliTools {
    tool_router: ToolRouter<Self>,
    executor: CommandExecutor,
    state: Arc<StateManager>,
    profile: Option<AgentProfile>,
    ignore: Arc<AgentIgnore>,
    /// Dynamic toolset configuration (beta feature)
    dynamic_config: DynamicToolsetConfig,
    /// Reverse lookup: tool name -> group (for filtering)
    tool_to_group: HashMap<&'static str, ToolGroup>,
    /// Dual-response mode: return formatted summary + raw data
    dual_response: bool,
}

// ============================================================================
// REQUEST TYPES
// ============================================================================

// --- Filesystem ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EzaRequest {
    #[schemars(description = "Directory path to list (default: current directory)")]
    pub path: Option<String>,
    #[schemars(description = "Show all files including hidden")]
    pub all: Option<bool>,
    #[schemars(description = "Long format with details")]
    pub long: Option<bool>,
    #[schemars(description = "Tree view")]
    pub tree: Option<bool>,
    #[schemars(description = "Tree depth level")]
    pub level: Option<u32>,
    #[schemars(description = "Show git status")]
    pub git: Option<bool>,
    #[schemars(description = "Show icons")]
    pub icons: Option<bool>,
    #[schemars(description = "Sort by: name, size, time, ext, type")]
    pub sort: Option<String>,
    #[schemars(description = "Reverse sort order")]
    pub reverse: Option<bool>,
    #[schemars(description = "Only show directories")]
    pub dirs_only: Option<bool>,
    #[schemars(description = "Only show files")]
    pub files_only: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BatRequest {
    #[schemars(description = "File path to display")]
    pub path: String,
    #[schemars(description = "Language for syntax highlighting")]
    pub language: Option<String>,
    #[schemars(description = "Show line numbers")]
    pub number: Option<bool>,
    #[schemars(description = "Line range to show (e.g., '10:20' or ':50')")]
    pub range: Option<String>,
    #[schemars(description = "Highlight specific lines (e.g., '1,3,5-10')")]
    pub highlight: Option<String>,
    #[schemars(description = "Style: full, plain, numbers, grid, header")]
    pub style: Option<String>,
    #[schemars(description = "Show non-printable characters")]
    pub show_all: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FdRequest {
    #[schemars(description = "Search pattern (regex by default)")]
    pub pattern: Option<String>,
    #[schemars(description = "Directory to search in")]
    pub path: Option<String>,
    #[schemars(description = "File extension to filter")]
    pub extension: Option<String>,
    #[schemars(description = "Type: f(ile), d(irectory), l(ink), x(executable)")]
    pub file_type: Option<String>,
    #[schemars(description = "Include hidden files")]
    pub hidden: Option<bool>,
    #[allow(dead_code)] // Deprecated: use .agentignore instead
    #[schemars(description = "Don't respect .gitignore")]
    pub no_ignore: Option<bool>,
    #[schemars(description = "Maximum search depth")]
    pub max_depth: Option<u32>,
    #[schemars(description = "Minimum search depth")]
    pub min_depth: Option<u32>,
    #[schemars(description = "Case-insensitive search")]
    pub ignore_case: Option<bool>,
    #[schemars(description = "Exclude pattern")]
    pub exclude: Option<String>,
    #[schemars(description = "Follow symlinks")]
    pub follow: Option<bool>,
    #[schemars(description = "Show absolute paths")]
    pub absolute: Option<bool>,
    #[schemars(description = "Size filter (e.g., '+1M', '-100k')")]
    pub size: Option<String>,
    #[schemars(description = "Modification time filter (e.g., '-1d', '+1w')")]
    pub changed_within: Option<String>,
    #[schemars(description = "Maximum number of results to return")]
    pub max_results: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DufRequest {
    #[schemars(description = "Specific mount point to show")]
    pub path: Option<String>,
    #[schemars(description = "Show all filesystems including pseudo")]
    pub all: Option<bool>,
    #[schemars(description = "Show inodes")]
    pub inodes: Option<bool>,
    #[schemars(description = "Output format: json or default")]
    pub json: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DustRequest {
    #[schemars(description = "Directory to analyze")]
    pub path: Option<String>,
    #[schemars(description = "Number of items to show")]
    pub number: Option<u32>,
    #[schemars(description = "Maximum depth")]
    pub depth: Option<u32>,
    #[schemars(description = "Reverse order (smallest first)")]
    pub reverse: Option<bool>,
    #[schemars(description = "Only show directories")]
    pub only_dirs: Option<bool>,
    #[schemars(description = "Only show files")]
    pub only_files: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TrashRequest {
    #[schemars(
        description = "File or directory path to trash (supports multiple space-separated paths)"
    )]
    pub path: String,
    #[schemars(description = "Custom graveyard directory (default: ~/.graveyard)")]
    pub graveyard: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TrashListRequest {
    #[schemars(description = "Custom graveyard directory (default: ~/.graveyard)")]
    pub graveyard: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TrashRestoreRequest {
    #[schemars(description = "Restore last deleted item, or specify record index from seance")]
    pub target: Option<String>,
    #[schemars(description = "Custom graveyard directory (default: ~/.graveyard)")]
    pub graveyard: Option<String>,
}

// --- Grouped: Filesystem ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FilesystemGroupRequest {
    #[schemars(
        description = "Subcommand: list, view, find, disk_usage, dir_size, trash, trash_list, trash_restore, copy, move, mkdir, stat, exists, symlink, hardlink, file_type, permissions"
    )]
    pub command: String,

    // Target path (used by most subcommands)
    #[schemars(description = "Target path (file or directory)")]
    pub path: Option<String>,

    // list (eza) options
    #[schemars(description = "[list] Show hidden files")]
    pub all: Option<bool>,
    #[schemars(description = "[list] Long format with details")]
    pub long: Option<bool>,
    #[schemars(description = "[list] Tree view")]
    pub tree: Option<bool>,
    #[schemars(description = "[list] Tree depth level")]
    pub level: Option<u32>,
    #[schemars(description = "[list] Show git status")]
    pub git: Option<bool>,
    #[schemars(description = "[list] Show icons")]
    pub icons: Option<bool>,
    #[schemars(description = "[list] Sort by: name, size, time, ext, type")]
    pub sort: Option<String>,
    #[schemars(description = "[list] Reverse sort order")]
    pub reverse: Option<bool>,
    #[schemars(description = "[list] Only show directories")]
    pub dirs_only: Option<bool>,
    #[schemars(description = "[list] Only show files")]
    pub files_only: Option<bool>,

    // find (fd) options
    #[schemars(description = "[find] Search pattern (regex)")]
    pub pattern: Option<String>,
    #[schemars(description = "[find] File extension filter")]
    pub extension: Option<String>,
    #[schemars(description = "[find] Type: f(ile), d(irectory), l(ink), x(executable)")]
    pub file_type: Option<String>,
    #[schemars(description = "[find] Maximum search depth")]
    pub max_depth: Option<u32>,
    #[schemars(description = "[find] Minimum search depth")]
    pub min_depth: Option<u32>,
    #[schemars(description = "[find] Include hidden files")]
    pub hidden: Option<bool>,
    #[schemars(description = "[find] Don't respect .gitignore")]
    pub no_ignore: Option<bool>,
    #[schemars(description = "[find] Case-insensitive search")]
    pub ignore_case: Option<bool>,
    #[schemars(description = "[find] Exclude pattern")]
    pub exclude: Option<String>,
    #[schemars(description = "[find] Follow symlinks")]
    pub follow: Option<bool>,
    #[schemars(description = "[find] Show absolute paths")]
    pub absolute: Option<bool>,
    #[schemars(description = "[find] Size filter (e.g., '+1M', '-100k')")]
    pub size: Option<String>,
    #[schemars(description = "[find] Modification time filter (e.g., '-1d', '+1w')")]
    pub changed_within: Option<String>,

    // view (bat) options
    #[schemars(description = "[view] Line range (e.g., '10:20')")]
    pub range: Option<String>,
    #[schemars(description = "[view] Language for syntax highlighting")]
    pub language: Option<String>,
    #[schemars(description = "[view] Show line numbers")]
    pub number: Option<bool>,
    #[schemars(description = "[view] Highlight specific lines (e.g., '1,3,5-10')")]
    pub highlight: Option<String>,
    #[schemars(description = "[view] Style: full, plain, numbers, grid, header")]
    pub style: Option<String>,
    #[schemars(description = "[view] Show non-printable characters")]
    pub show_all: Option<bool>,

    // disk_usage (duf) options
    #[schemars(description = "[disk_usage] Show all filesystems including pseudo")]
    pub duf_all: Option<bool>,
    #[schemars(description = "[disk_usage] Show inodes")]
    pub inodes: Option<bool>,
    #[schemars(description = "[disk_usage/dir_size] JSON output")]
    pub json: Option<bool>,

    // dir_size (dust) options
    #[schemars(description = "[dir_size] Number of items to show")]
    pub dust_number: Option<u32>,
    #[schemars(description = "[dir_size] Maximum depth")]
    pub depth: Option<u32>,
    #[schemars(description = "[dir_size] Only show directories")]
    pub only_dirs: Option<bool>,
    #[schemars(description = "[dir_size] Only show files")]
    pub only_files: Option<bool>,

    // copy/move options
    #[schemars(description = "[copy/move] Destination path")]
    pub dest: Option<String>,
    #[schemars(description = "[copy] Copy directories recursively")]
    pub recursive: Option<bool>,
    #[schemars(
        description = "[copy/move/symlink/hardlink/trash] Backup dest to graveyard before overwriting"
    )]
    pub safe_overwrite: Option<bool>,
    #[schemars(description = "[trash/trash_list/trash_restore] Custom graveyard directory")]
    pub graveyard: Option<String>,

    // symlink/hardlink options
    #[schemars(description = "[symlink] Target path (what link points to)")]
    pub target: Option<String>,
    #[schemars(description = "[symlink/hardlink] Link path to create")]
    pub link: Option<String>,
    #[schemars(description = "[hardlink] Source path (existing file to link to)")]
    pub source: Option<String>,

    // mkdir options
    #[schemars(description = "[mkdir] Create parent directories (default: true)")]
    pub parents: Option<bool>,

    // trash_restore options
    #[schemars(description = "[trash_restore] Record index from seance to restore")]
    pub restore_target: Option<String>,

    // permissions options
    #[schemars(description = "[permissions] Mode to explain (e.g., '755', 'rwxr-xr-x')")]
    pub mode: Option<String>,
}

// ============================================================================
// GROUPED TOOL REQUEST STRUCTS
// ============================================================================

/// File operations grouped tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FileOpsGroupRequest {
    #[schemars(description = "Subcommand: read, write, edit, append, patch")]
    pub command: String,

    #[schemars(description = "File path")]
    pub path: String,

    // read options
    #[schemars(description = "[read] Starting line number (1-indexed)")]
    pub offset: Option<usize>,
    #[schemars(description = "[read] Number of lines to read")]
    pub limit: Option<usize>,

    // write/append options
    #[schemars(description = "[write/append] Content to write")]
    pub content: Option<String>,
    #[schemars(description = "[write] Create parent directories if needed")]
    pub create_dirs: Option<bool>,
    #[schemars(description = "[write] Backup existing file to graveyard before overwriting")]
    pub safe_overwrite: Option<bool>,
    #[schemars(description = "[write/edit/patch] Custom graveyard directory")]
    pub graveyard: Option<String>,

    // edit options
    #[schemars(description = "[edit] Text to find (must be unique in file)")]
    pub old_text: Option<String>,
    #[schemars(description = "[edit] Text to replace with")]
    pub new_text: Option<String>,
    #[schemars(description = "[edit] Replace all occurrences")]
    pub replace_all: Option<bool>,
    #[schemars(description = "[edit/patch] Backup file before modifying")]
    pub backup: Option<bool>,

    // patch options
    #[schemars(description = "[patch] Unified diff patch content")]
    pub patch: Option<String>,
}

/// Search grouped tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchGroupRequest {
    #[schemars(description = "Subcommand: grep, ast, symbols, references, fzf")]
    pub command: String,

    // Common
    #[schemars(description = "Search pattern")]
    pub pattern: Option<String>,
    #[schemars(description = "Path to search in")]
    pub path: Option<String>,

    // grep (ripgrep) options
    #[schemars(description = "[grep] Case-insensitive search")]
    pub ignore_case: Option<bool>,
    #[schemars(description = "[grep] Smart case")]
    pub smart_case: Option<bool>,
    #[schemars(description = "[grep] Search hidden files")]
    pub hidden: Option<bool>,
    #[schemars(description = "[grep] Don't respect .gitignore")]
    pub no_ignore: Option<bool>,
    #[schemars(description = "[grep] Only show matching filenames")]
    pub files_with_matches: Option<bool>,
    #[schemars(description = "[grep] Only show count of matches")]
    pub count: Option<bool>,
    #[schemars(description = "[grep] Show line numbers")]
    pub line_number: Option<bool>,
    #[schemars(description = "[grep] Lines of context before and after")]
    pub context: Option<u32>,
    #[schemars(description = "[grep] File type to search")]
    pub file_type: Option<String>,
    #[schemars(description = "[grep] Glob pattern to include")]
    pub glob: Option<String>,
    #[schemars(description = "[grep] Match whole words only")]
    pub word: Option<bool>,
    #[schemars(description = "[grep] Fixed string search (not regex)")]
    pub fixed_strings: Option<bool>,
    #[schemars(description = "[grep] Multiline mode")]
    pub multiline: Option<bool>,
    #[schemars(description = "[grep] Follow symlinks")]
    pub follow: Option<bool>,
    #[schemars(description = "[grep] JSON output")]
    pub json: Option<bool>,
    #[schemars(description = "[grep] Maximum number of results")]
    pub max_count: Option<u32>,
    #[schemars(description = "[grep] Invert match")]
    pub invert: Option<bool>,
    #[schemars(description = "[grep] Show only matches")]
    pub only_matching: Option<bool>,
    #[schemars(description = "[grep] Replace matches with this text")]
    pub replace: Option<String>,

    // ast (ast-grep) options
    #[schemars(description = "[ast] Language (rust, python, javascript, typescript, go)")]
    pub lang: Option<String>,
    #[schemars(description = "[ast] Replacement pattern")]
    pub rewrite: Option<String>,

    // symbols options
    #[schemars(description = "[symbols/references] Symbol name")]
    pub symbol: Option<String>,

    // references options
    #[schemars(description = "[references] Language for context-aware search")]
    pub language: Option<String>,

    // fzf options
    #[schemars(description = "[fzf] Input text to filter (newline-separated items)")]
    pub input: Option<String>,
    #[schemars(description = "[fzf] Filter query")]
    pub query: Option<String>,
    #[schemars(description = "[fzf] Exact match (no fuzzy)")]
    pub exact: Option<bool>,
    #[schemars(description = "[fzf] Number of results to return")]
    pub limit: Option<u32>,
}

/// Text processing grouped tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TextGroupRequest {
    #[schemars(description = "Subcommand: jq, yq, sd, htmlq, pup, miller, dasel, gron, hck, csv")]
    pub command: String,

    #[schemars(description = "Input text/data")]
    pub input: String,

    // jq options
    #[schemars(description = "[jq] jq filter expression")]
    pub filter: Option<String>,
    #[schemars(description = "[jq] Raw output (no JSON encoding for strings)")]
    pub raw: Option<bool>,
    #[schemars(description = "[jq] Compact output")]
    pub compact: Option<bool>,
    #[schemars(description = "[jq] Slurp mode")]
    pub slurp: Option<bool>,
    #[schemars(description = "[jq] Sort keys")]
    pub sort_keys: Option<bool>,

    // yq options
    #[schemars(description = "[yq] yq expression")]
    pub expression: Option<String>,
    #[schemars(description = "[yq/dasel/miller] Input format: yaml, json, xml, csv, toml")]
    pub input_format: Option<String>,
    #[schemars(description = "[yq/dasel/miller] Output format")]
    pub output_format: Option<String>,
    #[schemars(description = "[yq] Pretty print output")]
    pub prettyprint: Option<bool>,

    // sd options
    #[schemars(description = "[sd] Pattern to find")]
    pub find: Option<String>,
    #[schemars(description = "[sd] Replacement string")]
    pub replace: Option<String>,
    #[schemars(description = "[sd] Fixed string mode (no regex)")]
    pub fixed: Option<bool>,

    // htmlq/pup options
    #[schemars(description = "[htmlq/pup] CSS selector")]
    pub selector: Option<String>,
    #[schemars(description = "[htmlq] Extract attribute value")]
    pub attribute: Option<String>,
    #[schemars(description = "[htmlq/pup] Extract text content only")]
    pub text: Option<bool>,
    #[schemars(description = "[pup] Output as JSON")]
    pub json: Option<bool>,

    // miller options
    #[schemars(
        description = "[miller] Miller verb: cat, cut, head, tail, sort, filter, put, stats1, uniq, join"
    )]
    pub verb: Option<String>,
    #[schemars(description = "[miller/csv] Additional arguments")]
    pub args: Option<String>,

    // dasel options
    #[schemars(description = "[dasel] Selector query")]
    pub dasel_selector: Option<String>,
    #[schemars(description = "[dasel] Compact output")]
    pub dasel_compact: Option<bool>,

    // gron options
    #[schemars(description = "[gron] Ungron mode (convert back to JSON)")]
    pub ungron: Option<bool>,
    #[schemars(description = "[gron] Stream mode")]
    pub stream: Option<bool>,

    // hck options
    #[schemars(description = "[hck] Fields to extract (e.g., '1,3', '1-3', '2-')")]
    pub fields: Option<String>,
    #[schemars(description = "[hck/csv] Input delimiter")]
    pub delimiter: Option<String>,
    #[schemars(description = "[hck] Output delimiter")]
    pub output_delimiter: Option<String>,

    // csv (xsv) options
    #[schemars(
        description = "[csv] xsv subcommand: stats, select, search, sort, slice, frequency, count, headers"
    )]
    pub csv_command: Option<String>,
    #[schemars(description = "[csv] No header row")]
    pub no_headers: Option<bool>,
}

/// Git grouped tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitGroupRequest {
    #[schemars(
        description = "Subcommand: status, add, commit, branch, checkout, log, diff, stash"
    )]
    pub command: String,

    #[schemars(
        description = "Git repository path (runs git -C <path>). Defaults to current directory."
    )]
    pub path: Option<String>,

    // status options
    #[schemars(description = "[status] Short format output")]
    pub short: Option<bool>,
    #[schemars(description = "[status] Machine-readable porcelain format")]
    pub porcelain: Option<bool>,

    // add options
    #[schemars(description = "[add] Files to add (space-separated paths, or '.' for all)")]
    pub files: Option<String>,
    #[schemars(description = "[add] Add all changes including untracked")]
    pub all: Option<bool>,

    // commit options
    #[schemars(description = "[commit] Commit message")]
    pub message: Option<String>,
    #[schemars(description = "[commit] Amend previous commit")]
    pub amend: Option<bool>,
    #[schemars(description = "[commit] Automatically stage modified/deleted files")]
    pub commit_all: Option<bool>,

    // branch options
    #[schemars(description = "[branch] Branch subcommand: list, create, delete, rename")]
    pub branch_command: Option<String>,
    #[schemars(description = "[branch/checkout] Branch name")]
    pub name: Option<String>,
    #[schemars(description = "[branch] New name for rename operation")]
    pub new_name: Option<String>,
    #[schemars(description = "[branch] Force operation")]
    pub force: Option<bool>,

    // checkout options
    #[schemars(description = "[checkout] Branch, commit, or tag to checkout")]
    pub target: Option<String>,
    #[schemars(description = "[checkout] Create new branch (-b flag)")]
    pub create: Option<bool>,
    #[schemars(description = "[checkout] Specific files to checkout")]
    pub checkout_files: Option<String>,

    // log options
    #[schemars(description = "[log] Number of commits to show")]
    pub count: Option<u32>,
    #[schemars(description = "[log] Show history for specific file")]
    pub file: Option<String>,
    #[schemars(description = "[log] One line per commit")]
    pub oneline: Option<bool>,
    #[schemars(description = "[log] Custom format string")]
    pub format: Option<String>,

    // diff options
    #[schemars(description = "[diff] Show staged changes")]
    pub staged: Option<bool>,
    #[schemars(description = "[diff] Compare with specific commit")]
    pub commit: Option<String>,
    #[schemars(description = "[diff] Compare between two commits (commit1..commit2)")]
    pub range: Option<String>,

    // stash options
    #[schemars(description = "[stash] Stash subcommand: push, pop, list, drop, apply, show")]
    pub stash_command: Option<String>,
    #[schemars(description = "[stash] Stash message (for push)")]
    pub stash_message: Option<String>,
    #[schemars(description = "[stash] Stash index")]
    pub index: Option<u32>,
}

/// GitHub grouped tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitHubGroupRequest {
    #[schemars(
        description = "Subcommand: repo, issue, pr, search, release, workflow, run, api, auth_status, auth_login"
    )]
    pub command: String,

    #[schemars(description = "Repository in owner/repo format")]
    pub repo: Option<String>,

    // Common subcommand
    #[schemars(
        description = "[issue/pr/repo/release/workflow/run] Sub-subcommand: list, view, create, close, etc."
    )]
    pub subcommand: Option<String>,

    // issue/pr options
    #[schemars(description = "[issue/pr] Issue/PR number")]
    pub number: Option<u32>,
    #[schemars(description = "[issue/pr] Title")]
    pub title: Option<String>,
    #[schemars(description = "[issue/pr/release] Body/notes")]
    pub body: Option<String>,
    #[schemars(description = "[issue/pr] State filter: open, closed, all")]
    pub state: Option<String>,
    #[schemars(description = "[issue/pr/search/run] Maximum results")]
    pub limit: Option<u32>,
    #[schemars(description = "[issue] Labels (comma-separated)")]
    pub labels: Option<String>,
    #[schemars(description = "[issue] Assignees (comma-separated)")]
    pub assignees: Option<String>,

    // pr options
    #[schemars(description = "[pr] Base branch")]
    pub base: Option<String>,
    #[schemars(description = "[pr] Head branch")]
    pub head: Option<String>,
    #[schemars(description = "[pr] Merge method: merge, squash, rebase")]
    pub merge_method: Option<String>,

    // search options
    #[schemars(description = "[search] Search type: repos, issues, prs, code, commits")]
    pub search_type: Option<String>,
    #[schemars(description = "[search] Search query")]
    pub query: Option<String>,

    // release options
    #[schemars(description = "[release] Release tag")]
    pub tag: Option<String>,
    #[schemars(description = "[release] Mark as draft")]
    pub draft: Option<bool>,
    #[schemars(description = "[release] Mark as prerelease")]
    pub prerelease: Option<bool>,
    #[schemars(description = "[release] Release notes")]
    pub notes: Option<String>,

    // workflow/run options
    #[schemars(description = "[workflow/run] Workflow ID or filename")]
    pub workflow: Option<String>,
    #[schemars(description = "[workflow] Branch to run workflow on")]
    pub ref_branch: Option<String>,
    #[schemars(description = "[run] Run ID")]
    pub run_id: Option<u64>,
    #[schemars(description = "[run] Status filter: queued, in_progress, completed")]
    pub status: Option<String>,

    // api options
    #[schemars(description = "[api] API endpoint")]
    pub endpoint: Option<String>,
    #[schemars(description = "[api] HTTP method: GET, POST, PUT, PATCH, DELETE")]
    pub method: Option<String>,
    #[schemars(description = "[api] jq filter for response")]
    pub jq_filter: Option<String>,

    // auth options
    #[schemars(description = "[auth_status/auth_login] Hostname")]
    pub hostname: Option<String>,
    #[schemars(description = "[auth_login] Authentication token")]
    pub token: Option<String>,

    // repo options
    #[schemars(description = "[repo] Additional arguments")]
    pub args: Option<String>,
}

/// GitLab grouped tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitLabGroupRequest {
    #[schemars(description = "Subcommand: issue, mr, pipeline, auth_status, auth_login")]
    pub command: String,

    #[schemars(description = "Project path (group/project)")]
    pub project: Option<String>,

    #[schemars(
        description = "[issue/mr/pipeline] Sub-subcommand: list, view, create, close, etc."
    )]
    pub subcommand: Option<String>,

    // issue/mr options
    #[schemars(description = "[issue/mr] Issue/MR IID")]
    pub iid: Option<u32>,
    #[schemars(description = "[issue/mr] Title")]
    pub title: Option<String>,
    #[schemars(description = "[issue/mr] Description")]
    pub description: Option<String>,
    #[schemars(description = "[issue/mr] State filter: opened, closed, merged, all")]
    pub state: Option<String>,
    #[schemars(description = "[issue/mr/pipeline] Maximum results")]
    pub per_page: Option<u32>,
    #[schemars(description = "[issue] Labels (comma-separated)")]
    pub labels: Option<String>,

    // mr options
    #[schemars(description = "[mr] Source branch")]
    pub source_branch: Option<String>,
    #[schemars(description = "[mr] Target branch")]
    pub target_branch: Option<String>,

    // pipeline options
    #[schemars(description = "[pipeline] Pipeline ID")]
    pub pipeline_id: Option<u64>,
    #[schemars(description = "[pipeline] Branch/ref to run pipeline on")]
    pub ref_name: Option<String>,
    #[schemars(description = "[pipeline] Status filter: running, pending, success, failed")]
    pub status: Option<String>,

    // auth options
    #[schemars(description = "[auth_status/auth_login] Hostname")]
    pub hostname: Option<String>,
    #[schemars(description = "[auth_login] Authentication token")]
    pub token: Option<String>,
}

/// Kubernetes grouped tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KubernetesGroupRequest {
    #[schemars(
        description = "Subcommand: get, describe, logs, apply, delete, exec, stern, helm, kustomize"
    )]
    pub command: String,

    #[schemars(description = "Namespace")]
    pub namespace: Option<String>,

    // get/describe/delete options
    #[schemars(
        description = "[get/describe/delete] Resource type: pods, deployments, services, etc."
    )]
    pub resource: Option<String>,
    #[schemars(description = "[get/describe/delete/logs/exec] Resource name")]
    pub name: Option<String>,
    #[schemars(description = "[get] Label selector")]
    pub selector: Option<String>,
    #[schemars(description = "[get] All namespaces")]
    pub all_namespaces: Option<bool>,
    #[schemars(description = "[get] Output format: json, yaml, wide, name")]
    pub output: Option<String>,

    // logs options
    #[schemars(description = "[logs/exec/stern] Container name")]
    pub container: Option<String>,
    #[schemars(description = "[logs/stern] Number of lines to show")]
    pub tail: Option<u32>,
    #[schemars(description = "[logs/stern] Show logs since duration")]
    pub since: Option<String>,
    #[schemars(description = "[logs] Show previous container logs")]
    pub previous: Option<bool>,
    #[schemars(description = "[logs/stern] Include timestamps")]
    pub timestamps: Option<bool>,

    // apply options
    #[schemars(description = "[apply] YAML/JSON manifest content")]
    pub manifest: Option<String>,
    #[schemars(description = "[apply] Dry run mode: none, client, server")]
    pub dry_run: Option<String>,

    // delete options
    #[schemars(description = "[delete] Force deletion")]
    pub force: Option<bool>,

    // exec options
    #[schemars(description = "[exec] Command to execute in pod")]
    pub exec_command: Option<String>,

    // stern options
    #[schemars(description = "[stern] Pod query (regex or exact match)")]
    pub query: Option<String>,
    #[schemars(description = "[stern] Output format: default, raw, json")]
    pub stern_output: Option<String>,

    // helm options
    #[schemars(
        description = "[helm] Helm subcommand: list, status, get, install, upgrade, uninstall, search, show, repo"
    )]
    pub helm_command: Option<String>,
    #[schemars(description = "[helm] Release name")]
    pub release: Option<String>,
    #[schemars(description = "[helm] Chart reference")]
    pub chart: Option<String>,
    #[schemars(description = "[helm] Values file path or inline YAML")]
    pub values: Option<String>,
    #[schemars(description = "[helm] Additional arguments")]
    pub args: Option<String>,

    // kustomize options
    #[schemars(description = "[kustomize] Kustomize subcommand: build, edit, create")]
    pub kustomize_command: Option<String>,
    #[schemars(description = "[kustomize] Path to kustomization directory")]
    pub kustomize_path: Option<String>,
    #[schemars(description = "[kustomize] Output format: yaml, json")]
    pub kustomize_output: Option<String>,
}

/// Container grouped tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ContainerGroupRequest {
    #[schemars(description = "Subcommand: podman, dive, skopeo, crane, trivy")]
    pub command: String,

    // podman options
    #[schemars(
        description = "[podman] Podman subcommand: ps, images, inspect, logs, pull, run, stop, rm, rmi, build"
    )]
    pub podman_command: Option<String>,
    #[schemars(description = "[podman/dive/skopeo/crane/trivy] Container/image name or ID")]
    pub target: Option<String>,
    #[schemars(description = "[podman] Show all containers/images")]
    pub all: Option<bool>,
    #[schemars(description = "[podman] Additional arguments")]
    pub args: Option<String>,

    // dive options
    #[schemars(description = "[dive] Image to analyze")]
    pub image: Option<String>,
    #[schemars(description = "[dive] CI mode - check efficiency")]
    pub ci: Option<bool>,
    #[schemars(description = "[dive/trivy] Export JSON report")]
    pub json: Option<bool>,

    // skopeo options
    #[schemars(description = "[skopeo] Skopeo subcommand: inspect, copy, delete, list-tags, sync")]
    pub skopeo_command: Option<String>,
    #[schemars(description = "[skopeo] Source image reference")]
    pub source: Option<String>,
    #[schemars(description = "[skopeo] Destination image reference")]
    pub dest: Option<String>,
    #[schemars(description = "[skopeo] Don't verify TLS")]
    pub insecure: Option<bool>,

    // crane options
    #[schemars(
        description = "[crane] Crane subcommand: digest, manifest, config, ls, pull, push, copy, tag"
    )]
    pub crane_command: Option<String>,

    // trivy options
    #[schemars(description = "[trivy] Scan type: image, fs, repo, config")]
    pub scan_type: Option<String>,
    #[schemars(description = "[trivy] Output format: table, json, sarif")]
    pub format: Option<String>,
    #[schemars(description = "[trivy] Severity filter: UNKNOWN,LOW,MEDIUM,HIGH,CRITICAL")]
    pub severity: Option<String>,
    #[schemars(description = "[trivy] Ignore unfixed vulnerabilities")]
    pub ignore_unfixed: Option<bool>,
}

/// Network grouped tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NetworkGroupRequest {
    #[schemars(description = "Subcommand: http, sql, dns")]
    pub command: String,

    // http (xh) options
    #[schemars(description = "[http] URL to request")]
    pub url: Option<String>,
    #[schemars(description = "[http] HTTP method: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS")]
    pub method: Option<String>,
    #[schemars(description = "[http] Request body (JSON)")]
    pub body: Option<String>,
    #[schemars(description = "[http] Headers as JSON object")]
    pub headers: Option<String>,
    #[schemars(description = "[http] Basic auth (user:pass)")]
    pub auth: Option<String>,
    #[schemars(description = "[http] Bearer token")]
    pub bearer: Option<String>,
    #[schemars(description = "[http] Follow redirects")]
    pub follow: Option<bool>,
    #[schemars(description = "[http] Request timeout in seconds")]
    pub timeout: Option<u32>,
    #[schemars(description = "[http] Form data instead of JSON")]
    pub form: Option<bool>,
    #[schemars(description = "[http] Output JSON only")]
    pub json_output: Option<bool>,
    #[schemars(description = "[http] Print mode: all, headers, body")]
    pub print: Option<String>,

    // sql (usql) options
    #[schemars(description = "[sql] Database URL")]
    pub db_url: Option<String>,
    #[schemars(description = "[sql] SQL command to execute")]
    pub sql_command: Option<String>,
    #[schemars(description = "[sql] Output format: csv, json, table")]
    pub format: Option<String>,

    // dns (doggo) options
    #[schemars(description = "[dns] Domain to query")]
    pub domain: Option<String>,
    #[schemars(description = "[dns] Record type: A, AAAA, MX, NS, TXT, CNAME, SOA, etc.")]
    pub record_type: Option<String>,
    #[schemars(description = "[dns] DNS server to use")]
    pub server: Option<String>,
    #[schemars(description = "[dns] Short output")]
    pub short: Option<bool>,
    #[schemars(description = "[dns] JSON output")]
    pub json: Option<bool>,
}

/// System grouped tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SystemGroupRequest {
    #[schemars(description = "Subcommand: shell, nix_shell, benchmark, procs, info, bats")]
    pub command: String,

    // shell options
    #[schemars(description = "[shell/nix_shell] Command to execute")]
    pub exec_command: Option<String>,
    #[schemars(description = "[shell] Shell: bash, zsh, fish, nu, dash")]
    pub shell: Option<String>,
    #[schemars(description = "[shell/nix_shell] Working directory")]
    pub working_dir: Option<String>,
    #[schemars(description = "[shell/nix_shell] Timeout in seconds")]
    pub timeout: Option<u64>,
    #[schemars(description = "[shell] Environment variables as JSON object")]
    pub env: Option<String>,

    // nix_shell options
    #[schemars(description = "[nix_shell] Flake reference")]
    pub flake: Option<String>,
    #[schemars(description = "[nix_shell] Devshell attribute name")]
    pub devshell: Option<String>,

    // benchmark options
    #[schemars(description = "[benchmark] Command to benchmark")]
    pub benchmark_command: Option<String>,
    #[schemars(description = "[benchmark] Compare with another command")]
    pub compare: Option<String>,
    #[schemars(description = "[benchmark] Number of warmup runs")]
    pub warmup: Option<u32>,
    #[schemars(description = "[benchmark] Minimum number of runs")]
    pub min_runs: Option<u32>,
    #[schemars(description = "[benchmark] Export results as JSON")]
    pub json: Option<bool>,

    // procs options
    #[schemars(description = "[procs] Filter processes by keyword")]
    pub keyword: Option<String>,
    #[schemars(description = "[procs] Show tree view")]
    pub tree: Option<bool>,
    #[schemars(description = "[procs] Sort column: cpu, mem, pid, user, etc.")]
    pub sort: Option<String>,

    // bats options
    #[schemars(description = "[bats] Test file or directory path")]
    pub path: Option<String>,
    #[schemars(description = "[bats] Filter tests by name pattern")]
    pub filter: Option<String>,
    #[schemars(description = "[bats] TAP output format")]
    pub tap: Option<bool>,
    #[schemars(description = "[bats] Count test cases")]
    pub count: Option<bool>,
}

/// Archive grouped tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ArchiveGroupRequest {
    #[schemars(description = "Subcommand: compress, decompress, list")]
    pub command: String,

    #[schemars(description = "Archive file path")]
    pub archive: Option<String>,

    // compress options
    #[schemars(description = "[compress] Files to compress (comma-separated paths)")]
    pub files: Option<String>,
    #[schemars(description = "[compress] Output archive path")]
    pub output: Option<String>,

    // decompress options
    #[schemars(description = "[decompress] Output directory")]
    pub output_dir: Option<String>,
}

/// Reference grouped tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReferenceGroupRequest {
    #[schemars(description = "Subcommand: tldr, cheat, regex")]
    pub command: String,

    // tldr options
    #[schemars(description = "[tldr/cheat] Command name to get help for")]
    pub cmd: Option<String>,
    #[schemars(description = "[tldr] Platform: linux, osx, windows, common")]
    pub platform: Option<String>,

    // cheat (navi) options
    #[schemars(description = "[cheat] Query to search cheats")]
    pub query: Option<String>,
    #[schemars(description = "[cheat] Show best match without prompting")]
    pub best_match: Option<bool>,

    // regex (grex) options
    #[schemars(description = "[regex] Test strings that the regex should match")]
    pub input: Option<String>,
    #[schemars(description = "[regex] Enable case-insensitive matching")]
    pub ignore_case: Option<bool>,
    #[schemars(description = "[regex] Escape special characters")]
    pub escape: Option<bool>,
    #[schemars(description = "[regex] Use anchors (^ and $)")]
    pub anchors: Option<bool>,
    #[schemars(description = "[regex] Use verbose mode with comments")]
    pub verbose: Option<bool>,
    #[schemars(description = "[regex] Convert to non-capturing groups")]
    pub no_capture: Option<bool>,
}

/// Diff grouped tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DiffGroupRequest {
    #[schemars(description = "Subcommand: files, structural")]
    pub command: String,

    #[schemars(description = "First file path")]
    pub file_a: Option<String>,
    #[schemars(description = "Second file path")]
    pub file_b: Option<String>,

    // structural (difftastic) options
    #[schemars(description = "[structural] Display mode: side-by-side, inline")]
    pub display: Option<String>,
    #[schemars(description = "[structural] Override language detection")]
    pub language: Option<String>,
    #[schemars(description = "[structural] Context lines around changes")]
    pub context: Option<u32>,
}

/// MCP state grouped tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct McpGroupRequest {
    #[schemars(
        description = "Subcommand: cache_get, cache_set, task_create, task_update, task_list, task_delete, context_get, context_set, context_list, auth_check"
    )]
    pub command: String,

    // cache options
    #[schemars(description = "[cache_get/cache_set/context_get/context_set/context_delete] Key")]
    pub key: Option<String>,
    #[schemars(description = "[cache_set/context_set] Value to store")]
    pub value: Option<String>,
    #[schemars(description = "[cache_set] Time-to-live in seconds")]
    pub ttl_secs: Option<i64>,

    // task options
    #[schemars(description = "[task_create] Task description")]
    pub content: Option<String>,
    #[schemars(description = "[task_update/task_delete] Task ID")]
    pub id: Option<i64>,
    #[schemars(description = "[task_update] New status: pending, in_progress, completed")]
    pub status: Option<String>,

    // context options
    #[schemars(
        description = "[context_get/context_set/context_list] Scope: session, project, global"
    )]
    pub scope: Option<String>,
}

// --- Search ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RipgrepRequest {
    #[schemars(description = "Search pattern (regex)")]
    pub pattern: String,
    #[schemars(description = "Path to search in")]
    pub path: Option<String>,
    #[schemars(description = "Case-insensitive search")]
    pub ignore_case: Option<bool>,
    #[schemars(description = "Smart case (case-insensitive unless pattern has uppercase)")]
    pub smart_case: Option<bool>,
    #[schemars(description = "Search hidden files")]
    pub hidden: Option<bool>,
    #[allow(dead_code)] // Deprecated: use .agentignore instead
    #[schemars(description = "Don't respect .gitignore")]
    pub no_ignore: Option<bool>,
    #[schemars(description = "Only show matching filenames")]
    pub files_with_matches: Option<bool>,
    #[schemars(description = "Only show count of matches")]
    pub count: Option<bool>,
    #[schemars(description = "Show line numbers")]
    pub line_number: Option<bool>,
    #[schemars(description = "Lines of context before and after")]
    pub context: Option<u32>,
    #[schemars(description = "File type to search (e.g., 'rust', 'py', 'js')")]
    pub file_type: Option<String>,
    #[schemars(description = "Glob pattern to include")]
    pub glob: Option<String>,
    #[schemars(description = "Match whole words only")]
    pub word: Option<bool>,
    #[schemars(description = "Invert match (show non-matching lines)")]
    pub invert: Option<bool>,
    #[schemars(description = "Maximum matches per file")]
    pub max_count: Option<u32>,
    #[schemars(description = "Maximum total results to return")]
    pub max_results: Option<u32>,
    #[schemars(description = "Follow symlinks")]
    pub follow: Option<bool>,
    #[schemars(description = "Multiline mode")]
    pub multiline: Option<bool>,
    #[schemars(description = "Show only matches (not full lines)")]
    pub only_matching: Option<bool>,
    #[schemars(description = "Replace matches with this text")]
    pub replace: Option<String>,
    #[schemars(description = "Fixed string search (not regex)")]
    pub fixed_strings: Option<bool>,
    #[schemars(description = "JSON output")]
    pub json: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FzfFilterRequest {
    #[schemars(description = "Input text to filter (newline-separated items)")]
    pub input: String,
    #[schemars(description = "Filter query")]
    pub query: String,
    #[schemars(description = "Exact match (no fuzzy)")]
    pub exact: Option<bool>,
    #[schemars(description = "Case-insensitive match")]
    pub ignore_case: Option<bool>,
    #[schemars(description = "Number of results to return")]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AstGrepRequest {
    #[schemars(description = "AST pattern to search for")]
    pub pattern: String,
    #[schemars(description = "Path to search in")]
    pub path: Option<String>,
    #[schemars(description = "Language (rust, python, javascript, typescript, go, etc.)")]
    pub lang: Option<String>,
    #[schemars(description = "Replacement pattern")]
    pub rewrite: Option<String>,
}

// --- Text Processing ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SdRequest {
    #[schemars(description = "Pattern to find (regex)")]
    pub find: String,
    #[schemars(description = "Replacement string")]
    pub replace: String,
    #[schemars(description = "Input text to transform")]
    pub input: String,
    #[schemars(description = "Fixed string mode (no regex)")]
    pub fixed: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JqRequest {
    #[schemars(description = "jq filter expression")]
    pub filter: String,
    #[schemars(description = "JSON input")]
    pub input: String,
    #[schemars(description = "Raw output (no JSON encoding for strings)")]
    pub raw: Option<bool>,
    #[schemars(description = "Compact output (no pretty printing)")]
    pub compact: Option<bool>,
    #[schemars(description = "Sort keys in objects")]
    pub sort_keys: Option<bool>,
    #[schemars(description = "Slurp mode (read all inputs into array)")]
    pub slurp: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct YqRequest {
    #[schemars(description = "yq expression")]
    pub expression: String,
    #[schemars(description = "YAML input")]
    pub input: String,
    #[schemars(description = "Output format: yaml, json, props, xml")]
    pub output_format: Option<String>,
    #[schemars(description = "Input format: yaml, json, props, xml, csv, tsv")]
    pub input_format: Option<String>,
    #[schemars(description = "Prettyprint output")]
    pub prettyprint: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct XsvRequest {
    #[schemars(
        description = "qsv subcommand: stats, select, search, sort, slice, frequency, count, headers"
    )]
    pub command: String,
    #[schemars(description = "CSV data")]
    pub input: String,
    #[schemars(description = "Additional arguments for the command")]
    pub args: Option<String>,
    #[schemars(description = "Delimiter character")]
    pub delimiter: Option<String>,
    #[schemars(description = "No header row")]
    pub no_headers: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HckRequest {
    #[schemars(description = "Input text")]
    pub input: String,
    #[schemars(description = "Fields to extract (e.g., '1,3', '1-3', '2-')")]
    pub fields: String,
    #[schemars(description = "Input delimiter (default: whitespace)")]
    pub delimiter: Option<String>,
    #[schemars(description = "Output delimiter")]
    pub output_delimiter: Option<String>,
}

// --- System ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProcsRequest {
    #[schemars(description = "Filter processes by keyword")]
    pub keyword: Option<String>,
    #[schemars(description = "Sort column: cpu, mem, pid, user, etc.")]
    pub sort: Option<String>,
    #[schemars(description = "Show tree view")]
    pub tree: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TokeiRequest {
    #[schemars(description = "Path to analyze")]
    pub path: Option<String>,
    #[schemars(description = "Output format: json, yaml")]
    pub output: Option<String>,
    #[schemars(description = "Sort by: lines, code, comments, blanks, files")]
    pub sort: Option<String>,
    #[schemars(description = "Show files")]
    pub files: Option<bool>,
    #[schemars(description = "Exclude patterns")]
    pub exclude: Option<String>,
    #[schemars(description = "Only show specific languages (comma-separated)")]
    pub languages: Option<String>,
    #[schemars(description = "Show hidden files")]
    pub hidden: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HyperfineRequest {
    #[schemars(description = "Command to benchmark")]
    pub command: String,
    #[schemars(description = "Number of warmup runs")]
    pub warmup: Option<u32>,
    #[schemars(description = "Minimum number of runs")]
    pub min_runs: Option<u32>,
    #[schemars(description = "Export results as JSON")]
    pub json: Option<bool>,
    #[schemars(description = "Compare with another command")]
    pub compare: Option<String>,
}

// --- Network ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct HttpRequest {
    #[schemars(description = "HTTP method: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS")]
    pub method: Option<String>,
    #[schemars(description = "URL to request")]
    pub url: String,
    #[schemars(description = "Request body (JSON)")]
    pub body: Option<String>,
    #[schemars(description = "Headers as JSON object")]
    pub headers: Option<String>,
    #[schemars(description = "Basic auth (user:pass)")]
    pub auth: Option<String>,
    #[schemars(description = "Bearer token")]
    pub bearer: Option<String>,
    #[schemars(description = "Follow redirects")]
    pub follow: Option<bool>,
    #[schemars(description = "Request timeout in seconds")]
    pub timeout: Option<u32>,
    #[schemars(description = "Print mode: all, headers, body")]
    pub print: Option<String>,
    #[schemars(description = "Output JSON only")]
    pub json_output: Option<bool>,
    #[schemars(description = "Form data instead of JSON")]
    pub form: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DnsRequest {
    #[schemars(description = "Domain to query")]
    pub domain: String,
    #[schemars(description = "Record type: A, AAAA, MX, NS, TXT, CNAME, SOA, etc.")]
    pub record_type: Option<String>,
    #[schemars(description = "DNS server to use (e.g., 8.8.8.8, 1.1.1.1)")]
    pub server: Option<String>,
    #[schemars(description = "Short output")]
    pub short: Option<bool>,
    #[schemars(description = "JSON output")]
    pub json: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UsqlRequest {
    #[schemars(
        description = "Database URL (e.g., postgres://user:pass@host/db, mysql://, sqlite:)"
    )]
    pub url: String,
    #[schemars(description = "SQL command to execute")]
    pub command: Option<String>,
    #[schemars(description = "Output format: csv, json, table")]
    pub format: Option<String>,
}

// --- Web Search ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WebSearchRequest {
    #[schemars(description = "Search query keywords")]
    pub query: String,
    #[schemars(description = "Number of results to return (0-25, default 10)")]
    pub num_results: Option<u32>,
    #[schemars(description = "Region for search (e.g., 'us-en', 'uk-en', 'de-de')")]
    pub region: Option<String>,
    #[schemars(description = "Time limit: d (day), w (week), m (month), y (year)")]
    pub time: Option<String>,
    #[schemars(description = "Limit search to a specific site")]
    pub site: Option<String>,
    #[schemars(description = "Show expanded URLs")]
    pub expand_urls: Option<bool>,
}

// --- Utility ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeltaRequest {
    #[schemars(description = "First file path")]
    pub file_a: String,
    #[schemars(description = "Second file path")]
    pub file_b: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitDiffRequest {
    #[schemars(description = "Git repository path")]
    pub path: Option<String>,
    #[schemars(description = "Show staged changes")]
    pub staged: Option<bool>,
    #[schemars(description = "Compare with specific commit")]
    pub commit: Option<String>,
    #[schemars(description = "Compare between two commits (commit1..commit2)")]
    pub range: Option<String>,
    #[schemars(description = "Specific file to diff")]
    pub file: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BatsRequest {
    #[schemars(description = "Test file or directory")]
    pub path: String,
    #[schemars(description = "TAP output format")]
    pub tap: Option<bool>,
    #[schemars(description = "Count test cases")]
    pub count: Option<bool>,
    #[schemars(description = "Filter tests by name pattern")]
    pub filter: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FileTypeRequest {
    #[schemars(description = "File path to analyze")]
    pub path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PermissionsRequest {
    #[schemars(description = "Permission mode (e.g., 755, rwxr-xr-x)")]
    pub mode: String,
}

// --- New AI-helpful tools ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TldrRequest {
    #[schemars(description = "Command name to get help for")]
    pub command: String,
    #[schemars(description = "Platform: linux, osx, windows, common")]
    pub platform: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GrexRequest {
    #[schemars(
        description = "Test strings that the regex should match (one per line or comma-separated)"
    )]
    pub input: String,
    #[schemars(description = "Escape special characters")]
    pub escape: Option<bool>,
    #[schemars(description = "Enable case-insensitive matching")]
    pub ignore_case: Option<bool>,
    #[schemars(description = "Use verbose mode with comments")]
    pub verbose: Option<bool>,
    #[schemars(description = "Convert to non-capturing groups")]
    pub no_capture: Option<bool>,
    #[schemars(description = "Use anchors (^ and $)")]
    pub anchors: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SadRequest {
    #[schemars(description = "Pattern to search for (regex)")]
    pub pattern: String,
    #[schemars(description = "Replacement string")]
    pub replace: String,
    #[schemars(description = "Files or glob patterns to process")]
    pub files: String,
    #[schemars(description = "Preview changes without applying")]
    pub preview: Option<bool>,
    #[schemars(description = "Fixed string mode (no regex)")]
    pub fixed: Option<bool>,
    #[schemars(description = "Case insensitive")]
    pub ignore_case: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DifftasticRequest {
    #[schemars(description = "First file path")]
    pub left: String,
    #[schemars(description = "Second file path")]
    pub right: String,
    #[schemars(description = "Display mode: side-by-side, inline")]
    pub display: Option<String>,
    #[schemars(description = "Override language detection")]
    pub language: Option<String>,
    #[schemars(description = "Context lines around changes")]
    pub context: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OuchCompressRequest {
    #[schemars(description = "Files to compress (comma-separated paths)")]
    pub files: String,
    #[schemars(
        description = "Output archive path (extension determines format: .tar.gz, .zip, .7z, etc.)"
    )]
    pub output: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OuchDecompressRequest {
    #[schemars(description = "Archive file to decompress")]
    pub archive: String,
    #[schemars(description = "Output directory")]
    pub output_dir: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OuchListRequest {
    #[schemars(description = "Archive file to list contents")]
    pub archive: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PueueAddRequest {
    #[schemars(description = "Command to add to queue")]
    pub command: String,
    #[schemars(description = "Start task immediately")]
    pub immediate: Option<bool>,
    #[schemars(description = "Add task in paused state")]
    pub stashed: Option<bool>,
    #[schemars(description = "Task label")]
    pub label: Option<String>,
    #[schemars(description = "Working directory")]
    pub working_dir: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PueueStatusRequest {
    #[schemars(description = "Show only specific group")]
    pub group: Option<String>,
    #[schemars(description = "JSON output")]
    pub json: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PueueLogRequest {
    #[schemars(description = "Task ID to get logs for")]
    pub task_id: u32,
    #[schemars(description = "Show full output")]
    pub full: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NaviRequest {
    #[schemars(description = "Query to search cheats")]
    pub query: Option<String>,
    #[schemars(description = "Show best match without prompting")]
    pub best_match: Option<bool>,
}

// ============================================================================
// GIT FORGE TOOLS
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GhRepoRequest {
    #[schemars(description = "Subcommand: list, view, clone, create, fork, delete")]
    pub command: String,
    #[schemars(description = "Repository in owner/repo format")]
    pub repo: Option<String>,
    #[schemars(description = "Additional arguments")]
    pub args: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GhIssueRequest {
    #[schemars(description = "Subcommand: list, view, create, close, reopen, edit, comment")]
    pub command: String,
    #[schemars(description = "Repository in owner/repo format (uses current repo if omitted)")]
    pub repo: Option<String>,
    #[schemars(description = "Issue number (for view, close, reopen, edit, comment)")]
    pub number: Option<u32>,
    #[schemars(description = "Issue title (for create)")]
    pub title: Option<String>,
    #[schemars(description = "Issue body (for create, edit, comment)")]
    pub body: Option<String>,
    #[schemars(description = "Labels (comma-separated)")]
    pub labels: Option<String>,
    #[schemars(description = "Assignees (comma-separated)")]
    pub assignees: Option<String>,
    #[schemars(description = "State filter: open, closed, all (for list)")]
    pub state: Option<String>,
    #[schemars(description = "Maximum results (for list)")]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GhPrRequest {
    #[schemars(
        description = "Subcommand: list, view, create, close, reopen, merge, checkout, diff, checks"
    )]
    pub command: String,
    #[schemars(description = "Repository in owner/repo format")]
    pub repo: Option<String>,
    #[schemars(description = "PR number")]
    pub number: Option<u32>,
    #[schemars(description = "PR title (for create)")]
    pub title: Option<String>,
    #[schemars(description = "PR body (for create)")]
    pub body: Option<String>,
    #[schemars(description = "Base branch (for create)")]
    pub base: Option<String>,
    #[schemars(description = "Head branch (for create)")]
    pub head: Option<String>,
    #[schemars(description = "State filter: open, closed, merged, all")]
    pub state: Option<String>,
    #[schemars(description = "Maximum results")]
    pub limit: Option<u32>,
    #[schemars(description = "Merge method: merge, squash, rebase")]
    pub merge_method: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GhSearchRequest {
    #[schemars(description = "Search type: repos, issues, prs, code, commits")]
    pub search_type: String,
    #[schemars(description = "Search query")]
    pub query: String,
    #[schemars(description = "Maximum results")]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GhReleaseRequest {
    #[schemars(description = "Subcommand: list, view, create, delete, download")]
    pub command: String,
    #[schemars(description = "Repository in owner/repo format")]
    pub repo: Option<String>,
    #[schemars(description = "Release tag")]
    pub tag: Option<String>,
    #[schemars(description = "Release title")]
    pub title: Option<String>,
    #[schemars(description = "Release notes")]
    pub notes: Option<String>,
    #[schemars(description = "Mark as draft")]
    pub draft: Option<bool>,
    #[schemars(description = "Mark as prerelease")]
    pub prerelease: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GhWorkflowRequest {
    #[schemars(description = "Subcommand: list, view, run, disable, enable")]
    pub command: String,
    #[schemars(description = "Repository in owner/repo format")]
    pub repo: Option<String>,
    #[schemars(description = "Workflow ID or filename")]
    pub workflow: Option<String>,
    #[schemars(description = "Branch to run workflow on")]
    pub ref_branch: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GhRunRequest {
    #[schemars(description = "Subcommand: list, view, watch, download, rerun, cancel")]
    pub command: String,
    #[schemars(description = "Repository in owner/repo format")]
    pub repo: Option<String>,
    #[schemars(description = "Run ID")]
    pub run_id: Option<u64>,
    #[schemars(description = "Workflow filter")]
    pub workflow: Option<String>,
    #[schemars(description = "Status filter: queued, in_progress, completed")]
    pub status: Option<String>,
    #[schemars(description = "Maximum results")]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GhApiRequest {
    #[schemars(description = "API endpoint (e.g., /repos/{owner}/{repo})")]
    pub endpoint: String,
    #[schemars(description = "HTTP method: GET, POST, PUT, PATCH, DELETE")]
    pub method: Option<String>,
    #[schemars(description = "Request body as JSON")]
    pub body: Option<String>,
    #[schemars(description = "jq filter for response")]
    pub jq_filter: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GlabIssueRequest {
    #[schemars(description = "Subcommand: list, view, create, close, reopen")]
    pub command: String,
    #[schemars(description = "Project path (group/project)")]
    pub project: Option<String>,
    #[schemars(description = "Issue IID")]
    pub iid: Option<u32>,
    #[schemars(description = "Issue title")]
    pub title: Option<String>,
    #[schemars(description = "Issue description")]
    pub description: Option<String>,
    #[schemars(description = "Labels (comma-separated)")]
    pub labels: Option<String>,
    #[schemars(description = "State filter: opened, closed, all")]
    pub state: Option<String>,
    #[schemars(description = "Maximum results")]
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GlabMrRequest {
    #[schemars(description = "Subcommand: list, view, create, close, reopen, merge, approve")]
    pub command: String,
    #[schemars(description = "Project path")]
    pub project: Option<String>,
    #[schemars(description = "MR IID")]
    pub iid: Option<u32>,
    #[schemars(description = "MR title")]
    pub title: Option<String>,
    #[schemars(description = "MR description")]
    pub description: Option<String>,
    #[schemars(description = "Source branch")]
    pub source_branch: Option<String>,
    #[schemars(description = "Target branch")]
    pub target_branch: Option<String>,
    #[schemars(description = "State filter: opened, closed, merged, all")]
    pub state: Option<String>,
    #[schemars(description = "Maximum results")]
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GlabPipelineRequest {
    #[schemars(description = "Subcommand: list, view, run, cancel, retry, delete")]
    pub command: String,
    #[schemars(description = "Project path")]
    pub project: Option<String>,
    #[schemars(description = "Pipeline ID")]
    pub pipeline_id: Option<u64>,
    #[schemars(description = "Branch/ref to run pipeline on")]
    pub ref_name: Option<String>,
    #[schemars(description = "Status filter: running, pending, success, failed")]
    pub status: Option<String>,
}

// ============================================================================
// DATA TRANSFORMATION TOOLS
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GronRequest {
    #[schemars(description = "JSON input to transform")]
    pub input: String,
    #[schemars(description = "Ungron mode (convert back to JSON)")]
    pub ungron: Option<bool>,
    #[schemars(description = "Stream mode for large inputs")]
    pub stream: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HtmlqRequest {
    #[schemars(description = "HTML input")]
    pub input: String,
    #[schemars(description = "CSS selector")]
    pub selector: String,
    #[schemars(description = "Extract attribute value")]
    pub attribute: Option<String>,
    #[schemars(description = "Extract text content only")]
    pub text: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PupRequest {
    #[schemars(description = "HTML input")]
    pub input: String,
    #[schemars(description = "CSS selector with optional display filter (e.g., 'a attr{href}')")]
    pub selector: String,
    #[schemars(description = "Output as JSON")]
    pub json: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MillerRequest {
    #[schemars(
        description = "Miller verb: cat, cut, head, tail, sort, filter, put, stats1, uniq, join"
    )]
    pub verb: String,
    #[schemars(description = "Input data")]
    pub input: String,
    #[schemars(description = "Input format: json, csv, dkvp, nidx, pprint, xtab")]
    pub input_format: Option<String>,
    #[schemars(description = "Output format: json, csv, dkvp, pprint, markdown")]
    pub output_format: Option<String>,
    #[schemars(description = "Additional arguments (e.g., '-f field1,field2' for cut)")]
    pub args: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DaselRequest {
    #[schemars(description = "Selector query (e.g., '.users.[0].name')")]
    pub selector: String,
    #[schemars(description = "Input data")]
    pub input: String,
    #[schemars(description = "Input format: json, yaml, toml, xml, csv")]
    pub input_format: Option<String>,
    #[schemars(description = "Output format: json, yaml, toml, xml, csv, plain")]
    pub output_format: Option<String>,
    #[schemars(description = "Compact output (no pretty print)")]
    pub compact: Option<bool>,
}

// ============================================================================
// CONTAINER TOOLS
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PodmanRequest {
    #[schemars(
        description = "Podman subcommand: ps, images, inspect, logs, pull, run, stop, rm, rmi, build"
    )]
    pub command: String,
    #[schemars(description = "Container/image name or ID")]
    pub target: Option<String>,
    #[schemars(description = "Additional arguments")]
    pub args: Option<String>,
    #[schemars(description = "Show all (for ps, images)")]
    pub all: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DiveRequest {
    #[schemars(description = "Image to analyze")]
    pub image: String,
    #[schemars(description = "CI mode - check efficiency")]
    pub ci: Option<bool>,
    #[schemars(description = "Export JSON report")]
    pub json: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SkopeoRequest {
    #[schemars(description = "Subcommand: inspect, copy, delete, list-tags, sync")]
    pub command: String,
    #[schemars(description = "Source image reference (e.g., docker://alpine:latest)")]
    pub source: String,
    #[schemars(description = "Destination image reference (for copy, sync)")]
    pub dest: Option<String>,
    #[schemars(description = "Don't verify TLS")]
    pub insecure: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CraneRequest {
    #[schemars(description = "Subcommand: digest, manifest, config, ls, pull, push, copy, tag")]
    pub command: String,
    #[schemars(description = "Image reference")]
    pub image: String,
    #[schemars(description = "Additional arguments")]
    pub args: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TrivyRequest {
    #[schemars(description = "Scan type: image, fs, repo, config")]
    pub scan_type: String,
    #[schemars(description = "Target to scan (image name, path, or repo URL)")]
    pub target: String,
    #[schemars(description = "Severity filter: UNKNOWN,LOW,MEDIUM,HIGH,CRITICAL")]
    pub severity: Option<String>,
    #[schemars(description = "Output format: table, json, sarif")]
    pub format: Option<String>,
    #[schemars(description = "Ignore unfixed vulnerabilities")]
    pub ignore_unfixed: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ComposeRequest {
    #[schemars(
        description = "Compose subcommand: up, down, ps, logs, build, pull, restart, stop, start"
    )]
    pub command: String,
    #[schemars(description = "Container runtime: podman (default, rootless) or docker")]
    pub runtime: Option<String>,
    #[schemars(description = "Path to compose file (default: docker-compose.yml)")]
    pub file: Option<String>,
    #[schemars(description = "Service name(s) to target (space-separated)")]
    pub services: Option<String>,
    #[schemars(description = "Run in detached mode (for up)")]
    pub detach: Option<bool>,
    #[schemars(description = "Remove volumes (for down)")]
    pub volumes: Option<bool>,
    #[schemars(description = "Follow log output (for logs)")]
    pub follow: Option<bool>,
    #[schemars(description = "Number of lines to show from end of logs")]
    pub tail: Option<u32>,
    #[schemars(description = "Additional arguments")]
    pub args: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BuildxRequest {
    #[schemars(description = "Buildx subcommand: build, imagetools, inspect, ls, create, use, rm")]
    pub command: String,
    #[schemars(description = "Build context path or image reference")]
    pub target: Option<String>,
    #[schemars(description = "Target platforms (e.g., linux/amd64,linux/arm64)")]
    pub platform: Option<String>,
    #[schemars(description = "Image tag(s) (comma-separated)")]
    pub tags: Option<String>,
    #[schemars(description = "Path to Dockerfile")]
    pub file: Option<String>,
    #[schemars(description = "Push image after build")]
    pub push: Option<bool>,
    #[schemars(description = "Load single-platform image into docker/podman")]
    pub load: Option<bool>,
    #[schemars(description = "Build arguments (KEY=VALUE, comma-separated)")]
    pub build_args: Option<String>,
    #[schemars(description = "Builder instance name")]
    pub builder: Option<String>,
    #[schemars(description = "Additional arguments")]
    pub args: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BuildahRequest {
    #[schemars(
        description = "Buildah subcommand: from, run, copy, add, commit, push, pull, images, containers, rm, rmi, build"
    )]
    pub command: String,
    #[schemars(description = "Target (image, container ID, or context path)")]
    pub target: Option<String>,
    #[schemars(description = "Image tag for commit/push")]
    pub tag: Option<String>,
    #[schemars(description = "Source path (for copy/add)")]
    pub source: Option<String>,
    #[schemars(description = "Destination path (for copy/add)")]
    pub dest: Option<String>,
    #[schemars(description = "Command to run (for run subcommand)")]
    pub run_command: Option<String>,
    #[schemars(description = "Path to Containerfile/Dockerfile (for build)")]
    pub file: Option<String>,
    #[schemars(description = "Output format: json (for images, containers)")]
    pub format: Option<String>,
    #[schemars(description = "Additional arguments")]
    pub args: Option<String>,
}

// ============================================================================
// KUBERNETES TOOLS
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KubectlGetRequest {
    #[schemars(
        description = "Resource type: pods, deployments, services, configmaps, secrets, nodes, namespaces, events"
    )]
    pub resource: String,
    #[schemars(description = "Resource name (optional, lists all if omitted)")]
    pub name: Option<String>,
    #[schemars(description = "Namespace (default: current)")]
    pub namespace: Option<String>,
    #[schemars(description = "All namespaces")]
    pub all_namespaces: Option<bool>,
    #[schemars(description = "Label selector (e.g., 'app=nginx')")]
    pub selector: Option<String>,
    #[schemars(description = "Output format: json, yaml, wide, name")]
    pub output: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KubectlDescribeRequest {
    #[schemars(description = "Resource type")]
    pub resource: String,
    #[schemars(description = "Resource name")]
    pub name: String,
    #[schemars(description = "Namespace")]
    pub namespace: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KubectlLogsRequest {
    #[schemars(description = "Pod name")]
    pub pod: String,
    #[schemars(description = "Container name (if multiple containers)")]
    pub container: Option<String>,
    #[schemars(description = "Namespace")]
    pub namespace: Option<String>,
    #[schemars(description = "Number of lines to show")]
    pub tail: Option<u32>,
    #[schemars(description = "Show previous container logs")]
    pub previous: Option<bool>,
    #[schemars(description = "Show logs since duration (e.g., '1h', '5m')")]
    pub since: Option<String>,
    #[schemars(description = "Include timestamps")]
    pub timestamps: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KubectlApplyRequest {
    #[schemars(description = "YAML/JSON manifest content")]
    pub manifest: String,
    #[schemars(description = "Namespace")]
    pub namespace: Option<String>,
    #[schemars(description = "Dry run mode: none, client, server")]
    pub dry_run: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KubectlDeleteRequest {
    #[schemars(description = "Resource type")]
    pub resource: String,
    #[schemars(description = "Resource name")]
    pub name: String,
    #[schemars(description = "Namespace")]
    pub namespace: Option<String>,
    #[schemars(description = "Force deletion")]
    pub force: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KubectlExecRequest {
    #[schemars(description = "Pod name")]
    pub pod: String,
    #[schemars(description = "Command to execute")]
    pub command: String,
    #[schemars(description = "Container name")]
    pub container: Option<String>,
    #[schemars(description = "Namespace")]
    pub namespace: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SternRequest {
    #[schemars(description = "Pod query (regex or exact match)")]
    pub query: String,
    #[schemars(description = "Namespace")]
    pub namespace: Option<String>,
    #[schemars(description = "Container name filter (regex)")]
    pub container: Option<String>,
    #[schemars(description = "Show logs since duration")]
    pub since: Option<String>,
    #[schemars(description = "Label selector")]
    pub selector: Option<String>,
    #[schemars(description = "Output format: default, raw, json")]
    pub output: Option<String>,
    #[schemars(description = "Include timestamps")]
    pub timestamps: Option<bool>,
    #[schemars(description = "Maximum log lines per container")]
    pub tail: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HelmRequest {
    #[schemars(
        description = "Helm subcommand: list, status, get, install, upgrade, uninstall, search, show, repo"
    )]
    pub command: String,
    #[schemars(description = "Release name")]
    pub release: Option<String>,
    #[schemars(description = "Chart reference (for install/upgrade/show)")]
    pub chart: Option<String>,
    #[schemars(description = "Namespace")]
    pub namespace: Option<String>,
    #[schemars(description = "Values file path or inline YAML")]
    pub values: Option<String>,
    #[schemars(description = "Additional arguments")]
    pub args: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KustomizeRequest {
    #[schemars(description = "Subcommand: build, edit, create")]
    pub command: String,
    #[schemars(description = "Path to kustomization directory")]
    pub path: Option<String>,
    #[schemars(description = "Output format: yaml, json")]
    pub output: Option<String>,
}

// --- Shell Execution ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ShellExecRequest {
    #[schemars(description = "Command to execute")]
    pub command: String,
    #[schemars(description = "Shell: bash, zsh, fish, nu, dash (default: bash)")]
    pub shell: Option<String>,
    #[schemars(description = "Working directory")]
    pub working_dir: Option<String>,
    #[schemars(description = "Timeout in seconds (default: 30, max: 300)")]
    pub timeout: Option<u64>,
    #[schemars(description = "Environment variables as JSON object")]
    pub env: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NixShellExecRequest {
    #[schemars(description = "Command to execute in devshell")]
    pub command: String,
    #[schemars(description = "Flake reference (default: current directory)")]
    pub flake: Option<String>,
    #[schemars(description = "Devshell attribute name (e.g., 'default', 'ci')")]
    pub devshell: Option<String>,
    #[schemars(description = "Inner shell: bash, zsh, fish, nu (default: bash)")]
    pub shell: Option<String>,
    #[schemars(description = "Working directory")]
    pub working_dir: Option<String>,
    #[schemars(description = "Timeout in seconds (default: 120, max: 600)")]
    pub timeout: Option<u64>,
}

// --- Git Forge Auth ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GhAuthStatusRequest {
    #[schemars(description = "Specific hostname to check (default: github.com)")]
    pub hostname: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GhAuthLoginRequest {
    #[schemars(description = "Hostname to authenticate (default: github.com)")]
    pub hostname: Option<String>,
    #[schemars(description = "Authentication token (alternative to interactive login)")]
    pub token: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GlabAuthStatusRequest {
    #[schemars(description = "Specific hostname to check (default: gitlab.com)")]
    pub hostname: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GlabAuthLoginRequest {
    #[schemars(description = "Hostname to authenticate (default: gitlab.com)")]
    pub hostname: Option<String>,
    #[schemars(description = "Authentication token")]
    pub token: Option<String>,
}

// ========================================================================
// GIT PRIMITIVE REQUEST TYPES
// ========================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitStatusRequest {
    #[schemars(
        description = "Git repository path (runs git -C <path>). Defaults to current directory."
    )]
    pub path: Option<String>,
    #[schemars(description = "Short format output")]
    pub short: Option<bool>,
    #[schemars(description = "Machine-readable porcelain format")]
    pub porcelain: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitAddRequest {
    #[schemars(
        description = "Git repository path (runs git -C <path>). Defaults to current directory."
    )]
    pub path: Option<String>,
    #[schemars(description = "Files to add (space-separated paths, or '.' for all)")]
    pub files: String,
    #[schemars(description = "Add all changes including untracked")]
    pub all: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitCommitRequest {
    #[schemars(
        description = "Git repository path (runs git -C <path>). Defaults to current directory."
    )]
    pub path: Option<String>,
    #[schemars(description = "Commit message")]
    pub message: String,
    #[schemars(description = "Automatically stage modified/deleted files")]
    pub all: Option<bool>,
    #[schemars(description = "Amend previous commit")]
    pub amend: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitBranchRequest {
    #[schemars(
        description = "Git repository path (runs git -C <path>). Defaults to current directory."
    )]
    pub path: Option<String>,
    #[schemars(description = "Subcommand: list, create, delete, rename")]
    pub command: String,
    #[schemars(description = "Branch name")]
    pub name: Option<String>,
    #[schemars(description = "New name for rename operation")]
    pub new_name: Option<String>,
    #[schemars(description = "Force operation")]
    pub force: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitCheckoutRequest {
    #[schemars(
        description = "Git repository path (runs git -C <path>). Defaults to current directory."
    )]
    pub path: Option<String>,
    #[schemars(description = "Branch name, commit, or tag to checkout")]
    pub target: String,
    #[schemars(description = "Create new branch (-b flag)")]
    pub create: Option<bool>,
    #[schemars(description = "Specific files to checkout (space-separated)")]
    pub files: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitLogRequest {
    #[schemars(
        description = "Git repository path (runs git -C <path>). Defaults to current directory."
    )]
    pub path: Option<String>,
    #[schemars(description = "Number of commits to show")]
    pub count: Option<u32>,
    #[schemars(description = "One line per commit")]
    pub oneline: Option<bool>,
    #[schemars(description = "Custom format string")]
    pub format: Option<String>,
    #[schemars(description = "Show history for specific file")]
    pub file: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitStashRequest {
    #[schemars(
        description = "Git repository path (runs git -C <path>). Defaults to current directory."
    )]
    pub path: Option<String>,
    #[schemars(description = "Subcommand: push, pop, list, drop, apply, show")]
    pub command: String,
    #[schemars(description = "Stash message (for push)")]
    pub message: Option<String>,
    #[schemars(description = "Stash index (for pop/drop/apply/show)")]
    pub index: Option<u32>,
}

// ========================================================================
// CODE INTELLIGENCE REQUEST TYPES
// ========================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SymbolsRequest {
    #[schemars(description = "File or directory path to analyze")]
    pub path: String,
    #[schemars(description = "Language: rust, python, javascript, typescript, go")]
    pub language: Option<String>,
    #[schemars(description = "Filter pattern for symbol names")]
    pub pattern: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReferencesRequest {
    #[schemars(description = "Symbol name to find references for")]
    pub symbol: String,
    #[schemars(description = "Search scope path (default: current directory)")]
    pub path: Option<String>,
    #[schemars(description = "Language for context-aware search")]
    pub language: Option<String>,
}

// --- File Operations ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FileReadRequest {
    #[schemars(description = "Absolute path to file")]
    pub path: String,
    #[schemars(description = "Starting line number (1-indexed, default: 1)")]
    pub offset: Option<usize>,
    #[schemars(description = "Number of lines to read (default: all)")]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FileWriteRequest {
    #[schemars(description = "Absolute path to file")]
    pub path: String,
    #[schemars(description = "Content to write")]
    pub content: String,
    #[schemars(description = "Create parent directories if needed")]
    pub create_dirs: Option<bool>,
    #[schemars(description = "If true and file exists, move it to graveyard before writing")]
    pub safe_overwrite: Option<bool>,
    #[schemars(description = "Custom graveyard directory for safe_overwrite")]
    pub graveyard: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FileEditRequest {
    #[schemars(
        description = "Absolute path(s) to file - space-separated for batch edit across multiple files"
    )]
    pub path: String,
    #[schemars(
        description = "Text to find (must be unique in each file unless replace_all is true)"
    )]
    pub old_text: String,
    #[schemars(description = "Text to replace with")]
    pub new_text: String,
    #[schemars(description = "Replace all occurrences (default: false, fails if not unique)")]
    pub replace_all: Option<bool>,
    #[schemars(description = "If true, backup files to graveyard before editing")]
    pub backup: Option<bool>,
    #[schemars(description = "Custom graveyard directory for backup")]
    pub graveyard: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FileAppendRequest {
    #[schemars(description = "Absolute path to file")]
    pub path: String,
    #[schemars(description = "Content to append")]
    pub content: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FilePatchRequest {
    #[schemars(description = "Absolute path to file")]
    pub path: String,
    #[schemars(description = "Unified diff patch content")]
    pub patch: String,
    #[schemars(description = "If true, backup file to graveyard before patching")]
    pub backup: Option<bool>,
    #[schemars(description = "Custom graveyard directory for backup")]
    pub graveyard: Option<String>,
}

// --- Filesystem Operations ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FsMkdirRequest {
    #[schemars(description = "Directory path(s) to create - space-separated for multiple")]
    pub path: String,
    #[schemars(description = "Create parent directories (default: true)")]
    pub parents: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FsCopyRequest {
    #[schemars(description = "Source path(s) - space-separated for multiple files")]
    pub source: String,
    #[schemars(description = "Destination path (must be directory if multiple sources)")]
    pub dest: String,
    #[schemars(description = "Copy directories recursively")]
    pub recursive: Option<bool>,
    #[schemars(description = "If true and dest exists, move it to graveyard before copying")]
    pub safe_overwrite: Option<bool>,
    #[schemars(description = "Custom graveyard directory for safe_overwrite")]
    pub graveyard: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FsMoveRequest {
    #[schemars(description = "Source path(s) - space-separated for multiple files")]
    pub source: String,
    #[schemars(description = "Destination path (must be directory if multiple sources)")]
    pub dest: String,
    #[schemars(description = "If true and dest exists, move dest to graveyard before overwriting")]
    pub safe_overwrite: Option<bool>,
    #[schemars(
        description = "Custom graveyard directory for safe_overwrite (default: ~/.graveyard)"
    )]
    pub graveyard: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FsStatRequest {
    #[schemars(description = "Path(s) to get info for - space-separated for multiple")]
    pub path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FsExistsRequest {
    #[schemars(description = "Path(s) to check - space-separated for multiple")]
    pub path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FsSymlinkRequest {
    #[schemars(description = "Target path (what the link points to)")]
    pub target: String,
    #[schemars(description = "Link path (the symlink to create)")]
    pub link: String,
    #[schemars(description = "If true and link exists, move it to graveyard before creating")]
    pub safe_overwrite: Option<bool>,
    #[schemars(description = "Custom graveyard directory for safe_overwrite")]
    pub graveyard: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FsHardlinkRequest {
    #[schemars(description = "Source path (existing file to link to)")]
    pub source: String,
    #[schemars(description = "Link path (the hard link to create)")]
    pub link: String,
    #[schemars(description = "If true and link exists, move it to graveyard before creating")]
    pub safe_overwrite: Option<bool>,
    #[schemars(description = "Custom graveyard directory for safe_overwrite")]
    pub graveyard: Option<String>,
}

// --- MCP State Tools ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct McpCacheGetRequest {
    #[schemars(description = "Cache key")]
    pub key: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct McpCacheSetRequest {
    #[schemars(description = "Cache key")]
    pub key: String,
    #[schemars(description = "Value to cache")]
    pub value: String,
    #[schemars(description = "Time-to-live in seconds (optional)")]
    pub ttl_secs: Option<i64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct McpTaskCreateRequest {
    #[schemars(description = "Task description")]
    pub content: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct McpTaskUpdateRequest {
    #[schemars(description = "Task ID")]
    pub id: i64,
    #[schemars(description = "New status: pending, in_progress, completed")]
    pub status: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct McpTaskListRequest {
    #[schemars(description = "Filter by status: pending, in_progress, completed (optional)")]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct McpTaskDeleteRequest {
    #[schemars(description = "Task ID to delete")]
    pub id: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct McpContextGetRequest {
    #[schemars(description = "Context key")]
    pub key: String,
    #[schemars(description = "Scope: session, project, global (default: session)")]
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct McpContextSetRequest {
    #[schemars(description = "Context key")]
    pub key: String,
    #[schemars(description = "Value to store")]
    pub value: String,
    #[schemars(description = "Scope: session, project, global (default: session)")]
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct McpContextListRequest {
    #[schemars(description = "Filter by scope: session, project, global (optional)")]
    pub scope: Option<String>,
}

// --- Virtual Tool Groups ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExpandToolsRequest {
    #[schemars(
        description = "Tool group to expand. Available groups: filesystem, file_ops, search, \
        text, git, github, gitlab, kubernetes, container, network, system, archive, reference, diff, mcp"
    )]
    pub group: String,
}

// --- Dynamic Toolsets (Beta) ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetToolsetToolsRequest {
    #[schemars(
        description = "Toolset name to get tools for. Available: filesystem, file_ops, search, \
        text, git, github, gitlab, kubernetes, container, network, system, archive, reference, diff, mcp"
    )]
    pub toolset: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EnableToolsetRequest {
    #[schemars(
        description = "Toolset name to enable. Use 'all' to enable all toolsets. \
        Available: filesystem, file_ops, search, text, git, github, gitlab, kubernetes, \
        container, network, system, archive, reference, diff, mcp"
    )]
    pub toolset: String,
}

// ============================================================================
// TOOL IMPLEMENTATIONS
// ============================================================================

#[tool_router]
impl ModernCliTools {
    /// Create a new ModernCliTools instance with default settings (all tools enabled).
    #[allow(dead_code)]
    pub fn new(profile: Option<AgentProfile>) -> Self {
        Self::new_with_config(profile, false, Vec::new(), false)
    }

    pub fn new_with_config(
        profile: Option<AgentProfile>,
        dynamic_toolsets: bool,
        pre_enabled_groups: Vec<ToolGroup>,
        dual_response: bool,
    ) -> Self {
        let state = StateManager::new().expect("Failed to initialize state manager");
        let ignore = AgentIgnore::new().unwrap_or_default();

        // Initialize enabled groups
        let enabled_groups: HashSet<ToolGroup> = if dynamic_toolsets {
            pre_enabled_groups.into_iter().collect()
        } else {
            // When not in dynamic mode, all groups are implicitly enabled
            ToolGroup::ALL.iter().copied().collect()
        };

        // Build reverse lookup: tool name -> group
        let mut tool_to_group = HashMap::new();
        for group in ToolGroup::ALL {
            for tool_name in group.tools() {
                tool_to_group.insert(*tool_name, *group);
            }
        }

        Self {
            tool_router: Self::tool_router(),
            executor: CommandExecutor::new(),
            state: Arc::new(state),
            profile,
            ignore: Arc::new(ignore),
            dynamic_config: DynamicToolsetConfig {
                enabled: dynamic_toolsets,
                enabled_groups: Arc::new(RwLock::new(enabled_groups)),
            },
            tool_to_group,
            dual_response,
        }
    }

    /// Check if a tool group is currently enabled
    fn is_group_enabled(&self, group: ToolGroup) -> bool {
        if !self.dynamic_config.enabled {
            return true; // All groups enabled when not in dynamic mode
        }
        self.dynamic_config.enabled_groups.read().contains(&group)
    }

    /// Enable a tool group (for dynamic toolsets mode)
    fn enable_group(&self, group: ToolGroup) -> bool {
        if !self.dynamic_config.enabled {
            return false; // No-op when not in dynamic mode
        }
        let mut groups = self.dynamic_config.enabled_groups.write();
        groups.insert(group)
    }

    /// Get the list of currently enabled groups
    #[allow(dead_code)]
    fn get_enabled_groups(&self) -> Vec<ToolGroup> {
        self.dynamic_config
            .enabled_groups
            .read()
            .iter()
            .copied()
            .collect()
    }

    /// Build a tool response, optionally with dual-response format.
    ///
    /// In dual-response mode, returns two content blocks:
    /// 1. Human-readable summary (text)
    /// 2. Raw structured data (embedded resource)
    ///
    /// In normal mode, returns only the raw data as text.
    fn build_response(&self, summary: &str, raw_data: &str, uri: &str) -> CallToolResult {
        if self.dual_response {
            CallToolResult::success(vec![
                Content::text(summary),
                Content::embedded_text(uri, raw_data),
            ])
        } else {
            CallToolResult::success(vec![Content::text(raw_data)])
        }
    }

    /// Build an error response (same format regardless of dual-response mode)
    fn build_error(&self, error: &str) -> CallToolResult {
        CallToolResult::error(vec![Content::text(error)])
    }

    // ========================================================================
    // FILESYSTEM TOOLS
    // ========================================================================

    #[tool(
        name = "Filesystem - List (eza)",
        description = "List directory contents with eza (modern ls replacement). Returns JSON. \
        Features: icons, git integration, tree view, extended attributes."
    )]
    async fn eza(
        &self,
        Parameters(req): Parameters<EzaRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        // Validate directory path against .agentignore
        let path_str = req.path.as_deref().unwrap_or(".");
        if let Err(msg) = self.ignore.validate_path(std::path::Path::new(path_str)) {
            return Ok(CallToolResult::error(vec![Content::text(msg)]));
        }

        let mut args: Vec<String> = vec!["--color=never".into()];

        if req.all.unwrap_or(false) {
            args.push("-a".into());
        }
        if req.long.unwrap_or(false) {
            args.push("-l".into());
        }
        if req.tree.unwrap_or(false) {
            args.push("--tree".into());
        }
        if req.git.unwrap_or(false) {
            args.push("--git".into());
        }
        if req.icons.unwrap_or(false) {
            args.push("--icons".into());
        }
        if req.dirs_only.unwrap_or(false) {
            args.push("-D".into());
        }
        if req.files_only.unwrap_or(false) {
            args.push("-f".into());
        }
        if req.reverse.unwrap_or(false) {
            args.push("-r".into());
        }
        if let Some(level) = req.level {
            args.push(format!("--level={}", level));
        }
        if let Some(ref sort) = req.sort {
            args.push(format!("--sort={}", sort));
        }
        let path = req.path.clone().unwrap_or_else(|| ".".to_string());
        args.push(path.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("eza", &args_ref).await {
            Ok(output) => {
                let json_output = parse_eza_to_json(&output.stdout, &path);
                let summary = format::format_eza_summary(&json_output, &path);
                Ok(self.build_response(&summary, &json_output, "data://eza/listing.json"))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Filesystem - View (bat)",
        description = "Display file contents with syntax highlighting using bat (modern cat replacement). \
        Features: syntax highlighting, line numbers, git integration, line ranges."
    )]
    async fn bat(
        &self,
        Parameters(req): Parameters<BatRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        // Check .agentignore
        let path = std::path::Path::new(&req.path);
        if let Err(msg) = self.ignore.validate_path(path) {
            return Ok(CallToolResult::error(vec![Content::text(msg)]));
        }

        let mut args: Vec<String> = vec!["--color=never".into(), "--paging=never".into()];

        if req.number.unwrap_or(true) {
            args.push("-n".into());
        }
        if req.show_all.unwrap_or(false) {
            args.push("-A".into());
        }
        if let Some(ref lang) = req.language {
            args.push(format!("--language={}", lang));
        }
        if let Some(ref range) = req.range {
            args.push(format!("--line-range={}", range));
        }
        if let Some(ref highlight) = req.highlight {
            args.push(format!("--highlight-line={}", highlight));
        }
        if let Some(ref style) = req.style {
            args.push(format!("--style={}", style));
        }
        args.push(req.path.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("bat", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Filesystem - Find (fd)",
        description = "Find files and directories with fd (modern find replacement). Returns JSON. \
        Features: regex patterns, respects .agentignore, type filtering, parallel execution."
    )]
    async fn fd(
        &self,
        Parameters(req): Parameters<FdRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["--color=never".into()];

        // Add .agentignore support (disables .gitignore, uses only .agentignore)
        let working_dir = req.path.as_deref().unwrap_or(".");
        let ignore_args = self
            .ignore
            .get_ignore_file_args(std::path::Path::new(working_dir));
        args.extend(ignore_args);

        if req.hidden.unwrap_or(false) {
            args.push("-H".into());
        }
        // Note: no_ignore is deprecated - .agentignore is now the only ignore mechanism
        if req.ignore_case.unwrap_or(false) {
            args.push("-i".into());
        }
        if req.follow.unwrap_or(false) {
            args.push("-L".into());
        }
        if req.absolute.unwrap_or(false) {
            args.push("-a".into());
        }
        if let Some(ref ext) = req.extension {
            args.push(format!("-e{}", ext));
        }
        if let Some(ref ft) = req.file_type {
            args.push(format!("-t{}", ft));
        }
        if let Some(depth) = req.max_depth {
            args.push(format!("--max-depth={}", depth));
        }
        if let Some(depth) = req.min_depth {
            args.push(format!("--min-depth={}", depth));
        }
        if let Some(ref exclude) = req.exclude {
            args.push(format!("--exclude={}", exclude));
        }
        if let Some(ref size) = req.size {
            args.push(format!("--size={}", size));
        }
        if let Some(ref changed) = req.changed_within {
            args.push(format!("--changed-within={}", changed));
        }
        if let Some(max) = req.max_results {
            args.push(format!("--max-results={}", max));
        }
        // fd expects [pattern] [path] - if path is given without pattern, use "." to match all
        if let Some(ref pattern) = req.pattern {
            args.push(pattern.clone());
        } else if req.path.is_some() {
            // Default pattern to match all files when only path is specified
            args.push(".".into());
        }
        if let Some(ref path) = req.path {
            args.push(path.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let pattern = req.pattern.as_deref().unwrap_or("*");
        match self.executor.run("fd", &args_ref).await {
            Ok(output) => {
                let json_output = parse_fd_to_json(&output.stdout);
                let summary = format::format_fd_summary(&json_output, pattern);
                Ok(self.build_response(&summary, &json_output, "data://fd/results.json"))
            }
            Err(e) => Ok(self.build_error(&e)),
        }
    }

    #[tool(
        name = "Filesystem - Disk Usage (duf)",
        description = "Show disk usage with duf (modern df replacement). \
        Features: colorful output, all mount points, JSON output."
    )]
    async fn duf(
        &self,
        Parameters(req): Parameters<DufRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];

        if req.all.unwrap_or(false) {
            args.push("--all".into());
        }
        if req.inodes.unwrap_or(false) {
            args.push("--inodes".into());
        }
        if req.json.unwrap_or(false) {
            args.push("--json".into());
        }
        if let Some(ref path) = req.path {
            args.push(path.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("duf", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Filesystem - Directory Size (dust)",
        description = "Analyze directory sizes with dust (modern du replacement). \
        Features: visual bars, tree view, customizable depth."
    )]
    async fn dust(
        &self,
        Parameters(req): Parameters<DustRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        // Validate directory path against .agentignore
        let path_str = req.path.as_deref().unwrap_or(".");
        if let Err(msg) = self.ignore.validate_path(std::path::Path::new(path_str)) {
            return Ok(CallToolResult::error(vec![Content::text(msg)]));
        }

        let mut args: Vec<String> = vec![];

        if req.reverse.unwrap_or(false) {
            args.push("-r".into());
        }
        if req.only_dirs.unwrap_or(false) {
            args.push("-D".into());
        }
        if req.only_files.unwrap_or(false) {
            args.push("-F".into());
        }
        if let Some(n) = req.number {
            args.push(format!("-n{}", n));
        }
        if let Some(d) = req.depth {
            args.push(format!("-d{}", d));
        }
        let path = req.path.clone().unwrap_or_else(|| ".".to_string());
        args.push(path.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("dust", &args_ref).await {
            Ok(output) => {
                let json_output = parse_dust_to_json(&output.stdout, &path);
                Ok(CallToolResult::success(vec![Content::text(json_output)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Filesystem - Trash (rip)",
        description = "Move file(s) or directory(s) to graveyard (safe delete using rip). Supports multiple space-separated paths."
    )]
    async fn trash_put(
        &self,
        Parameters(req): Parameters<TrashRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];
        if let Some(graveyard) = &req.graveyard {
            args.push(format!("--graveyard={}", graveyard));
        }

        // Support multiple space-separated paths
        let paths: Vec<&str> = req.path.split_whitespace().collect();
        for path in &paths {
            args.push((*path).to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("rip", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                if output.success {
                    let result = serde_json::json!({
                        "success": true,
                        "paths": paths,
                        "count": paths.len(),
                        "graveyard": req.graveyard.as_deref().unwrap_or("~/.graveyard")
                    });
                    result.to_string()
                } else {
                    output.to_result_string()
                },
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Filesystem - Trash List (rip)",
        description = "List files in graveyard (seance). Shows deleted files with restore info."
    )]
    async fn trash_list(
        &self,
        Parameters(req): Parameters<TrashListRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["--seance".into()];
        if let Some(graveyard) = &req.graveyard {
            args.push(format!("--graveyard={}", graveyard));
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("rip", &args_ref).await {
            Ok(output) => {
                // Parse seance output to JSON
                let lines: Vec<&str> = output.stdout.lines().collect();
                let items: Vec<serde_json::Value> = lines
                    .iter()
                    .map(|line| serde_json::json!({ "entry": line.trim() }))
                    .collect();
                let result = serde_json::json!({
                    "count": items.len(),
                    "graveyard": req.graveyard.as_deref().unwrap_or("~/.graveyard"),
                    "items": items
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Filesystem - Trash Restore (rip)",
        description = "Restore file from graveyard (unbury). Restores last deleted item by default."
    )]
    async fn trash_restore(
        &self,
        Parameters(req): Parameters<TrashRestoreRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["--unbury".into()];
        if let Some(graveyard) = &req.graveyard {
            args.push(format!("--graveyard={}", graveyard));
        }
        if let Some(target) = &req.target {
            args.push(target.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("rip", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                if output.success {
                    format!("Restored from graveyard: {}", output.stdout.trim())
                } else {
                    output.to_result_string()
                },
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // GROUPED TOOLS (15 tools instead of 106)
    // ========================================================================

    #[tool(
        name = "filesystem",
        description = "Filesystem operations. Subcommands: list (eza), view (bat), find (fd), \
        disk_usage (duf), dir_size (dust), trash, trash_list, trash_restore, copy, move, \
        mkdir, stat, exists, symlink, hardlink, file_type, permissions"
    )]
    async fn filesystem_group(
        &self,
        Parameters(req): Parameters<FilesystemGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "list" | "ls" | "eza" => {
                let eza_req = EzaRequest {
                    path: req.path,
                    all: req.all,
                    long: req.long,
                    tree: req.tree,
                    level: req.level,
                    git: req.git,
                    icons: req.icons,
                    sort: req.sort,
                    reverse: req.reverse,
                    dirs_only: req.dirs_only,
                    files_only: req.files_only,
                };
                self.eza(Parameters(eza_req)).await
            }

            "view" | "cat" | "bat" => {
                let path = req.path.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "path is required for view command",
                        None::<serde_json::Value>,
                    )
                })?;
                let bat_req = BatRequest {
                    path,
                    language: req.language,
                    number: req.number,
                    range: req.range,
                    highlight: req.highlight,
                    style: req.style,
                    show_all: req.show_all,
                };
                self.bat(Parameters(bat_req)).await
            }

            "find" | "fd" => {
                let fd_req = FdRequest {
                    pattern: req.pattern,
                    path: req.path,
                    extension: req.extension,
                    file_type: req.file_type,
                    hidden: req.hidden,
                    no_ignore: req.no_ignore,
                    max_depth: req.max_depth,
                    min_depth: req.min_depth,
                    ignore_case: req.ignore_case,
                    exclude: req.exclude,
                    follow: req.follow,
                    absolute: req.absolute,
                    size: req.size,
                    changed_within: req.changed_within,
                    max_results: None, // Use individual fd tool for max_results
                };
                self.fd(Parameters(fd_req)).await
            }

            "disk_usage" | "duf" => {
                let duf_req = DufRequest {
                    path: req.path,
                    all: req.duf_all,
                    inodes: req.inodes,
                    json: req.json,
                };
                self.duf(Parameters(duf_req)).await
            }

            "dir_size" | "dust" => {
                let dust_req = DustRequest {
                    path: req.path,
                    number: req.dust_number,
                    depth: req.depth,
                    reverse: req.reverse,
                    only_dirs: req.only_dirs,
                    only_files: req.only_files,
                };
                self.dust(Parameters(dust_req)).await
            }

            "trash" | "rm" | "delete" => {
                let path = req.path.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "path is required for trash command",
                        None::<serde_json::Value>,
                    )
                })?;
                let trash_req = TrashRequest {
                    path,
                    graveyard: req.graveyard,
                };
                self.trash_put(Parameters(trash_req)).await
            }

            "trash_list" | "seance" => {
                let trash_list_req = TrashListRequest {
                    graveyard: req.graveyard,
                };
                self.trash_list(Parameters(trash_list_req)).await
            }

            "trash_restore" | "unbury" => {
                let trash_restore_req = TrashRestoreRequest {
                    target: req.restore_target,
                    graveyard: req.graveyard,
                };
                self.trash_restore(Parameters(trash_restore_req)).await
            }

            "copy" | "cp" => {
                let source = req.path.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "path (source) is required for copy command",
                        None::<serde_json::Value>,
                    )
                })?;
                let dest = req.dest.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "dest is required for copy command",
                        None::<serde_json::Value>,
                    )
                })?;
                let copy_req = FsCopyRequest {
                    source,
                    dest,
                    recursive: req.recursive,
                    safe_overwrite: req.safe_overwrite,
                    graveyard: req.graveyard,
                };
                self.fs_copy(Parameters(copy_req)).await
            }

            "move" | "mv" => {
                let source = req.path.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "path (source) is required for move command",
                        None::<serde_json::Value>,
                    )
                })?;
                let dest = req.dest.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "dest is required for move command",
                        None::<serde_json::Value>,
                    )
                })?;
                let move_req = FsMoveRequest {
                    source,
                    dest,
                    safe_overwrite: req.safe_overwrite,
                    graveyard: req.graveyard,
                };
                self.fs_move(Parameters(move_req)).await
            }

            "mkdir" => {
                let path = req.path.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "path is required for mkdir command",
                        None::<serde_json::Value>,
                    )
                })?;
                let mkdir_req = FsMkdirRequest {
                    path,
                    parents: req.parents,
                };
                self.fs_mkdir(Parameters(mkdir_req)).await
            }

            "stat" => {
                let path = req.path.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "path is required for stat command",
                        None::<serde_json::Value>,
                    )
                })?;
                let stat_req = FsStatRequest { path };
                self.fs_stat(Parameters(stat_req)).await
            }

            "exists" => {
                let path = req.path.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "path is required for exists command",
                        None::<serde_json::Value>,
                    )
                })?;
                let exists_req = FsExistsRequest { path };
                self.fs_exists(Parameters(exists_req)).await
            }

            "symlink" | "ln_s" => {
                let target = req.target.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "target is required for symlink command",
                        None::<serde_json::Value>,
                    )
                })?;
                let link = req.link.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "link is required for symlink command",
                        None::<serde_json::Value>,
                    )
                })?;
                let symlink_req = FsSymlinkRequest {
                    target,
                    link,
                    safe_overwrite: req.safe_overwrite,
                    graveyard: req.graveyard,
                };
                self.fs_symlink(Parameters(symlink_req)).await
            }

            "hardlink" | "ln" => {
                let source = req.source.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "source is required for hardlink command",
                        None::<serde_json::Value>,
                    )
                })?;
                let link = req.link.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "link is required for hardlink command",
                        None::<serde_json::Value>,
                    )
                })?;
                let hardlink_req = FsHardlinkRequest {
                    source,
                    link,
                    safe_overwrite: req.safe_overwrite,
                    graveyard: req.graveyard,
                };
                self.fs_hardlink(Parameters(hardlink_req)).await
            }

            "file_type" | "file" => {
                let path = req.path.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "path is required for file_type command",
                        None::<serde_json::Value>,
                    )
                })?;
                let file_type_req = FileTypeRequest { path };
                self.file_type(Parameters(file_type_req)).await
            }

            "permissions" | "perms" => {
                let mode = req.mode.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "mode is required for permissions command",
                        None::<serde_json::Value>,
                    )
                })?;
                let perms_req = PermissionsRequest { mode };
                self.permissions(Parameters(perms_req)).await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!(
                    "Unknown filesystem command: '{}'. Available: list, view, find, \
                    disk_usage, dir_size, trash, trash_list, trash_restore, copy, move, \
                    mkdir, stat, exists, symlink, hardlink, file_type, permissions",
                    req.command
                ),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // FILE_OPS GROUPED TOOL
    // ========================================================================

    #[tool(
        name = "file_ops",
        description = "File operations. Subcommands: read, write, edit, append, patch"
    )]
    async fn file_ops_group(
        &self,
        Parameters(req): Parameters<FileOpsGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "read" => {
                let read_req = FileReadRequest {
                    path: req.path,
                    offset: req.offset,
                    limit: req.limit,
                };
                self.file_read(Parameters(read_req)).await
            }

            "write" => {
                let content = req.content.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "content is required for write command",
                        None::<serde_json::Value>,
                    )
                })?;
                let write_req = FileWriteRequest {
                    path: req.path,
                    content,
                    create_dirs: req.create_dirs,
                    safe_overwrite: req.safe_overwrite,
                    graveyard: req.graveyard,
                };
                self.file_write(Parameters(write_req)).await
            }

            "edit" => {
                let old_text = req.old_text.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "old_text is required for edit command",
                        None::<serde_json::Value>,
                    )
                })?;
                let new_text = req.new_text.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "new_text is required for edit command",
                        None::<serde_json::Value>,
                    )
                })?;
                let edit_req = FileEditRequest {
                    path: req.path,
                    old_text,
                    new_text,
                    replace_all: req.replace_all,
                    backup: req.backup,
                    graveyard: req.graveyard,
                };
                self.file_edit(Parameters(edit_req)).await
            }

            "append" => {
                let content = req.content.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "content is required for append command",
                        None::<serde_json::Value>,
                    )
                })?;
                let append_req = FileAppendRequest {
                    path: req.path,
                    content,
                };
                self.file_append(Parameters(append_req)).await
            }

            "patch" => {
                let patch = req.patch.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "patch is required for patch command",
                        None::<serde_json::Value>,
                    )
                })?;
                let patch_req = FilePatchRequest {
                    path: req.path,
                    patch,
                    backup: req.backup,
                    graveyard: req.graveyard,
                };
                self.file_patch(Parameters(patch_req)).await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!(
                    "Unknown file_ops command: '{}'. Available: read, write, edit, append, patch",
                    req.command
                ),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // SEARCH GROUPED TOOL
    // ========================================================================

    #[tool(
        name = "search",
        description = "Search operations. Subcommands: grep (ripgrep), ast (ast-grep), symbols, references, fzf"
    )]
    async fn search_group(
        &self,
        Parameters(req): Parameters<SearchGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "grep" | "rg" | "ripgrep" => {
                let pattern = req.pattern.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "pattern is required for grep command",
                        None::<serde_json::Value>,
                    )
                })?;
                let rg_req = RipgrepRequest {
                    pattern,
                    path: req.path,
                    ignore_case: req.ignore_case,
                    smart_case: req.smart_case,
                    hidden: req.hidden,
                    no_ignore: req.no_ignore,
                    files_with_matches: req.files_with_matches,
                    count: req.count,
                    line_number: req.line_number,
                    context: req.context,
                    file_type: req.file_type,
                    glob: req.glob,
                    word: req.word,
                    fixed_strings: req.fixed_strings,
                    multiline: req.multiline,
                    follow: req.follow,
                    json: req.json,
                    max_count: req.max_count,
                    max_results: None, // Use individual rg tool for max_results
                    invert: req.invert,
                    only_matching: req.only_matching,
                    replace: req.replace,
                };
                self.rg(Parameters(rg_req)).await
            }

            "ast" | "sg" | "ast-grep" => {
                let pattern = req.pattern.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "pattern is required for ast command",
                        None::<serde_json::Value>,
                    )
                })?;
                let ast_req = AstGrepRequest {
                    pattern,
                    path: req.path,
                    lang: req.lang,
                    rewrite: req.rewrite,
                };
                self.ast_grep(Parameters(ast_req)).await
            }

            "symbols" => {
                let path = req.path.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "path is required for symbols command",
                        None::<serde_json::Value>,
                    )
                })?;
                let symbols_req = SymbolsRequest {
                    path,
                    language: req.language,
                    pattern: req.pattern,
                };
                self.symbols(Parameters(symbols_req)).await
            }

            "references" | "refs" => {
                let symbol = req.symbol.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "symbol is required for references command",
                        None::<serde_json::Value>,
                    )
                })?;
                let refs_req = ReferencesRequest {
                    symbol,
                    path: req.path,
                    language: req.language,
                };
                self.references(Parameters(refs_req)).await
            }

            "fzf" | "fuzzy" => {
                let input = req.input.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "input is required for fzf command",
                        None::<serde_json::Value>,
                    )
                })?;
                let query = req.query.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "query is required for fzf command",
                        None::<serde_json::Value>,
                    )
                })?;
                let fzf_req = FzfFilterRequest {
                    input,
                    query,
                    exact: req.exact,
                    ignore_case: req.ignore_case,
                    limit: req.limit,
                };
                self.fzf_filter(Parameters(fzf_req)).await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!(
                    "Unknown search command: '{}'. Available: grep, ast, symbols, references, fzf",
                    req.command
                ),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // TEXT GROUPED TOOL
    // ========================================================================

    #[tool(
        name = "text",
        description = "Text processing. Subcommands: jq, yq, sd, htmlq, pup, miller, dasel, gron, hck, csv"
    )]
    async fn text_group(
        &self,
        Parameters(req): Parameters<TextGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "jq" => {
                let filter = req.filter.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "filter is required for jq command",
                        None::<serde_json::Value>,
                    )
                })?;
                let jq_req = JqRequest {
                    filter,
                    input: req.input,
                    raw: req.raw,
                    compact: req.compact,
                    slurp: req.slurp,
                    sort_keys: req.sort_keys,
                };
                self.jq(Parameters(jq_req)).await
            }

            "yq" => {
                let expression = req.expression.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "expression is required for yq command",
                        None::<serde_json::Value>,
                    )
                })?;
                let yq_req = YqRequest {
                    expression,
                    input: req.input,
                    input_format: req.input_format,
                    output_format: req.output_format,
                    prettyprint: req.prettyprint,
                };
                self.yq(Parameters(yq_req)).await
            }

            "sd" => {
                let find = req.find.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "find is required for sd command",
                        None::<serde_json::Value>,
                    )
                })?;
                let replace = req.replace.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "replace is required for sd command",
                        None::<serde_json::Value>,
                    )
                })?;
                let sd_req = SdRequest {
                    find,
                    replace,
                    input: req.input,
                    fixed: req.fixed,
                };
                self.sd(Parameters(sd_req)).await
            }

            "htmlq" => {
                let selector = req.selector.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "selector is required for htmlq command",
                        None::<serde_json::Value>,
                    )
                })?;
                let htmlq_req = HtmlqRequest {
                    input: req.input,
                    selector,
                    attribute: req.attribute,
                    text: req.text,
                };
                self.htmlq(Parameters(htmlq_req)).await
            }

            "pup" => {
                let selector = req.selector.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "selector is required for pup command",
                        None::<serde_json::Value>,
                    )
                })?;
                let pup_req = PupRequest {
                    input: req.input,
                    selector,
                    json: req.json,
                };
                self.pup(Parameters(pup_req)).await
            }

            "miller" | "mlr" => {
                let verb = req.verb.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "verb is required for miller command",
                        None::<serde_json::Value>,
                    )
                })?;
                let miller_req = MillerRequest {
                    verb,
                    input: req.input,
                    input_format: req.input_format,
                    output_format: req.output_format,
                    args: req.args,
                };
                self.miller(Parameters(miller_req)).await
            }

            "dasel" => {
                let selector = req.dasel_selector.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "dasel_selector is required for dasel command",
                        None::<serde_json::Value>,
                    )
                })?;
                let dasel_req = DaselRequest {
                    selector,
                    input: req.input,
                    input_format: req.input_format,
                    output_format: req.output_format,
                    compact: req.dasel_compact,
                };
                self.dasel(Parameters(dasel_req)).await
            }

            "gron" => {
                let gron_req = GronRequest {
                    input: req.input,
                    ungron: req.ungron,
                    stream: req.stream,
                };
                self.gron(Parameters(gron_req)).await
            }

            "hck" | "cut" => {
                let fields = req.fields.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "fields is required for hck command",
                        None::<serde_json::Value>,
                    )
                })?;
                let hck_req = HckRequest {
                    input: req.input,
                    fields,
                    delimiter: req.delimiter,
                    output_delimiter: req.output_delimiter,
                };
                self.hck(Parameters(hck_req)).await
            }

            "csv" | "xsv" => {
                let command = req.csv_command.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "csv_command is required for csv command",
                        None::<serde_json::Value>,
                    )
                })?;
                let xsv_req = XsvRequest {
                    command,
                    input: req.input,
                    delimiter: req.delimiter,
                    no_headers: req.no_headers,
                    args: req.args,
                };
                self.xsv(Parameters(xsv_req)).await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!("Unknown text command: '{}'. Available: jq, yq, sd, htmlq, pup, miller, dasel, gron, hck, csv", req.command),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // GIT GROUPED TOOL
    // ========================================================================

    #[tool(
        name = "git",
        description = "Git operations. Subcommands: status, add, commit, branch, checkout, log, diff, stash"
    )]
    async fn git_group(
        &self,
        Parameters(req): Parameters<GitGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "status" => {
                let status_req = GitStatusRequest {
                    path: req.path,
                    short: req.short,
                    porcelain: req.porcelain,
                };
                self.git_status(Parameters(status_req)).await
            }

            "add" => {
                let files = req.files.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "files is required for add command",
                        None::<serde_json::Value>,
                    )
                })?;
                let add_req = GitAddRequest {
                    files,
                    path: req.path,
                    all: req.all,
                };
                self.git_add(Parameters(add_req)).await
            }

            "commit" => {
                let message = req.message.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "message is required for commit command",
                        None::<serde_json::Value>,
                    )
                })?;
                let commit_req = GitCommitRequest {
                    message,
                    path: req.path,
                    all: req.commit_all,
                    amend: req.amend,
                };
                self.git_commit(Parameters(commit_req)).await
            }

            "branch" => {
                let branch_cmd = req.branch_command.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "branch_command is required for branch command",
                        None::<serde_json::Value>,
                    )
                })?;
                let branch_req = GitBranchRequest {
                    command: branch_cmd,
                    path: req.path,
                    name: req.name,
                    new_name: req.new_name,
                    force: req.force,
                };
                self.git_branch(Parameters(branch_req)).await
            }

            "checkout" => {
                let target = req.target.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "target is required for checkout command",
                        None::<serde_json::Value>,
                    )
                })?;
                let checkout_req = GitCheckoutRequest {
                    target,
                    path: req.path,
                    create: req.create,
                    files: req.checkout_files,
                };
                self.git_checkout(Parameters(checkout_req)).await
            }

            "log" => {
                let log_req = GitLogRequest {
                    path: req.path,
                    count: req.count,
                    file: req.file,
                    oneline: req.oneline,
                    format: req.format,
                };
                self.git_log(Parameters(log_req)).await
            }

            "diff" => {
                let diff_req = GitDiffRequest {
                    path: req.path,
                    staged: req.staged,
                    commit: req.commit,
                    range: req.range,
                    file: req.file,
                };
                self.git_diff(Parameters(diff_req)).await
            }

            "stash" => {
                let stash_cmd = req.stash_command.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "stash_command is required for stash command",
                        None::<serde_json::Value>,
                    )
                })?;
                let stash_req = GitStashRequest {
                    command: stash_cmd,
                    path: req.path,
                    message: req.stash_message,
                    index: req.index,
                };
                self.git_stash(Parameters(stash_req)).await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!("Unknown git command: '{}'. Available: status, add, commit, branch, checkout, log, diff, stash", req.command),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // GITHUB GROUPED TOOL
    // ========================================================================

    #[tool(
        name = "github",
        description = "GitHub operations. Subcommands: repo, issue, pr, search, release, workflow, run, api, auth_status, auth_login"
    )]
    async fn github_group(
        &self,
        Parameters(req): Parameters<GitHubGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "repo" => {
                let subcommand = req.subcommand.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "subcommand is required for repo command",
                        None::<serde_json::Value>,
                    )
                })?;
                let repo_req = GhRepoRequest {
                    command: subcommand,
                    repo: req.repo,
                    args: req.args,
                };
                self.gh_repo(Parameters(repo_req)).await
            }

            "issue" => {
                let subcommand = req.subcommand.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "subcommand is required for issue command",
                        None::<serde_json::Value>,
                    )
                })?;
                let issue_req = GhIssueRequest {
                    command: subcommand,
                    repo: req.repo,
                    number: req.number,
                    title: req.title,
                    body: req.body,
                    state: req.state,
                    limit: req.limit,
                    labels: req.labels,
                    assignees: req.assignees,
                };
                self.gh_issue(Parameters(issue_req)).await
            }

            "pr" => {
                let subcommand = req.subcommand.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "subcommand is required for pr command",
                        None::<serde_json::Value>,
                    )
                })?;
                let pr_req = GhPrRequest {
                    command: subcommand,
                    repo: req.repo,
                    number: req.number,
                    title: req.title,
                    body: req.body,
                    state: req.state,
                    limit: req.limit,
                    base: req.base,
                    head: req.head,
                    merge_method: req.merge_method,
                };
                self.gh_pr(Parameters(pr_req)).await
            }

            "search" => {
                let search_type = req.search_type.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "search_type is required for search command",
                        None::<serde_json::Value>,
                    )
                })?;
                let query = req.query.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "query is required for search command",
                        None::<serde_json::Value>,
                    )
                })?;
                let search_req = GhSearchRequest {
                    search_type,
                    query,
                    limit: req.limit,
                };
                self.gh_search(Parameters(search_req)).await
            }

            "release" => {
                let subcommand = req.subcommand.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "subcommand is required for release command",
                        None::<serde_json::Value>,
                    )
                })?;
                let release_req = GhReleaseRequest {
                    command: subcommand,
                    repo: req.repo,
                    tag: req.tag,
                    title: req.title,
                    notes: req.notes,
                    draft: req.draft,
                    prerelease: req.prerelease,
                };
                self.gh_release(Parameters(release_req)).await
            }

            "workflow" => {
                let subcommand = req.subcommand.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "subcommand is required for workflow command",
                        None::<serde_json::Value>,
                    )
                })?;
                let workflow_req = GhWorkflowRequest {
                    command: subcommand,
                    repo: req.repo,
                    workflow: req.workflow,
                    ref_branch: req.ref_branch,
                };
                self.gh_workflow(Parameters(workflow_req)).await
            }

            "run" => {
                let subcommand = req.subcommand.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "subcommand is required for run command",
                        None::<serde_json::Value>,
                    )
                })?;
                let run_req = GhRunRequest {
                    command: subcommand,
                    repo: req.repo,
                    run_id: req.run_id,
                    workflow: req.workflow,
                    status: req.status,
                    limit: req.limit,
                };
                self.gh_run(Parameters(run_req)).await
            }

            "api" => {
                let endpoint = req.endpoint.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "endpoint is required for api command",
                        None::<serde_json::Value>,
                    )
                })?;
                let api_req = GhApiRequest {
                    endpoint,
                    method: req.method,
                    body: req.body,
                    jq_filter: req.jq_filter,
                };
                self.gh_api(Parameters(api_req)).await
            }

            "auth_status" => {
                let auth_req = GhAuthStatusRequest {
                    hostname: req.hostname,
                };
                self.gh_auth_status(Parameters(auth_req)).await
            }

            "auth_login" => {
                let auth_req = GhAuthLoginRequest {
                    hostname: req.hostname,
                    token: req.token,
                };
                self.gh_auth_login(Parameters(auth_req)).await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!("Unknown github command: '{}'. Available: repo, issue, pr, search, release, workflow, run, api, auth_status, auth_login", req.command),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // GITLAB GROUPED TOOL
    // ========================================================================

    #[tool(
        name = "gitlab",
        description = "GitLab operations. Subcommands: issue, mr, pipeline, auth_status, auth_login"
    )]
    async fn gitlab_group(
        &self,
        Parameters(req): Parameters<GitLabGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "issue" => {
                let subcommand = req.subcommand.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "subcommand is required for issue command",
                        None::<serde_json::Value>,
                    )
                })?;
                let issue_req = GlabIssueRequest {
                    command: subcommand,
                    project: req.project,
                    iid: req.iid,
                    title: req.title,
                    description: req.description,
                    state: req.state,
                    per_page: req.per_page,
                    labels: req.labels,
                };
                self.glab_issue(Parameters(issue_req)).await
            }

            "mr" => {
                let subcommand = req.subcommand.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "subcommand is required for mr command",
                        None::<serde_json::Value>,
                    )
                })?;
                let mr_req = GlabMrRequest {
                    command: subcommand,
                    project: req.project,
                    iid: req.iid,
                    title: req.title,
                    description: req.description,
                    state: req.state,
                    per_page: req.per_page,
                    source_branch: req.source_branch,
                    target_branch: req.target_branch,
                };
                self.glab_mr(Parameters(mr_req)).await
            }

            "pipeline" => {
                let subcommand = req.subcommand.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "subcommand is required for pipeline command",
                        None::<serde_json::Value>,
                    )
                })?;
                let pipeline_req = GlabPipelineRequest {
                    command: subcommand,
                    project: req.project,
                    pipeline_id: req.pipeline_id,
                    ref_name: req.ref_name,
                    status: req.status,
                };
                self.glab_pipeline(Parameters(pipeline_req)).await
            }

            "auth_status" => {
                let auth_req = GlabAuthStatusRequest {
                    hostname: req.hostname,
                };
                self.glab_auth_status(Parameters(auth_req)).await
            }

            "auth_login" => {
                let auth_req = GlabAuthLoginRequest {
                    hostname: req.hostname,
                    token: req.token,
                };
                self.glab_auth_login(Parameters(auth_req)).await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!("Unknown gitlab command: '{}'. Available: issue, mr, pipeline, auth_status, auth_login", req.command),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // KUBERNETES GROUPED TOOL
    // ========================================================================

    #[tool(
        name = "kubernetes",
        description = "Kubernetes operations. Subcommands: get, describe, logs, apply, delete, exec, stern, helm, kustomize"
    )]
    async fn kubernetes_group(
        &self,
        Parameters(req): Parameters<KubernetesGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "get" => {
                let resource = req.resource.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "resource is required for get command",
                        None::<serde_json::Value>,
                    )
                })?;
                let get_req = KubectlGetRequest {
                    resource,
                    namespace: req.namespace,
                    name: req.name,
                    selector: req.selector,
                    all_namespaces: req.all_namespaces,
                    output: req.output,
                };
                self.kubectl_get(Parameters(get_req)).await
            }

            "describe" => {
                let resource = req.resource.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "resource is required for describe command",
                        None::<serde_json::Value>,
                    )
                })?;
                let name = req.name.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "name is required for describe command",
                        None::<serde_json::Value>,
                    )
                })?;
                let describe_req = KubectlDescribeRequest {
                    resource,
                    name,
                    namespace: req.namespace,
                };
                self.kubectl_describe(Parameters(describe_req)).await
            }

            "logs" => {
                let pod = req.name.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "name (pod) is required for logs command",
                        None::<serde_json::Value>,
                    )
                })?;
                let logs_req = KubectlLogsRequest {
                    pod,
                    namespace: req.namespace,
                    container: req.container,
                    tail: req.tail,
                    since: req.since,
                    previous: req.previous,
                    timestamps: req.timestamps,
                };
                self.kubectl_logs(Parameters(logs_req)).await
            }

            "apply" => {
                let manifest = req.manifest.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "manifest is required for apply command",
                        None::<serde_json::Value>,
                    )
                })?;
                let apply_req = KubectlApplyRequest {
                    manifest,
                    namespace: req.namespace,
                    dry_run: req.dry_run,
                };
                self.kubectl_apply(Parameters(apply_req)).await
            }

            "delete" => {
                let resource = req.resource.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "resource is required for delete command",
                        None::<serde_json::Value>,
                    )
                })?;
                let name = req.name.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "name is required for delete command",
                        None::<serde_json::Value>,
                    )
                })?;
                let delete_req = KubectlDeleteRequest {
                    resource,
                    name,
                    namespace: req.namespace,
                    force: req.force,
                };
                self.kubectl_delete(Parameters(delete_req)).await
            }

            "exec" => {
                let pod = req.name.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "name (pod) is required for exec command",
                        None::<serde_json::Value>,
                    )
                })?;
                let command = req.exec_command.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "exec_command is required for exec command",
                        None::<serde_json::Value>,
                    )
                })?;
                let exec_req = KubectlExecRequest {
                    pod,
                    command,
                    namespace: req.namespace,
                    container: req.container,
                };
                self.kubectl_exec(Parameters(exec_req)).await
            }

            "stern" => {
                let query = req.query.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "query is required for stern command",
                        None::<serde_json::Value>,
                    )
                })?;
                let stern_req = SternRequest {
                    query,
                    namespace: req.namespace,
                    container: req.container,
                    tail: req.tail,
                    since: req.since,
                    timestamps: req.timestamps,
                    output: req.stern_output,
                    selector: req.selector,
                };
                self.stern(Parameters(stern_req)).await
            }

            "helm" => {
                let helm_cmd = req.helm_command.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "helm_command is required for helm command",
                        None::<serde_json::Value>,
                    )
                })?;
                let helm_req = HelmRequest {
                    command: helm_cmd,
                    namespace: req.namespace,
                    release: req.release,
                    chart: req.chart,
                    values: req.values,
                    args: req.args,
                };
                self.helm(Parameters(helm_req)).await
            }

            "kustomize" => {
                let kustomize_cmd = req.kustomize_command.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "kustomize_command is required for kustomize command",
                        None::<serde_json::Value>,
                    )
                })?;
                let kustomize_req = KustomizeRequest {
                    command: kustomize_cmd,
                    path: req.kustomize_path,
                    output: req.kustomize_output,
                };
                self.kustomize(Parameters(kustomize_req)).await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!("Unknown kubernetes command: '{}'. Available: get, describe, logs, apply, delete, exec, stern, helm, kustomize", req.command),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // CONTAINER GROUPED TOOL
    // ========================================================================

    #[tool(
        name = "container",
        description = "Container operations. Subcommands: podman, dive, skopeo, crane, trivy"
    )]
    async fn container_group(
        &self,
        Parameters(req): Parameters<ContainerGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "podman" => {
                let podman_cmd = req.podman_command.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "podman_command is required for podman command",
                        None::<serde_json::Value>,
                    )
                })?;
                let podman_req = PodmanRequest {
                    command: podman_cmd,
                    target: req.target,
                    all: req.all,
                    args: req.args,
                };
                self.podman(Parameters(podman_req)).await
            }

            "dive" => {
                let image = req.image.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "image is required for dive command",
                        None::<serde_json::Value>,
                    )
                })?;
                let dive_req = DiveRequest {
                    image,
                    ci: req.ci,
                    json: req.json,
                };
                self.dive(Parameters(dive_req)).await
            }

            "skopeo" => {
                let skopeo_cmd = req.skopeo_command.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "skopeo_command is required for skopeo command",
                        None::<serde_json::Value>,
                    )
                })?;
                let source = req.source.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "source is required for skopeo command",
                        None::<serde_json::Value>,
                    )
                })?;
                let skopeo_req = SkopeoRequest {
                    command: skopeo_cmd,
                    source,
                    dest: req.dest,
                    insecure: req.insecure,
                };
                self.skopeo(Parameters(skopeo_req)).await
            }

            "crane" => {
                let crane_cmd = req.crane_command.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "crane_command is required for crane command",
                        None::<serde_json::Value>,
                    )
                })?;
                let image = req.target.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "target (image) is required for crane command",
                        None::<serde_json::Value>,
                    )
                })?;
                let crane_req = CraneRequest {
                    command: crane_cmd,
                    image,
                    args: req.args,
                };
                self.crane(Parameters(crane_req)).await
            }

            "trivy" => {
                let scan_type = req.scan_type.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "scan_type is required for trivy command",
                        None::<serde_json::Value>,
                    )
                })?;
                let target = req.target.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "target is required for trivy command",
                        None::<serde_json::Value>,
                    )
                })?;
                let trivy_req = TrivyRequest {
                    scan_type,
                    target,
                    format: req.format,
                    severity: req.severity,
                    ignore_unfixed: req.ignore_unfixed,
                };
                self.trivy(Parameters(trivy_req)).await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!("Unknown container command: '{}'. Available: podman, dive, skopeo, crane, trivy", req.command),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // NETWORK GROUPED TOOL
    // ========================================================================

    #[tool(
        name = "network",
        description = "Network operations. Subcommands: http (xh), sql (usql), dns (doggo)"
    )]
    async fn network_group(
        &self,
        Parameters(req): Parameters<NetworkGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "http" | "xh" => {
                let url = req.url.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "url is required for http command",
                        None::<serde_json::Value>,
                    )
                })?;
                let http_req = HttpRequest {
                    url,
                    method: req.method,
                    body: req.body,
                    headers: req.headers,
                    auth: req.auth,
                    bearer: req.bearer,
                    follow: req.follow,
                    timeout: req.timeout,
                    form: req.form,
                    json_output: req.json_output,
                    print: req.print,
                };
                self.http(Parameters(http_req)).await
            }

            "sql" | "usql" => {
                let url = req.db_url.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "db_url is required for sql command",
                        None::<serde_json::Value>,
                    )
                })?;
                let sql_req = UsqlRequest {
                    url,
                    command: req.sql_command,
                    format: req.format,
                };
                self.usql(Parameters(sql_req)).await
            }

            "dns" | "doggo" => {
                let domain = req.domain.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "domain is required for dns command",
                        None::<serde_json::Value>,
                    )
                })?;
                let dns_req = DnsRequest {
                    domain,
                    record_type: req.record_type,
                    server: req.server,
                    short: req.short,
                    json: req.json,
                };
                self.dns(Parameters(dns_req)).await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!(
                    "Unknown network command: '{}'. Available: http, sql, dns",
                    req.command
                ),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // SYSTEM GROUPED TOOL
    // ========================================================================

    #[tool(
        name = "system",
        description = "System operations. Subcommands: shell, nix_shell, benchmark, procs, info, bats"
    )]
    async fn system_group(
        &self,
        Parameters(req): Parameters<SystemGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "shell" | "exec" => {
                let command = req.exec_command.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "exec_command is required for shell command",
                        None::<serde_json::Value>,
                    )
                })?;
                let shell_req = ShellExecRequest {
                    command,
                    shell: req.shell,
                    working_dir: req.working_dir,
                    timeout: req.timeout,
                    env: req.env,
                };
                self.shell_exec(Parameters(shell_req)).await
            }

            "nix_shell" | "nix" => {
                let command = req.exec_command.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "exec_command is required for nix_shell command",
                        None::<serde_json::Value>,
                    )
                })?;
                let nix_req = NixShellExecRequest {
                    command,
                    flake: req.flake,
                    devshell: req.devshell,
                    shell: req.shell,
                    working_dir: req.working_dir,
                    timeout: req.timeout,
                };
                self.nix_shell_exec(Parameters(nix_req)).await
            }

            "benchmark" | "hyperfine" => {
                let command = req.benchmark_command.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "benchmark_command is required for benchmark command",
                        None::<serde_json::Value>,
                    )
                })?;
                let bench_req = HyperfineRequest {
                    command,
                    compare: req.compare,
                    warmup: req.warmup,
                    min_runs: req.min_runs,
                    json: req.json,
                };
                self.hyperfine(Parameters(bench_req)).await
            }

            "procs" | "ps" => {
                let procs_req = ProcsRequest {
                    keyword: req.keyword,
                    tree: req.tree,
                    sort: req.sort,
                };
                self.procs(Parameters(procs_req)).await
            }

            "info" | "sysinfo" => {
                self.system_info().await
            }

            "bats" | "test" => {
                let path = req.path.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "path is required for bats command",
                        None::<serde_json::Value>,
                    )
                })?;
                let bats_req = BatsRequest {
                    path,
                    filter: req.filter,
                    tap: req.tap,
                    count: req.count,
                };
                self.bats(Parameters(bats_req)).await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!("Unknown system command: '{}'. Available: shell, nix_shell, benchmark, procs, info, bats", req.command),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // ARCHIVE GROUPED TOOL
    // ========================================================================

    #[tool(
        name = "archive",
        description = "Archive operations. Subcommands: compress, decompress, list"
    )]
    async fn archive_group(
        &self,
        Parameters(req): Parameters<ArchiveGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "compress" | "pack" => {
                let files = req.files.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "files is required for compress command",
                        None::<serde_json::Value>,
                    )
                })?;
                let output = req.output.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "output is required for compress command",
                        None::<serde_json::Value>,
                    )
                })?;
                let compress_req = OuchCompressRequest { files, output };
                self.ouch_compress(Parameters(compress_req)).await
            }

            "decompress" | "unpack" | "extract" => {
                let archive = req.archive.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "archive is required for decompress command",
                        None::<serde_json::Value>,
                    )
                })?;
                let decompress_req = OuchDecompressRequest {
                    archive,
                    output_dir: req.output_dir,
                };
                self.ouch_decompress(Parameters(decompress_req)).await
            }

            "list" | "ls" => {
                let archive = req.archive.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "archive is required for list command",
                        None::<serde_json::Value>,
                    )
                })?;
                let list_req = OuchListRequest { archive };
                self.ouch_list(Parameters(list_req)).await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!(
                    "Unknown archive command: '{}'. Available: compress, decompress, list",
                    req.command
                ),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // REFERENCE GROUPED TOOL
    // ========================================================================

    #[tool(
        name = "reference",
        description = "Reference operations. Subcommands: tldr, cheat (navi), regex (grex)"
    )]
    async fn reference_group(
        &self,
        Parameters(req): Parameters<ReferenceGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "tldr" => {
                let command = req.cmd.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "cmd is required for tldr command",
                        None::<serde_json::Value>,
                    )
                })?;
                let tldr_req = TldrRequest {
                    command,
                    platform: req.platform,
                };
                self.tldr(Parameters(tldr_req)).await
            }

            "cheat" | "navi" => {
                let navi_req = NaviRequest {
                    query: req.query,
                    best_match: req.best_match,
                };
                self.navi(Parameters(navi_req)).await
            }

            "regex" | "grex" => {
                let input = req.input.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "input is required for regex command",
                        None::<serde_json::Value>,
                    )
                })?;
                let grex_req = GrexRequest {
                    input,
                    ignore_case: req.ignore_case,
                    escape: req.escape,
                    anchors: req.anchors,
                    verbose: req.verbose,
                    no_capture: req.no_capture,
                };
                self.grex(Parameters(grex_req)).await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!(
                    "Unknown reference command: '{}'. Available: tldr, cheat, regex",
                    req.command
                ),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // DIFF GROUPED TOOL
    // ========================================================================

    #[tool(
        name = "diff",
        description = "Diff operations. Subcommands: files (delta), structural (difftastic)"
    )]
    async fn diff_group(
        &self,
        Parameters(req): Parameters<DiffGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "files" | "delta" => {
                let file_a = req.file_a.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "file_a is required for files command",
                        None::<serde_json::Value>,
                    )
                })?;
                let file_b = req.file_b.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "file_b is required for files command",
                        None::<serde_json::Value>,
                    )
                })?;
                let delta_req = DeltaRequest { file_a, file_b };
                self.delta(Parameters(delta_req)).await
            }

            "structural" | "difft" | "difftastic" => {
                let left = req.file_a.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "file_a (left) is required for structural command",
                        None::<serde_json::Value>,
                    )
                })?;
                let right = req.file_b.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "file_b (right) is required for structural command",
                        None::<serde_json::Value>,
                    )
                })?;
                let difft_req = DifftasticRequest {
                    left,
                    right,
                    display: req.display,
                    language: req.language,
                    context: req.context,
                };
                self.difft(Parameters(difft_req)).await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!(
                    "Unknown diff command: '{}'. Available: files, structural",
                    req.command
                ),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // MCP GROUPED TOOL
    // ========================================================================

    #[tool(
        name = "mcp",
        description = "MCP state operations. Subcommands: cache_get, cache_set, task_create, task_update, task_list, task_delete, context_get, context_set, context_list, auth_check"
    )]
    async fn mcp_group(
        &self,
        Parameters(req): Parameters<McpGroupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match req.command.as_str() {
            "cache_get" => {
                let key = req.key.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "key is required for cache_get command",
                        None::<serde_json::Value>,
                    )
                })?;
                let cache_req = McpCacheGetRequest { key };
                self.mcp_cache_get(Parameters(cache_req)).await
            }

            "cache_set" => {
                let key = req.key.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "key is required for cache_set command",
                        None::<serde_json::Value>,
                    )
                })?;
                let value = req.value.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "value is required for cache_set command",
                        None::<serde_json::Value>,
                    )
                })?;
                let cache_req = McpCacheSetRequest {
                    key,
                    value,
                    ttl_secs: req.ttl_secs,
                };
                self.mcp_cache_set(Parameters(cache_req)).await
            }

            "task_create" => {
                let content = req.content.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "content is required for task_create command",
                        None::<serde_json::Value>,
                    )
                })?;
                let task_req = McpTaskCreateRequest { content };
                self.mcp_task_create(Parameters(task_req)).await
            }

            "task_update" => {
                let id = req.id.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "id is required for task_update command",
                        None::<serde_json::Value>,
                    )
                })?;
                let status = req.status.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "status is required for task_update command",
                        None::<serde_json::Value>,
                    )
                })?;
                let task_req = McpTaskUpdateRequest { id, status };
                self.mcp_task_update(Parameters(task_req)).await
            }

            "task_list" => {
                let task_req = McpTaskListRequest { status: req.status };
                self.mcp_task_list(Parameters(task_req)).await
            }

            "task_delete" => {
                let id = req.id.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "id is required for task_delete command",
                        None::<serde_json::Value>,
                    )
                })?;
                let task_req = McpTaskDeleteRequest { id };
                self.mcp_task_delete(Parameters(task_req)).await
            }

            "context_get" => {
                let key = req.key.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "key is required for context_get command",
                        None::<serde_json::Value>,
                    )
                })?;
                let ctx_req = McpContextGetRequest {
                    key,
                    scope: req.scope,
                };
                self.mcp_context_get(Parameters(ctx_req)).await
            }

            "context_set" => {
                let key = req.key.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "key is required for context_set command",
                        None::<serde_json::Value>,
                    )
                })?;
                let value = req.value.ok_or_else(|| {
                    ErrorData::new(
                        rmcp::model::ErrorCode::INVALID_PARAMS,
                        "value is required for context_set command",
                        None::<serde_json::Value>,
                    )
                })?;
                let ctx_req = McpContextSetRequest {
                    key,
                    value,
                    scope: req.scope,
                };
                self.mcp_context_set(Parameters(ctx_req)).await
            }

            "context_list" => {
                let ctx_req = McpContextListRequest { scope: req.scope };
                self.mcp_context_list(Parameters(ctx_req)).await
            }

            "auth_check" => {
                self.mcp_auth_check().await
            }

            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!("Unknown mcp command: '{}'. Available: cache_get, cache_set, task_create, task_update, task_list, task_delete, context_get, context_set, context_list, auth_check", req.command),
                None::<serde_json::Value>,
            )),
        }
    }

    // ========================================================================
    // SEARCH TOOLS
    // ========================================================================

    #[tool(
        name = "Search - Content (ripgrep)",
        description = "Search file contents with ripgrep (rg) - extremely fast grep replacement. \
        Features: regex, respects .agentignore, parallel search, many output formats."
    )]
    async fn rg(
        &self,
        Parameters(req): Parameters<RipgrepRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["--color=never".into()];

        // Add .agentignore support (disables .gitignore, uses only .agentignore)
        let working_dir = req.path.as_deref().unwrap_or(".");
        let ignore_args = self
            .ignore
            .get_ignore_file_args(std::path::Path::new(working_dir));
        args.extend(ignore_args);

        // Determine if JSON output is compatible with requested options
        let files_only = req.files_with_matches.unwrap_or(false);
        let count_only = req.count.unwrap_or(false);
        // JSON output is default for AI consumption, but incompatible with -l/-c
        let use_json = req.json.unwrap_or(!files_only && !count_only);

        if req.ignore_case.unwrap_or(false) {
            args.push("-i".into());
        }
        if req.smart_case.unwrap_or(false) {
            args.push("-S".into());
        }
        if req.hidden.unwrap_or(false) {
            args.push("--hidden".into());
        }
        // Note: no_ignore is deprecated - .agentignore is now the only ignore mechanism
        if files_only {
            args.push("-l".into());
        }
        if count_only {
            args.push("-c".into());
        }
        if !use_json && req.line_number.unwrap_or(true) {
            args.push("-n".into());
        }
        if req.word.unwrap_or(false) {
            args.push("-w".into());
        }
        if req.invert.unwrap_or(false) {
            args.push("-v".into());
        }
        if req.follow.unwrap_or(false) {
            args.push("-L".into());
        }
        if req.multiline.unwrap_or(false) {
            args.push("-U".into());
        }
        if req.only_matching.unwrap_or(false) {
            args.push("-o".into());
        }
        if req.fixed_strings.unwrap_or(false) {
            args.push("-F".into());
        }
        if use_json {
            args.push("--json".into());
        }
        if let Some(ctx) = req.context {
            args.push(format!("-C{}", ctx));
        }
        if let Some(ref ft) = req.file_type {
            args.push(format!("-t{}", ft));
        }
        if let Some(ref glob) = req.glob {
            args.push(format!("--glob={}", glob));
        }
        if let Some(max) = req.max_count {
            args.push(format!("-m{}", max));
        }
        if let Some(ref replace) = req.replace {
            args.push(format!("--replace={}", replace));
        }

        args.push(req.pattern.clone());
        if let Some(ref path) = req.path {
            args.push(path.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("rg", &args_ref).await {
            Ok(output) => {
                if use_json {
                    // Parse JSONL to JSON array for AI consumption
                    let mut lines: Vec<serde_json::Value> = output
                        .stdout
                        .lines()
                        .filter_map(|line| serde_json::from_str(line).ok())
                        .collect();
                    // Apply max_results limiting if specified
                    if let Some(max) = req.max_results {
                        lines.truncate(max as usize);
                    }
                    Ok(CallToolResult::success(vec![Content::text(
                        serde_json::to_string(&lines).unwrap_or_else(|_| "[]".to_string()),
                    )]))
                } else {
                    Ok(CallToolResult::success(vec![Content::text(
                        output.to_result_string(),
                    )]))
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Search - Fuzzy (fzf)",
        description = "Filter and fuzzy-find items with fzf. \
        Pass a list of items and a query to get fuzzy-matched results."
    )]
    async fn fzf_filter(
        &self,
        Parameters(req): Parameters<FzfFilterRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["--filter".into(), req.query.clone()];

        if req.exact.unwrap_or(false) {
            args.push("-e".into());
        }
        if req.ignore_case.unwrap_or(false) {
            args.push("-i".into());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_with_stdin("fzf", &args_ref, &req.input)
            .await
        {
            Ok(output) => {
                let result = if let Some(limit) = req.limit {
                    output
                        .stdout
                        .lines()
                        .take(limit as usize)
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    output.stdout.clone()
                };
                let json_output = parse_fzf_to_json(&result, &req.query);
                Ok(CallToolResult::success(vec![Content::text(json_output)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Search - AST (ast-grep)",
        description = "Search code with ast-grep (sg) - AST-based structural search. \
        Find code patterns semantically, not just textually."
    )]
    async fn ast_grep(
        &self,
        Parameters(req): Parameters<AstGrepRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        // Check .agentignore if path is specified
        if let Some(ref path_str) = req.path {
            let path = std::path::Path::new(path_str);
            if let Err(msg) = self.ignore.validate_path(path) {
                return Ok(CallToolResult::error(vec![Content::text(msg)]));
            }
        }

        let mut args: Vec<String> = vec![
            "run".into(),
            "--pattern".into(),
            req.pattern.clone(),
            "--json".into(), // Always output JSON
        ];

        if let Some(ref lang) = req.lang {
            args.push(format!("--lang={}", lang));
        }
        if let Some(ref rewrite) = req.rewrite {
            args.push(format!("--rewrite={}", rewrite));
        }
        if let Some(ref path) = req.path {
            args.push(path.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("sg", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_json_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // TEXT PROCESSING TOOLS
    // ========================================================================

    #[tool(
        name = "Text - Substitute (sd)",
        description = "Find and replace text with sd (modern sed replacement). \
        Simpler syntax than sed, supports regex and literal modes."
    )]
    async fn sd(
        &self,
        Parameters(req): Parameters<SdRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];
        if req.fixed.unwrap_or(false) {
            args.push("-s".into());
        }
        args.push(req.find.clone());
        args.push(req.replace.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_with_stdin("sd", &args_ref, &req.input)
            .await
        {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                text_to_json_envelope("sd", &output.stdout, output.success),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(
                text_to_json_envelope("sd", &e, false),
            )])),
        }
    }

    #[tool(
        name = "Text - JSON (jq)",
        description = "Process JSON with jq - the powerful command-line JSON processor. \
        Filter, transform, and query JSON data."
    )]
    async fn jq(
        &self,
        Parameters(req): Parameters<JqRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];

        if req.raw.unwrap_or(false) {
            args.push("-r".into());
        }
        if req.compact.unwrap_or(false) {
            args.push("-c".into());
        }
        if req.sort_keys.unwrap_or(false) {
            args.push("-S".into());
        }
        if req.slurp.unwrap_or(false) {
            args.push("-s".into());
        }
        args.push(req.filter.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_with_stdin("jq", &args_ref, &req.input)
            .await
        {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Text - YAML (yq)",
        description = "Process YAML/JSON/XML/CSV with yq - portable command-line processor. \
        Query and transform data in various formats."
    )]
    async fn yq(
        &self,
        Parameters(req): Parameters<YqRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];

        if req.prettyprint.unwrap_or(true) {
            args.push("-P".into());
        }
        if let Some(ref fmt) = req.output_format {
            args.push(format!("-o={}", fmt));
        }
        if let Some(ref fmt) = req.input_format {
            args.push(format!("-p={}", fmt));
        }
        args.push(req.expression.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_with_stdin("yq", &args_ref, &req.input)
            .await
        {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Text - CSV (xsv)",
        description = "Process CSV with xsv - fast CSV toolkit. \
        Commands: stats, select, search, sort, slice, frequency, count, headers."
    )]
    async fn xsv(
        &self,
        Parameters(req): Parameters<XsvRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![req.command.clone()];

        if req.no_headers.unwrap_or(false) {
            args.push("--no-headers".into());
        }
        if let Some(ref d) = req.delimiter {
            args.push(format!("-d{}", d));
        }
        if let Some(ref extra) = req.args {
            for arg in extra.split_whitespace() {
                args.push(arg.to_string());
            }
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_with_stdin("qsv", &args_ref, &req.input)
            .await
        {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Text - Cut (hck)",
        description = "Extract fields with hck (hack) - a faster cut replacement. \
        Extract columns from delimited text."
    )]
    async fn hck(
        &self,
        Parameters(req): Parameters<HckRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["-f".into(), req.fields.clone()];

        if let Some(ref d) = req.delimiter {
            args.push(format!("-d{}", d));
        }
        if let Some(ref d) = req.output_delimiter {
            args.push(format!("-D{}", d));
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_with_stdin("hck", &args_ref, &req.input)
            .await
        {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                text_to_json_envelope("hck", &output.stdout, output.success),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(
                text_to_json_envelope("hck", &e, false),
            )])),
        }
    }

    // ========================================================================
    // SYSTEM TOOLS
    // ========================================================================

    #[tool(
        name = "System - Processes (procs)",
        description = "List and filter processes with procs (modern ps replacement). \
        Features: tree view, sorting, filtering, colorful output."
    )]
    async fn procs(
        &self,
        Parameters(req): Parameters<ProcsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        // Use JSON output by default for AI consumption
        let mut args: Vec<String> = vec!["--color=never".into(), "--json".into()];

        if req.tree.unwrap_or(false) {
            args.push("--tree".into());
        }
        if let Some(ref sort) = req.sort {
            args.push(format!("--sortd={}", sort));
        }
        if let Some(ref keyword) = req.keyword {
            args.push(keyword.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("procs", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_json_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "System - Code Stats (tokei)",
        description = "Count lines of code with tokei (fast code statistics). \
        Recognizes 150+ languages, shows code, comments, blanks."
    )]
    async fn tokei(
        &self,
        Parameters(req): Parameters<TokeiRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];

        if req.files.unwrap_or(false) {
            args.push("--files".into());
        }
        if req.hidden.unwrap_or(false) {
            args.push("--hidden".into());
        }
        // Default to JSON output for AI consumption
        let output_format = req.output.as_deref().unwrap_or("json");
        args.push(format!("--output={}", output_format));
        if let Some(ref sort) = req.sort {
            args.push(format!("--sort={}", sort));
        }
        if let Some(ref exclude) = req.exclude {
            args.push(format!("--exclude={}", exclude));
        }
        if let Some(ref langs) = req.languages {
            args.push(format!("--types={}", langs));
        }
        if let Some(ref path) = req.path {
            args.push(path.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("tokei", &args_ref).await {
            Ok(output) => {
                // tokei JSON output is already valid JSON
                if output_format == "json" {
                    Ok(CallToolResult::success(vec![Content::text(
                        output.to_json_string(),
                    )]))
                } else {
                    Ok(CallToolResult::success(vec![Content::text(
                        output.to_result_string(),
                    )]))
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "System - Benchmark (hyperfine)",
        description = "Benchmark commands with hyperfine. \
        Precise timing with warmup, statistical analysis, comparison."
    )]
    async fn hyperfine(
        &self,
        Parameters(req): Parameters<HyperfineRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["--style=basic".into()];

        if req.json.unwrap_or(false) {
            args.push("--export-json=/dev/stdout".into());
        }
        if let Some(warmup) = req.warmup {
            args.push(format!("--warmup={}", warmup));
        }
        if let Some(min) = req.min_runs {
            args.push(format!("--min-runs={}", min));
        }
        args.push(req.command.clone());
        if let Some(ref compare) = req.compare {
            args.push(compare.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("hyperfine", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "System - Info",
        description = "Get system resource usage snapshot (memory, CPU, uptime). Returns JSON."
    )]
    async fn system_info(&self) -> Result<CallToolResult, ErrorData> {
        use sysinfo::System;

        let mut sys = System::new_all();
        sys.refresh_all();

        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let total_swap = sys.total_swap();
        let used_swap = sys.used_swap();

        // Calculate CPU usage
        let cpu_usage: f32 =
            sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;

        let result = serde_json::json!({
            "memory": {
                "total_bytes": total_mem,
                "used_bytes": used_mem,
                "available_bytes": total_mem.saturating_sub(used_mem),
                "total_gb": format!("{:.1}", total_mem as f64 / 1_073_741_824.0),
                "used_gb": format!("{:.1}", used_mem as f64 / 1_073_741_824.0),
                "usage_percent": format!("{:.1}", (used_mem as f64 / total_mem as f64) * 100.0)
            },
            "swap": {
                "total_bytes": total_swap,
                "used_bytes": used_swap,
                "total_gb": format!("{:.1}", total_swap as f64 / 1_073_741_824.0),
                "used_gb": format!("{:.1}", used_swap as f64 / 1_073_741_824.0)
            },
            "cpu": {
                "cores": sys.cpus().len(),
                "usage_percent": format!("{:.1}", cpu_usage),
                "name": sys.cpus().first().map(|c| c.brand()).unwrap_or("unknown")
            },
            "system": {
                "name": System::name().unwrap_or_default(),
                "kernel_version": System::kernel_version().unwrap_or_default(),
                "os_version": System::os_version().unwrap_or_default(),
                "host_name": System::host_name().unwrap_or_default(),
                "uptime_secs": System::uptime(),
                "uptime_human": format_uptime(System::uptime())
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    // ========================================================================
    // NETWORK TOOLS
    // ========================================================================

    #[tool(
        name = "Network - HTTP (xh)",
        description = "Make HTTP requests with xh (HTTPie-compatible). \
        Features: JSON by default, syntax highlighting, intuitive syntax."
    )]
    async fn http(
        &self,
        Parameters(req): Parameters<HttpRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["--style=plain".into()];

        let method = req.method.as_deref().unwrap_or("GET");
        args.push(method.into());
        args.push(req.url.clone());

        if req.follow.unwrap_or(true) {
            args.push("--follow".into());
        }
        if req.json_output.unwrap_or(false) {
            args.push("--json".into());
        }
        if req.form.unwrap_or(false) {
            args.push("--form".into());
        }
        if let Some(ref auth) = req.auth {
            args.push(format!("--auth={}", auth));
        }
        if let Some(ref bearer) = req.bearer {
            args.push(format!("--bearer={}", bearer));
        }
        if let Some(timeout) = req.timeout {
            args.push(format!("--timeout={}", timeout));
        }
        if let Some(ref print) = req.print {
            args.push(format!("--print={}", print));
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("xh", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Network - DNS (doggo)",
        description = "DNS lookup with doggo (modern dig replacement). \
        Features: colorful output, DNS over HTTPS/TLS, multiple record types."
    )]
    async fn dns(
        &self,
        Parameters(req): Parameters<DnsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![req.domain.clone()];

        if let Some(ref rt) = req.record_type {
            args.push(rt.clone());
        }
        if req.short.unwrap_or(false) {
            args.push("--short".into());
        }
        if req.json.unwrap_or(false) {
            args.push("--json".into());
        }
        if let Some(ref server) = req.server {
            args.push(format!("@{}", server));
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("doggo", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Network - SQL (usql)",
        description = "Execute SQL across multiple databases with usql. \
        Supports PostgreSQL, MySQL, SQLite, SQL Server, Oracle, and more."
    )]
    async fn usql(
        &self,
        Parameters(req): Parameters<UsqlRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![req.url.clone()];

        if let Some(ref fmt) = req.format {
            args.push(format!("--format={}", fmt));
        }
        if let Some(ref cmd) = req.command {
            args.push("-c".into());
            args.push(cmd.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("usql", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // WEB SEARCH TOOLS
    // ========================================================================

    #[tool(
        name = "Search - Web (DuckDuckGo)",
        description = "Search the web using DuckDuckGo. Returns JSON results with titles, URLs, and abstracts. \
        Use for finding documentation, code examples, or general information."
    )]
    async fn web_search(
        &self,
        Parameters(req): Parameters<WebSearchRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![
            "--json".into(),
            "--np".into(), // no prompt, exit after results
            "--nocolor".into(),
        ];

        if let Some(num) = req.num_results {
            args.push(format!("--num={}", num.min(25)));
        }
        if let Some(ref region) = req.region {
            args.push(format!("--reg={}", region));
        }
        if let Some(ref time) = req.time {
            args.push(format!("--time={}", time));
        }
        if let Some(ref site) = req.site {
            args.push(format!("--site={}", site));
        }
        if req.expand_urls.unwrap_or(false) {
            args.push("--expand".into());
        }

        // Add the search query
        args.push(req.query.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("ddgr", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // UTILITY TOOLS
    // ========================================================================

    #[tool(
        name = "Diff - Files (delta)",
        description = "View file differences with delta or diff. \
        Returns JSON with structured diff hunks. Features: line-by-line changes with context."
    )]
    async fn delta(
        &self,
        Parameters(req): Parameters<DeltaRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let diff_result = self
            .executor
            .run("diff", &["-u", &req.file_a, &req.file_b])
            .await;

        match diff_result {
            Ok(output) => {
                // Parse diff to JSON for structured output
                let json_output = parse_diff_to_json(&output.stdout, &req.file_a, &req.file_b);
                Ok(CallToolResult::success(vec![Content::text(json_output)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Git - Diff",
        description = "Git diff with syntax highlighting (uses delta if available)."
    )]
    async fn git_diff(
        &self,
        Parameters(req): Parameters<GitDiffRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["diff".into(), "--color=never".into()];

        if req.staged.unwrap_or(false) {
            args.push("--staged".into());
        }
        if let Some(ref commit) = req.commit {
            args.push(commit.clone());
        }
        if let Some(ref range) = req.range {
            args.push(range.clone());
        }
        if let Some(ref file) = req.file {
            args.push("--".into());
            args.push(file.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let path = req.path.as_deref();
        match self.executor.run_in_dir("git", &args_ref, path).await {
            Ok(output) => {
                match self
                    .executor
                    .run_with_stdin("delta", &["--color-only"], &output.stdout)
                    .await
                {
                    Ok(delta_output) => Ok(CallToolResult::success(vec![Content::text(
                        delta_output.to_result_string(),
                    )])),
                    Err(_) => Ok(CallToolResult::success(vec![Content::text(
                        output.to_result_string(),
                    )])),
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Test - Shell (bats)",
        description = "Run shell tests with bats (Bash Automated Testing System)."
    )]
    async fn bats(
        &self,
        Parameters(req): Parameters<BatsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];

        if req.tap.unwrap_or(false) {
            args.push("--tap".into());
        } else {
            args.push("--pretty".into());
        }
        if req.count.unwrap_or(false) {
            args.push("--count".into());
        }
        if let Some(ref filter) = req.filter {
            args.push(format!("--filter={}", filter));
        }
        args.push(req.path.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("bats", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Filesystem - File Type",
        description = "Detect file type using magic bytes with file command."
    )]
    async fn file_type(
        &self,
        Parameters(req): Parameters<FileTypeRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.executor.run("file", &["-b", &req.path]).await {
            Ok(output) => {
                let json_output = parse_file_to_json(&output.stdout, &req.path);
                Ok(CallToolResult::success(vec![Content::text(json_output)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Filesystem - Permissions",
        description = "Explain Unix file permissions in human readable format."
    )]
    async fn permissions(
        &self,
        Parameters(req): Parameters<PermissionsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mode = &req.mode;
        let result = if mode.chars().all(|c| c.is_ascii_digit()) {
            let octal: u32 = mode.parse().unwrap_or(0);
            let user = octal_to_rwx((octal >> 6) & 7);
            let group = octal_to_rwx((octal >> 3) & 7);
            let other = octal_to_rwx(octal & 7);
            format!(
                "Octal: {}\nSymbolic: {}{}{}\n\nUser: {}\nGroup: {}\nOther: {}",
                mode,
                user,
                group,
                other,
                describe_perms(&user),
                describe_perms(&group),
                describe_perms(&other)
            )
        } else {
            let octal = symbolic_to_octal(mode);
            format!("Symbolic: {}\nOctal: {:o}", mode, octal)
        };
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ========================================================================
    // AI-HELPFUL TOOLS
    // ========================================================================

    #[tool(
        name = "Reference - TLDR",
        description = "Get simplified command help with tldr (tealdeer). \
        Shows practical examples for common commands."
    )]
    async fn tldr(
        &self,
        Parameters(req): Parameters<TldrRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];

        if let Some(ref platform) = req.platform {
            args.push(format!("--platform={}", platform));
        }
        args.push(req.command.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("tldr", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Reference - Regex Generator (grex)",
        description = "Generate regex from test strings with grex. \
        Provide example strings and get a regex that matches them."
    )]
    async fn grex(
        &self,
        Parameters(req): Parameters<GrexRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];

        if req.escape.unwrap_or(false) {
            args.push("-e".into());
        }
        if req.ignore_case.unwrap_or(false) {
            args.push("-i".into());
        }
        if req.verbose.unwrap_or(false) {
            args.push("-x".into());
        }
        if req.no_capture.unwrap_or(false) {
            args.push("-c".into());
        }
        if req.anchors.unwrap_or(false) {
            args.push("-a".into());
        }

        // Split input by newlines or commas
        let inputs: Vec<&str> = req
            .input
            .split(['\n', ','])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for input in &inputs {
            args.push(input.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("grex", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                text_to_json_envelope("grex", &output.stdout, output.success),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(
                text_to_json_envelope("grex", &e, false),
            )])),
        }
    }

    #[tool(
        name = "Text - Find Replace (sad)",
        description = "Batch find and replace across files with sad. \
        Like sed but for multiple files with preview support."
    )]
    async fn sad(
        &self,
        Parameters(req): Parameters<SadRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];

        if req.preview.unwrap_or(true) {
            args.push("--preview".into());
        }
        if req.fixed.unwrap_or(false) {
            args.push("-f".into());
        }
        if req.ignore_case.unwrap_or(false) {
            args.push("-i".into());
        }

        args.push(req.pattern.clone());
        args.push(req.replace.clone());

        // Add file patterns
        for file in req.files.split_whitespace() {
            args.push(file.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("sad", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Diff - Structural (difft)",
        description = "Structural diff with difftastic (difft). \
        Understands code syntax for better diffs than line-based tools."
    )]
    async fn difft(
        &self,
        Parameters(req): Parameters<DifftasticRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["--color=never".into()];

        if let Some(ref display) = req.display {
            args.push(format!("--display={}", display));
        }
        if let Some(ref lang) = req.language {
            args.push(format!("--language={}", lang));
        }
        if let Some(ctx) = req.context {
            args.push(format!("--context={}", ctx));
        }

        args.push(req.left.clone());
        args.push(req.right.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("difft", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Archive - Compress (ouch)",
        description = "Compress files with ouch. \
        Supports many formats: tar.gz, zip, 7z, xz, bz2, zstd, etc."
    )]
    async fn ouch_compress(
        &self,
        Parameters(req): Parameters<OuchCompressRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["compress".into()];

        // Add input files
        for file in req.files.split(',') {
            args.push(file.trim().to_string());
        }

        args.push(req.output.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("ouch", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                if output.success {
                    format!("Compressed to: {}", req.output)
                } else {
                    output.to_result_string()
                },
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Archive - Decompress (ouch)",
        description = "Decompress archives with ouch. \
        Auto-detects format from file extension or magic bytes."
    )]
    async fn ouch_decompress(
        &self,
        Parameters(req): Parameters<OuchDecompressRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["decompress".into()];

        args.push(req.archive.clone());

        if let Some(ref dir) = req.output_dir {
            args.push("--dir".into());
            args.push(dir.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("ouch", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Archive - List (ouch)",
        description = "List archive contents with ouch."
    )]
    async fn ouch_list(
        &self,
        Parameters(req): Parameters<OuchListRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.executor.run("ouch", &["list", &req.archive]).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Task Queue - Add (pueue)",
        description = "Add task to pueue queue for background execution."
    )]
    async fn pueue_add(
        &self,
        Parameters(req): Parameters<PueueAddRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["add".into()];

        if req.immediate.unwrap_or(false) {
            args.push("--immediate".into());
        }
        if req.stashed.unwrap_or(false) {
            args.push("--stashed".into());
        }
        if let Some(ref label) = req.label {
            args.push("--label".into());
            args.push(label.clone());
        }
        if let Some(ref dir) = req.working_dir {
            args.push("--working-directory".into());
            args.push(dir.clone());
        }

        args.push("--".into());
        // Split command into parts
        for part in req.command.split_whitespace() {
            args.push(part.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("pueue", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Task Queue - Status (pueue)",
        description = "Get pueue task queue status."
    )]
    async fn pueue_status(
        &self,
        Parameters(req): Parameters<PueueStatusRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["status".into()];

        if req.json.unwrap_or(false) {
            args.push("--json".into());
        }
        if let Some(ref group) = req.group {
            args.push("--group".into());
            args.push(group.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("pueue", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Task Queue - Logs (pueue)",
        description = "Get logs from a pueue task."
    )]
    async fn pueue_log(
        &self,
        Parameters(req): Parameters<PueueLogRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["log".into()];

        if req.full.unwrap_or(false) {
            args.push("--full".into());
        }
        args.push(req.task_id.to_string());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("pueue", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Reference - Cheatsheets (navi)",
        description = "Search command cheatsheets with navi. \
        Interactive cheatsheet tool for command-line."
    )]
    async fn navi(
        &self,
        Parameters(req): Parameters<NaviRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];

        if req.best_match.unwrap_or(true) {
            args.push("--best-match".into());
        }

        if let Some(ref query) = req.query {
            args.push("--query".into());
            args.push(query.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("navi", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // GIT FORGE TOOLS (GitHub)
    // ========================================================================

    #[tool(
        name = "GitHub - Repo",
        description = "GitHub repository operations via gh CLI. \
        Returns JSON for structured data. Subcommands: list, view, clone, create, fork, delete."
    )]
    async fn gh_repo(
        &self,
        Parameters(req): Parameters<GhRepoRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["repo".into(), req.command.clone()];

        // JSON output for list/view
        if matches!(req.command.as_str(), "list" | "view") {
            args.push("--json".into());
            args.push(
                "name,owner,description,url,isPrivate,isFork,stargazerCount,forkCount,updatedAt"
                    .into(),
            );
        }

        if let Some(ref repo) = req.repo {
            args.push(repo.clone());
        }
        if let Some(ref extra) = req.args {
            for arg in extra.split_whitespace() {
                args.push(arg.to_string());
            }
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("gh", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "GitHub - Issue",
        description = "GitHub issue operations. Returns JSON. \
        Subcommands: list, view, create, close, reopen, edit, comment."
    )]
    async fn gh_issue(
        &self,
        Parameters(req): Parameters<GhIssueRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["issue".into(), req.command.clone()];

        // JSON output for list/view
        if matches!(req.command.as_str(), "list" | "view") {
            args.push("--json".into());
            args.push(
                "number,title,state,author,assignees,labels,body,createdAt,updatedAt,url".into(),
            );
        }

        if let Some(ref repo) = req.repo {
            args.push("-R".into());
            args.push(repo.clone());
        }
        if let Some(number) = req.number {
            args.push(number.to_string());
        }
        if let Some(ref title) = req.title {
            args.push("--title".into());
            args.push(title.clone());
        }
        if let Some(ref body) = req.body {
            args.push("--body".into());
            args.push(body.clone());
        }
        if let Some(ref labels) = req.labels {
            args.push("--label".into());
            args.push(labels.clone());
        }
        if let Some(ref assignees) = req.assignees {
            args.push("--assignee".into());
            args.push(assignees.clone());
        }
        if let Some(ref state) = req.state {
            args.push("--state".into());
            args.push(state.clone());
        }
        if let Some(limit) = req.limit {
            args.push("--limit".into());
            args.push(limit.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("gh", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "GitHub - Pull Request",
        description = "GitHub pull request operations. Returns JSON. \
        Subcommands: list, view, create, close, reopen, merge, checkout, diff, checks."
    )]
    async fn gh_pr(
        &self,
        Parameters(req): Parameters<GhPrRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["pr".into(), req.command.clone()];

        // JSON output for list/view/checks
        if matches!(req.command.as_str(), "list" | "view" | "checks") {
            args.push("--json".into());
            if req.command == "checks" {
                args.push("name,state,conclusion,startedAt,completedAt,detailsUrl".into());
            } else {
                args.push("number,title,state,author,headRefName,baseRefName,mergeable,additions,deletions,url,createdAt".into());
            }
        }

        if let Some(ref repo) = req.repo {
            args.push("-R".into());
            args.push(repo.clone());
        }
        if let Some(number) = req.number {
            args.push(number.to_string());
        }
        if let Some(ref title) = req.title {
            args.push("--title".into());
            args.push(title.clone());
        }
        if let Some(ref body) = req.body {
            args.push("--body".into());
            args.push(body.clone());
        }
        if let Some(ref base) = req.base {
            args.push("--base".into());
            args.push(base.clone());
        }
        if let Some(ref head) = req.head {
            args.push("--head".into());
            args.push(head.clone());
        }
        if let Some(ref state) = req.state {
            args.push("--state".into());
            args.push(state.clone());
        }
        if let Some(limit) = req.limit {
            args.push("--limit".into());
            args.push(limit.to_string());
        }
        if let Some(ref method) = req.merge_method {
            args.push(format!("--{}", method));
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("gh", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "GitHub - Search",
        description = "GitHub search across repos, issues, PRs, code, commits. Returns JSON."
    )]
    async fn gh_search(
        &self,
        Parameters(req): Parameters<GhSearchRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["search".into(), req.search_type.clone()];

        args.push(req.query.clone());
        args.push("--json".into());

        // Different fields based on search type
        match req.search_type.as_str() {
            "repos" => args.push("name,owner,description,stars,forks,url".into()),
            "issues" | "prs" => {
                args.push("number,title,state,author,repository,url,createdAt".into())
            }
            "code" => args.push("path,repository,textMatches".into()),
            "commits" => args.push("sha,message,author,repository,url".into()),
            _ => args.push("*".into()),
        }

        if let Some(limit) = req.limit {
            args.push("--limit".into());
            args.push(limit.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("gh", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "GitHub - Release",
        description = "GitHub release operations. Returns JSON for list/view."
    )]
    async fn gh_release(
        &self,
        Parameters(req): Parameters<GhReleaseRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["release".into(), req.command.clone()];

        if matches!(req.command.as_str(), "list" | "view") {
            args.push("--json".into());
            args.push("tagName,name,isDraft,isPrerelease,createdAt,publishedAt,url,assets".into());
        }

        if let Some(ref repo) = req.repo {
            args.push("-R".into());
            args.push(repo.clone());
        }
        if let Some(ref tag) = req.tag {
            args.push(tag.clone());
        }
        if let Some(ref title) = req.title {
            args.push("--title".into());
            args.push(title.clone());
        }
        if let Some(ref notes) = req.notes {
            args.push("--notes".into());
            args.push(notes.clone());
        }
        if req.draft.unwrap_or(false) {
            args.push("--draft".into());
        }
        if req.prerelease.unwrap_or(false) {
            args.push("--prerelease".into());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("gh", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "GitHub - Workflow",
        description = "GitHub Actions workflow operations. Returns JSON."
    )]
    async fn gh_workflow(
        &self,
        Parameters(req): Parameters<GhWorkflowRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["workflow".into(), req.command.clone()];

        if matches!(req.command.as_str(), "list" | "view") {
            args.push("--json".into());
            args.push("id,name,state,path".into());
        }

        if let Some(ref repo) = req.repo {
            args.push("-R".into());
            args.push(repo.clone());
        }
        if let Some(ref workflow) = req.workflow {
            args.push(workflow.clone());
        }
        if let Some(ref ref_branch) = req.ref_branch {
            args.push("--ref".into());
            args.push(ref_branch.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("gh", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "GitHub - Workflow Run",
        description = "GitHub Actions workflow run operations. Returns JSON."
    )]
    async fn gh_run(
        &self,
        Parameters(req): Parameters<GhRunRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["run".into(), req.command.clone()];

        if matches!(req.command.as_str(), "list" | "view") {
            args.push("--json".into());
            args.push(
                "databaseId,workflowName,status,conclusion,headBranch,event,createdAt,url".into(),
            );
        }

        if let Some(ref repo) = req.repo {
            args.push("-R".into());
            args.push(repo.clone());
        }
        if let Some(run_id) = req.run_id {
            args.push(run_id.to_string());
        }
        if let Some(ref workflow) = req.workflow {
            args.push("--workflow".into());
            args.push(workflow.clone());
        }
        if let Some(ref status) = req.status {
            args.push("--status".into());
            args.push(status.clone());
        }
        if let Some(limit) = req.limit {
            args.push("--limit".into());
            args.push(limit.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("gh", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "GitHub - API",
        description = "Direct GitHub API access. Returns JSON. \
        Supports any API endpoint with optional jq filtering."
    )]
    async fn gh_api(
        &self,
        Parameters(req): Parameters<GhApiRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["api".into()];

        if let Some(ref method) = req.method {
            args.push("-X".into());
            args.push(method.clone());
        }
        if let Some(ref body) = req.body {
            args.push("-f".into());
            args.push(body.clone());
        }
        if let Some(ref jq) = req.jq_filter {
            args.push("--jq".into());
            args.push(jq.clone());
        }

        args.push(req.endpoint.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("gh", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // GIT FORGE TOOLS (GitLab)
    // ========================================================================

    #[tool(
        name = "GitLab - Issue",
        description = "GitLab issue operations via glab CLI. \
        Subcommands: list, view, create, close, reopen."
    )]
    async fn glab_issue(
        &self,
        Parameters(req): Parameters<GlabIssueRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["issue".into(), req.command.clone()];

        // JSON output for list/view
        if matches!(req.command.as_str(), "list" | "view") {
            args.push("--output".into());
            args.push("json".into());
        }

        if let Some(ref project) = req.project {
            args.push("--repo".into());
            args.push(project.clone());
        }
        if let Some(iid) = req.iid {
            args.push(iid.to_string());
        }
        if let Some(ref title) = req.title {
            args.push("--title".into());
            args.push(title.clone());
        }
        if let Some(ref desc) = req.description {
            args.push("--description".into());
            args.push(desc.clone());
        }
        if let Some(ref labels) = req.labels {
            args.push("--label".into());
            args.push(labels.clone());
        }
        if let Some(ref state) = req.state {
            args.push("--state".into());
            args.push(state.clone());
        }
        if let Some(per_page) = req.per_page {
            args.push("--per-page".into());
            args.push(per_page.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("glab", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "GitLab - Merge Request",
        description = "GitLab merge request operations. \
        Subcommands: list, view, create, close, reopen, merge, approve."
    )]
    async fn glab_mr(
        &self,
        Parameters(req): Parameters<GlabMrRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["mr".into(), req.command.clone()];

        if matches!(req.command.as_str(), "list" | "view") {
            args.push("--output".into());
            args.push("json".into());
        }

        if let Some(ref project) = req.project {
            args.push("--repo".into());
            args.push(project.clone());
        }
        if let Some(iid) = req.iid {
            args.push(iid.to_string());
        }
        if let Some(ref title) = req.title {
            args.push("--title".into());
            args.push(title.clone());
        }
        if let Some(ref desc) = req.description {
            args.push("--description".into());
            args.push(desc.clone());
        }
        if let Some(ref source) = req.source_branch {
            args.push("--source-branch".into());
            args.push(source.clone());
        }
        if let Some(ref target) = req.target_branch {
            args.push("--target-branch".into());
            args.push(target.clone());
        }
        if let Some(ref state) = req.state {
            args.push("--state".into());
            args.push(state.clone());
        }
        if let Some(per_page) = req.per_page {
            args.push("--per-page".into());
            args.push(per_page.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("glab", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "GitLab - Pipeline",
        description = "GitLab CI/CD pipeline operations. \
        Subcommands: list, view, run, cancel, retry, delete."
    )]
    async fn glab_pipeline(
        &self,
        Parameters(req): Parameters<GlabPipelineRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["pipeline".into(), req.command.clone()];

        if matches!(req.command.as_str(), "list" | "view") {
            args.push("--output".into());
            args.push("json".into());
        }

        if let Some(ref project) = req.project {
            args.push("--repo".into());
            args.push(project.clone());
        }
        if let Some(pipeline_id) = req.pipeline_id {
            args.push(pipeline_id.to_string());
        }
        if let Some(ref ref_name) = req.ref_name {
            args.push("--ref".into());
            args.push(ref_name.clone());
        }
        if let Some(ref status) = req.status {
            args.push("--status".into());
            args.push(status.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("glab", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // DATA TRANSFORMATION TOOLS
    // ========================================================================

    #[tool(
        name = "Text - JSON Grep (gron)",
        description = "Transform JSON to greppable format with gron. \
        Makes JSON amenable to grep. Use ungron to convert back."
    )]
    async fn gron(
        &self,
        Parameters(req): Parameters<GronRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];

        if req.ungron.unwrap_or(false) {
            args.push("--ungron".into());
        }
        if req.stream.unwrap_or(false) {
            args.push("--stream".into());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_with_stdin("gron", &args_ref, &req.input)
            .await
        {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Text - HTML Query (htmlq)",
        description = "Query HTML with CSS selectors using htmlq (jq for HTML). \
        Extract elements, attributes, or text content."
    )]
    async fn htmlq(
        &self,
        Parameters(req): Parameters<HtmlqRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];

        if req.text.unwrap_or(false) {
            args.push("--text".into());
        }
        if let Some(ref attr) = req.attribute {
            args.push("--attribute".into());
            args.push(attr.clone());
        }

        args.push(req.selector.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_with_stdin("htmlq", &args_ref, &req.input)
            .await
        {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                text_to_json_envelope("htmlq", &output.stdout, output.success),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(
                text_to_json_envelope("htmlq", &e, false),
            )])),
        }
    }

    #[tool(
        name = "Text - HTML Parse (pup)",
        description = "Parse HTML with CSS selectors using pup. \
        Supports display filters like 'a attr{href}' or 'div text{}'."
    )]
    async fn pup(
        &self,
        Parameters(req): Parameters<PupRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];

        if req.json.unwrap_or(true) {
            args.push("--json".into());
        }

        args.push(req.selector.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_with_stdin("pup", &args_ref, &req.input)
            .await
        {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Text - Data Process (miller)",
        description = "Process structured data with miller (mlr). \
        Like awk but for CSV, JSON, and other formats. Returns JSON by default."
    )]
    async fn miller(
        &self,
        Parameters(req): Parameters<MillerRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let input_fmt = req.input_format.as_deref().unwrap_or("json");
        let output_fmt = req.output_format.as_deref().unwrap_or("json");

        let mut args: Vec<String> = vec![
            format!("--i{}", input_fmt),
            format!("--o{}", output_fmt),
            req.verb.clone(),
        ];

        if let Some(ref extra) = req.args {
            for arg in extra.split_whitespace() {
                args.push(arg.to_string());
            }
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_with_stdin("mlr", &args_ref, &req.input)
            .await
        {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Text - Universal (dasel)",
        description = "Query JSON/YAML/TOML/XML with dasel (universal selector). \
        Single tool for multiple data formats. Returns JSON by default."
    )]
    async fn dasel(
        &self,
        Parameters(req): Parameters<DaselRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let input_fmt = req.input_format.as_deref().unwrap_or("json");
        let output_fmt = req.output_format.as_deref().unwrap_or("json");

        let mut args: Vec<String> = vec![
            format!("-p{}", input_fmt),
            format!("-w{}", output_fmt),
            req.selector.clone(),
        ];

        if req.compact.unwrap_or(false) {
            args.push("-c".into());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_with_stdin("dasel", &args_ref, &req.input)
            .await
        {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // CONTAINER TOOLS
    // ========================================================================

    #[tool(
        name = "Container - Podman",
        description = "Podman container operations. Returns JSON for inspect, ps, images. \
        Subcommands: ps, images, inspect, logs, pull, run, stop, rm, rmi, build."
    )]
    async fn podman(
        &self,
        Parameters(req): Parameters<PodmanRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![req.command.clone()];

        // JSON output for inspection commands
        if matches!(req.command.as_str(), "inspect" | "ps" | "images") {
            args.push("--format=json".into());
        }

        if req.all.unwrap_or(false) && matches!(req.command.as_str(), "ps" | "images") {
            args.push("-a".into());
        }

        if let Some(ref target) = req.target {
            args.push(target.clone());
        }
        if let Some(ref extra) = req.args {
            for arg in extra.split_whitespace() {
                args.push(arg.to_string());
            }
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("podman", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Container - Image Analyze (dive)",
        description = "Analyze container image layers with dive. \
        CI mode returns efficiency score. JSON mode exports full analysis."
    )]
    async fn dive(
        &self,
        Parameters(req): Parameters<DiveRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];

        if req.ci.unwrap_or(true) {
            args.push("--ci".into());
        }
        if req.json.unwrap_or(false) {
            args.push("--json".into());
            args.push("/dev/stdout".into());
        }

        args.push(req.image.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("dive", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Container - Registry (skopeo)",
        description = "Container registry operations with skopeo. \
        Inspect images without pulling, copy between registries. Returns JSON for inspect."
    )]
    async fn skopeo(
        &self,
        Parameters(req): Parameters<SkopeoRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![req.command.clone()];

        if req.insecure.unwrap_or(false) {
            args.push("--tls-verify=false".into());
        }

        args.push(req.source.clone());

        if let Some(ref dest) = req.dest {
            args.push(dest.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("skopeo", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Container - Registry Low-level (crane)",
        description = "Low-level container registry operations with crane. \
        Get digests, manifests, configs, list tags. Returns JSON."
    )]
    async fn crane(
        &self,
        Parameters(req): Parameters<CraneRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![req.command.clone()];

        args.push(req.image.clone());

        if let Some(ref extra) = req.args {
            for arg in extra.split_whitespace() {
                args.push(arg.to_string());
            }
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("crane", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Security - Scan (trivy)",
        description = "Security vulnerability scanner with trivy. \
        Scan images, filesystems, repos, configs. Returns JSON by default."
    )]
    async fn trivy(
        &self,
        Parameters(req): Parameters<TrivyRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let format = req.format.as_deref().unwrap_or("json");

        let mut args: Vec<String> = vec![
            req.scan_type.clone(),
            "--format".into(),
            format.into(),
            "--quiet".into(),
        ];

        if let Some(ref severity) = req.severity {
            args.push("--severity".into());
            args.push(severity.clone());
        }
        if req.ignore_unfixed.unwrap_or(false) {
            args.push("--ignore-unfixed".into());
        }

        args.push(req.target.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("trivy", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Container - Compose",
        description = "Multi-container orchestration. Supports both podman-compose (default, rootless) \
        and docker compose (v2). Manage services defined in docker-compose.yml files. \
        Subcommands: up, down, ps, logs, build, pull, restart, stop, start."
    )]
    async fn compose(
        &self,
        Parameters(req): Parameters<ComposeRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let runtime = req.runtime.as_deref().unwrap_or("podman");
        let use_docker = runtime == "docker";

        let mut args: Vec<String> = vec![];

        // For docker, we use `docker compose` (v2 plugin)
        if use_docker {
            args.push("compose".into());
        }

        if let Some(ref file) = req.file {
            args.push("-f".into());
            args.push(file.clone());
        }

        args.push(req.command.clone());

        match req.command.as_str() {
            "up" => {
                if req.detach.unwrap_or(true) {
                    args.push("-d".into());
                }
            }
            "down" => {
                if req.volumes.unwrap_or(false) {
                    args.push("-v".into());
                }
            }
            "logs" => {
                if req.follow.unwrap_or(false) {
                    args.push("-f".into());
                }
                if let Some(tail) = req.tail {
                    args.push("--tail".into());
                    args.push(tail.to_string());
                }
            }
            _ => {}
        }

        if let Some(ref services) = req.services {
            for svc in services.split_whitespace() {
                args.push(svc.to_string());
            }
        }

        if let Some(ref extra) = req.args {
            for arg in extra.split_whitespace() {
                args.push(arg.to_string());
            }
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let cmd = if use_docker {
            "docker"
        } else {
            "podman-compose"
        };
        match self.executor.run(cmd, &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Container - Buildx",
        description = "Multi-platform container builds with docker buildx. \
        Build for multiple architectures, manage builders. \
        Subcommands: build, imagetools, inspect, ls, create, use, rm."
    )]
    async fn buildx(
        &self,
        Parameters(req): Parameters<BuildxRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["buildx".into(), req.command.clone()];

        if let Some(ref builder) = req.builder {
            args.push("--builder".into());
            args.push(builder.clone());
        }

        if req.command == "build" {
            if let Some(ref platform) = req.platform {
                args.push("--platform".into());
                args.push(platform.clone());
            }
            if let Some(ref tags) = req.tags {
                for tag in tags.split(',') {
                    args.push("-t".into());
                    args.push(tag.trim().to_string());
                }
            }
            if let Some(ref file) = req.file {
                args.push("-f".into());
                args.push(file.clone());
            }
            if req.push.unwrap_or(false) {
                args.push("--push".into());
            }
            if req.load.unwrap_or(false) {
                args.push("--load".into());
            }
            if let Some(ref build_args) = req.build_args {
                for ba in build_args.split(',') {
                    args.push("--build-arg".into());
                    args.push(ba.trim().to_string());
                }
            }
        }

        if let Some(ref target) = req.target {
            args.push(target.clone());
        }

        if let Some(ref extra) = req.args {
            for arg in extra.split_whitespace() {
                args.push(arg.to_string());
            }
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("docker", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Container - Build (buildah)",
        description = "OCI container image builder with buildah. \
        Build images without daemon, fine-grained control. \
        Subcommands: from, run, copy, add, commit, push, pull, images, containers, rm, rmi, build."
    )]
    async fn buildah(
        &self,
        Parameters(req): Parameters<BuildahRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![req.command.clone()];

        // JSON output for listing commands
        if matches!(req.command.as_str(), "images" | "containers")
            && req.format.as_deref() == Some("json")
        {
            args.push("--json".into());
        }

        match req.command.as_str() {
            "from" | "pull" => {
                if let Some(ref target) = req.target {
                    args.push(target.clone());
                }
            }
            "run" => {
                if let Some(ref target) = req.target {
                    args.push(target.clone());
                }
                if let Some(ref cmd) = req.run_command {
                    args.push("--".into());
                    for part in cmd.split_whitespace() {
                        args.push(part.to_string());
                    }
                }
            }
            "copy" | "add" => {
                if let Some(ref target) = req.target {
                    args.push(target.clone());
                }
                if let Some(ref source) = req.source {
                    args.push(source.clone());
                }
                if let Some(ref dest) = req.dest {
                    args.push(dest.clone());
                }
            }
            "commit" => {
                if let Some(ref target) = req.target {
                    args.push(target.clone());
                }
                if let Some(ref tag) = req.tag {
                    args.push(tag.clone());
                }
            }
            "push" => {
                if let Some(ref target) = req.target {
                    args.push(target.clone());
                }
            }
            "build" => {
                if let Some(ref file) = req.file {
                    args.push("-f".into());
                    args.push(file.clone());
                }
                if let Some(ref tag) = req.tag {
                    args.push("-t".into());
                    args.push(tag.clone());
                }
                if let Some(ref target) = req.target {
                    args.push(target.clone());
                } else {
                    args.push(".".into());
                }
            }
            "rm" | "rmi" => {
                if let Some(ref target) = req.target {
                    args.push(target.clone());
                }
            }
            _ => {
                if let Some(ref target) = req.target {
                    args.push(target.clone());
                }
            }
        }

        if let Some(ref extra) = req.args {
            for arg in extra.split_whitespace() {
                args.push(arg.to_string());
            }
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("buildah", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // KUBERNETES TOOLS
    // ========================================================================

    #[tool(
        name = "Kubernetes - Get",
        description = "Get Kubernetes resources. Returns JSON by default for AI parsing. \
        Resources: pods, deployments, services, configmaps, secrets, nodes, events."
    )]
    async fn kubectl_get(
        &self,
        Parameters(req): Parameters<KubectlGetRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let output_fmt = req.output.as_deref().unwrap_or("json");

        let mut args: Vec<String> = vec!["get".into(), req.resource.clone()];

        args.push("-o".into());
        args.push(output_fmt.into());

        if let Some(ref name) = req.name {
            args.push(name.clone());
        }
        if let Some(ref ns) = req.namespace {
            args.push("-n".into());
            args.push(ns.clone());
        }
        if req.all_namespaces.unwrap_or(false) {
            args.push("-A".into());
        }
        if let Some(ref selector) = req.selector {
            args.push("-l".into());
            args.push(selector.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("kubectl", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Kubernetes - Describe",
        description = "Describe Kubernetes resource in detail. Returns human-readable text."
    )]
    async fn kubectl_describe(
        &self,
        Parameters(req): Parameters<KubectlDescribeRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["describe".into(), req.resource.clone(), req.name.clone()];

        if let Some(ref ns) = req.namespace {
            args.push("-n".into());
            args.push(ns.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("kubectl", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Kubernetes - Logs",
        description = "Get logs from a Kubernetes pod."
    )]
    async fn kubectl_logs(
        &self,
        Parameters(req): Parameters<KubectlLogsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["logs".into(), req.pod.clone()];

        if let Some(ref container) = req.container {
            args.push("-c".into());
            args.push(container.clone());
        }
        if let Some(ref ns) = req.namespace {
            args.push("-n".into());
            args.push(ns.clone());
        }
        if let Some(tail) = req.tail {
            args.push("--tail".into());
            args.push(tail.to_string());
        }
        if req.previous.unwrap_or(false) {
            args.push("--previous".into());
        }
        if let Some(ref since) = req.since {
            args.push("--since".into());
            args.push(since.clone());
        }
        if req.timestamps.unwrap_or(false) {
            args.push("--timestamps".into());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("kubectl", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Kubernetes - Apply",
        description = "Apply Kubernetes manifest. Supports dry-run modes. \
        Pass YAML/JSON content directly."
    )]
    async fn kubectl_apply(
        &self,
        Parameters(req): Parameters<KubectlApplyRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["apply".into(), "-f".into(), "-".into()];

        if let Some(ref ns) = req.namespace {
            args.push("-n".into());
            args.push(ns.clone());
        }
        if let Some(ref dry_run) = req.dry_run {
            args.push(format!("--dry-run={}", dry_run));
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_with_stdin("kubectl", &args_ref, &req.manifest)
            .await
        {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Kubernetes - Delete",
        description = "Delete a Kubernetes resource."
    )]
    async fn kubectl_delete(
        &self,
        Parameters(req): Parameters<KubectlDeleteRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["delete".into(), req.resource.clone(), req.name.clone()];

        if let Some(ref ns) = req.namespace {
            args.push("-n".into());
            args.push(ns.clone());
        }
        if req.force.unwrap_or(false) {
            args.push("--force".into());
            args.push("--grace-period=0".into());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("kubectl", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Kubernetes - Exec",
        description = "Execute command in a Kubernetes pod container."
    )]
    async fn kubectl_exec(
        &self,
        Parameters(req): Parameters<KubectlExecRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["exec".into(), req.pod.clone()];

        if let Some(ref container) = req.container {
            args.push("-c".into());
            args.push(container.clone());
        }
        if let Some(ref ns) = req.namespace {
            args.push("-n".into());
            args.push(ns.clone());
        }

        args.push("--".into());
        for part in req.command.split_whitespace() {
            args.push(part.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("kubectl", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Kubernetes - Multi-Logs (stern)",
        description = "Multi-pod log tailing with stern. \
        Aggregates logs from multiple pods matching a query. JSON output available."
    )]
    async fn stern(
        &self,
        Parameters(req): Parameters<SternRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let output_fmt = req.output.as_deref().unwrap_or("json");

        let mut args: Vec<String> = vec![
            req.query.clone(),
            "--output".into(),
            output_fmt.into(),
            "--no-follow".into(), // Non-interactive for MCP
        ];

        if let Some(ref ns) = req.namespace {
            args.push("--namespace".into());
            args.push(ns.clone());
        }
        if let Some(ref container) = req.container {
            args.push("--container".into());
            args.push(container.clone());
        }
        if let Some(ref since) = req.since {
            args.push("--since".into());
            args.push(since.clone());
        }
        if let Some(ref selector) = req.selector {
            args.push("--selector".into());
            args.push(selector.clone());
        }
        if req.timestamps.unwrap_or(false) {
            args.push("--timestamps".into());
        }
        if let Some(tail) = req.tail {
            args.push("--tail".into());
            args.push(tail.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("stern", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Kubernetes - Helm",
        description = "Helm chart operations. Returns JSON for list/status. \
        Subcommands: list, status, get, install, upgrade, uninstall, search, show, repo."
    )]
    async fn helm(
        &self,
        Parameters(req): Parameters<HelmRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![req.command.clone()];

        // JSON output for list/status
        if matches!(req.command.as_str(), "list" | "status") {
            args.push("-o".into());
            args.push("json".into());
        }

        if let Some(ref release) = req.release {
            args.push(release.clone());
        }
        if let Some(ref chart) = req.chart {
            args.push(chart.clone());
        }
        if let Some(ref ns) = req.namespace {
            args.push("-n".into());
            args.push(ns.clone());
        }
        if let Some(ref values) = req.values {
            args.push("-f".into());
            args.push(values.clone());
        }
        if let Some(ref extra) = req.args {
            for arg in extra.split_whitespace() {
                args.push(arg.to_string());
            }
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("helm", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Kubernetes - Kustomize",
        description = "Build and manage Kubernetes manifests with kustomize. \
        Build outputs YAML/JSON for applying to cluster."
    )]
    async fn kustomize(
        &self,
        Parameters(req): Parameters<KustomizeRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![req.command.clone()];

        if let Some(ref path) = req.path {
            args.push(path.clone());
        }
        if let Some(ref output) = req.output {
            args.push("-o".into());
            args.push(output.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("kustomize", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // SHELL EXECUTION TOOLS
    // ========================================================================

    #[tool(
        name = "Shell - Execute",
        description = "Execute command in a shell. Supports bash, zsh, fish, nushell (nu), dash. \
        Returns stdout/stderr with exit code. Use for running arbitrary commands."
    )]
    async fn shell_exec(
        &self,
        Parameters(req): Parameters<ShellExecRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let shell = req.shell.as_deref().unwrap_or("bash");
        let shell_cmd = match shell {
            "bash" => "bash",
            "zsh" => "zsh",
            "fish" => "fish",
            "nu" | "nushell" => "nu",
            "dash" => "dash",
            _ => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Unknown shell: {}. Supported: bash, zsh, fish, nu, dash",
                    shell
                ))]))
            }
        };

        let timeout = req.timeout.unwrap_or(30).min(300);

        let env_vars: Option<std::collections::HashMap<String, String>> =
            req.env.as_ref().and_then(|e| serde_json::from_str(e).ok());

        let opts = ExecOptions {
            working_dir: req.working_dir.as_deref(),
            timeout_secs: Some(timeout),
            env: env_vars.as_ref(),
            clear_env: false,
        };

        let args = vec!["-c", &req.command];
        match self.executor.run_with_options(shell_cmd, &args, opts).await {
            Ok(output) => {
                let result = serde_json::json!({
                    "success": output.success,
                    "exit_code": output.exit_code,
                    "stdout": output.stdout,
                    "stderr": output.stderr,
                    "shell": shell_cmd
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Shell - Nix",
        description = "Execute command in a Nix devshell. Provides access to all tools defined in the \
        flake's devShell. Useful for running commands with specific development dependencies."
    )]
    async fn nix_shell_exec(
        &self,
        Parameters(req): Parameters<NixShellExecRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let flake = req.flake.as_deref().unwrap_or(".");
        let inner_shell = req.shell.as_deref().unwrap_or("bash");

        let inner_shell_cmd = match inner_shell {
            "bash" => "bash",
            "zsh" => "zsh",
            "fish" => "fish",
            "nu" | "nushell" => "nu",
            _ => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Unknown inner shell: {}. Supported: bash, zsh, fish, nu",
                    inner_shell
                ))]))
            }
        };

        let timeout = req.timeout.unwrap_or(120).min(600);

        let flake_ref = match &req.devshell {
            Some(name) => format!("{}#{}", flake, name),
            None => flake.to_string(),
        };

        let opts = ExecOptions {
            working_dir: req.working_dir.as_deref(),
            timeout_secs: Some(timeout),
            env: None,
            clear_env: false,
        };

        let args = vec![
            "develop",
            &flake_ref,
            "-c",
            inner_shell_cmd,
            "-c",
            &req.command,
        ];

        match self.executor.run_with_options("nix", &args, opts).await {
            Ok(output) => {
                let result = serde_json::json!({
                    "success": output.success,
                    "exit_code": output.exit_code,
                    "stdout": output.stdout,
                    "stderr": output.stderr,
                    "flake": flake_ref,
                    "shell": inner_shell_cmd
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // GIT FORGE AUTH TOOLS
    // ========================================================================

    #[tool(
        name = "GitHub - Auth Status",
        description = "Check GitHub CLI authentication status. Returns auth state for the host."
    )]
    async fn gh_auth_status(
        &self,
        Parameters(req): Parameters<GhAuthStatusRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args = vec!["auth", "status"];

        let hostname = req.hostname.unwrap_or_else(|| "github.com".to_string());
        args.push("-h");
        let hostname_ref: &str = &hostname;
        let args_with_host: Vec<&str> = {
            let mut v = args.clone();
            v.push(hostname_ref);
            v
        };

        match self.executor.run("gh", &args_with_host).await {
            Ok(output) => {
                let authenticated = output.success;
                let result = serde_json::json!({
                    "authenticated": authenticated,
                    "hostname": hostname,
                    "output": if authenticated { &output.stdout } else { &output.stderr },
                    "message": if authenticated {
                        "GitHub CLI is authenticated"
                    } else {
                        "GitHub CLI is not authenticated. Use gh_auth_login to authenticate."
                    }
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "GitHub - Auth Login",
        description = "Authenticate GitHub CLI. Supports token-based auth (non-interactive) or \
        returns instructions for interactive web auth."
    )]
    async fn gh_auth_login(
        &self,
        Parameters(req): Parameters<GhAuthLoginRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let hostname = req.hostname.unwrap_or_else(|| "github.com".to_string());

        // Token-based auth (non-interactive)
        if let Some(token) = &req.token {
            let args = vec!["auth", "login", "-h", &hostname, "--with-token"];
            match self.executor.run_with_stdin("gh", &args, token).await {
                Ok(output) => {
                    let result = serde_json::json!({
                        "success": output.success,
                        "hostname": hostname,
                        "method": "token",
                        "message": if output.success {
                            "Successfully authenticated with token"
                        } else {
                            "Failed to authenticate with token"
                        },
                        "output": if output.success { &output.stdout } else { &output.stderr }
                    });
                    Ok(CallToolResult::success(vec![Content::text(
                        result.to_string(),
                    )]))
                }
                Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
            }
        } else {
            // Interactive auth - return instructions
            let result = serde_json::json!({
                "success": false,
                "hostname": hostname,
                "method": "interactive",
                "message": "Interactive authentication required. Run in terminal:",
                "command": format!("gh auth login -h {}", hostname),
                "instructions": [
                    "1. Run the command above in your terminal",
                    "2. Select authentication method (HTTPS recommended)",
                    "3. Authenticate via browser when prompted",
                    "4. Return here and retry your operation"
                ]
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
    }

    #[tool(
        name = "GitLab - Auth Status",
        description = "Check GitLab CLI authentication status. Returns auth state for the host."
    )]
    async fn glab_auth_status(
        &self,
        Parameters(req): Parameters<GlabAuthStatusRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let hostname = req.hostname.unwrap_or_else(|| "gitlab.com".to_string());
        let args = vec!["auth", "status", "-h", &hostname];

        match self.executor.run("glab", &args).await {
            Ok(output) => {
                let authenticated = output.success;
                let result = serde_json::json!({
                    "authenticated": authenticated,
                    "hostname": hostname,
                    "output": if authenticated { &output.stdout } else { &output.stderr },
                    "message": if authenticated {
                        "GitLab CLI is authenticated"
                    } else {
                        "GitLab CLI is not authenticated. Use glab_auth_login to authenticate."
                    }
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "GitLab - Auth Login",
        description = "Authenticate GitLab CLI. Supports token-based auth (non-interactive) or \
        returns instructions for interactive auth."
    )]
    async fn glab_auth_login(
        &self,
        Parameters(req): Parameters<GlabAuthLoginRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let hostname = req.hostname.unwrap_or_else(|| "gitlab.com".to_string());

        // Token-based auth (non-interactive)
        if let Some(token) = &req.token {
            let args = vec!["auth", "login", "-h", &hostname, "-t", token];
            match self.executor.run("glab", &args).await {
                Ok(output) => {
                    let result = serde_json::json!({
                        "success": output.success,
                        "hostname": hostname,
                        "method": "token",
                        "message": if output.success {
                            "Successfully authenticated with token"
                        } else {
                            "Failed to authenticate with token"
                        },
                        "output": if output.success { &output.stdout } else { &output.stderr }
                    });
                    Ok(CallToolResult::success(vec![Content::text(
                        result.to_string(),
                    )]))
                }
                Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
            }
        } else {
            // Interactive auth - return instructions
            let result = serde_json::json!({
                "success": false,
                "hostname": hostname,
                "method": "interactive",
                "message": "Interactive authentication required. Run in terminal:",
                "command": format!("glab auth login -h {}", hostname),
                "instructions": [
                    "1. Run the command above in your terminal",
                    "2. Enter your GitLab personal access token when prompted",
                    "3. Or select browser authentication",
                    "4. Return here and retry your operation"
                ]
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
    }

    // ========================================================================
    // GIT PRIMITIVE TOOLS
    // ========================================================================

    #[tool(
        name = "Git - Status",
        description = "Get git repository status. Shows staged, unstaged, and untracked files."
    )]
    async fn git_status(
        &self,
        Parameters(req): Parameters<GitStatusRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["status".into()];

        // Use porcelain v2 by default for machine-parseable JSON output
        let use_json = !req.short.unwrap_or(false);
        if req.porcelain.unwrap_or(use_json) {
            args.push("--porcelain=v2".into());
            args.push("--branch".into());
        } else if req.short.unwrap_or(false) {
            args.push("-s".into());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_in_dir("git", &args_ref, req.path.as_deref())
            .await
        {
            Ok(output) => {
                if use_json {
                    // Parse porcelain v2 format to JSON
                    let json = parse_git_status_porcelain_v2(&output.stdout);
                    let json_str =
                        serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string());
                    let summary = format::format_git_status_summary(&json_str);
                    Ok(self.build_response(&summary, &json_str, "data://git/status.json"))
                } else {
                    let raw = output.to_result_string();
                    let summary = format::format_generic_summary(
                        "git status",
                        output.success,
                        raw.lines().count(),
                    );
                    Ok(self.build_response(&summary, &raw, "data://git/status.txt"))
                }
            }
            Err(e) => Ok(self.build_error(&e)),
        }
    }

    #[tool(
        name = "Git - Add",
        description = "Stage files for commit. Add specific files or all changes."
    )]
    async fn git_add(
        &self,
        Parameters(req): Parameters<GitAddRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["add".into()];

        if req.all.unwrap_or(false) {
            args.push("-A".into());
        }

        // Parse files - support space-separated paths
        for file in req.files.split_whitespace() {
            args.push(file.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_in_dir("git", &args_ref, req.path.as_deref())
            .await
        {
            Ok(output) => {
                let result = serde_json::json!({
                    "success": output.exit_code == Some(0),
                    "files": req.files,
                    "output": output.to_result_string()
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Git - Commit",
        description = "Create a git commit with a message."
    )]
    async fn git_commit(
        &self,
        Parameters(req): Parameters<GitCommitRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["commit".into()];

        if req.all.unwrap_or(false) {
            args.push("-a".into());
        }

        if req.amend.unwrap_or(false) {
            args.push("--amend".into());
        }

        args.push("-m".into());
        args.push(req.message.clone());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_in_dir("git", &args_ref, req.path.as_deref())
            .await
        {
            Ok(output) => {
                let result = serde_json::json!({
                    "success": output.exit_code == Some(0),
                    "message": req.message,
                    "output": output.to_result_string()
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Git - Branch",
        description = "Manage git branches. List, create, delete, or rename branches."
    )]
    async fn git_branch(
        &self,
        Parameters(req): Parameters<GitBranchRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["branch".into()];

        match req.command.as_str() {
            "list" => {
                args.push("-a".into()); // Show all branches
            }
            "create" => {
                if let Some(name) = &req.name {
                    args.push(name.clone());
                } else {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "Branch name required for create",
                    )]));
                }
            }
            "delete" => {
                if req.force.unwrap_or(false) {
                    args.push("-D".into());
                } else {
                    args.push("-d".into());
                }
                if let Some(name) = &req.name {
                    args.push(name.clone());
                } else {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "Branch name required for delete",
                    )]));
                }
            }
            "rename" => {
                args.push("-m".into());
                if let Some(name) = &req.name {
                    args.push(name.clone());
                } else {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "Current branch name required for rename",
                    )]));
                }
                if let Some(new_name) = &req.new_name {
                    args.push(new_name.clone());
                } else {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "New branch name required for rename",
                    )]));
                }
            }
            _ => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Unknown command: {}. Use: list, create, delete, rename",
                    req.command
                ))]))
            }
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_in_dir("git", &args_ref, req.path.as_deref())
            .await
        {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Git - Checkout",
        description = "Switch branches or restore working tree files."
    )]
    async fn git_checkout(
        &self,
        Parameters(req): Parameters<GitCheckoutRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["checkout".into()];

        if req.create.unwrap_or(false) {
            args.push("-b".into());
        }

        args.push(req.target.clone());

        // Specific files to checkout
        if let Some(files) = &req.files {
            args.push("--".into());
            for file in files.split_whitespace() {
                args.push(file.to_string());
            }
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_in_dir("git", &args_ref, req.path.as_deref())
            .await
        {
            Ok(output) => {
                let result = serde_json::json!({
                    "success": output.exit_code == Some(0),
                    "target": req.target,
                    "output": output.to_result_string()
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(name = "Git - Log", description = "Show git commit history.")]
    async fn git_log(
        &self,
        Parameters(req): Parameters<GitLogRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["log".into()];

        if let Some(count) = req.count {
            args.push(format!("-{}", count));
        }

        // Use JSON output by default unless custom format requested
        let use_json = req.format.is_none() && !req.oneline.unwrap_or(false);
        if req.oneline.unwrap_or(false) {
            args.push("--oneline".into());
        } else if let Some(format) = &req.format {
            args.push(format!("--format={}", format));
        } else {
            // JSON-friendly format with delimiters
            args.push(
                "--format=<COMMIT>%H<SEP>%h<SEP>%an<SEP>%ae<SEP>%ai<SEP>%s<SEP>%b<END>".into(),
            );
        }

        if let Some(file) = &req.file {
            args.push("--".into());
            args.push(file.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_in_dir("git", &args_ref, req.path.as_deref())
            .await
        {
            Ok(output) => {
                if use_json {
                    // Parse custom format to JSON
                    let commits: Vec<serde_json::Value> = output
                        .stdout
                        .split("<COMMIT>")
                        .filter(|s| !s.is_empty())
                        .filter_map(|commit| {
                            let commit = commit.trim_end_matches("<END>").trim();
                            let parts: Vec<&str> = commit.splitn(7, "<SEP>").collect();
                            if parts.len() >= 6 {
                                Some(serde_json::json!({
                                    "hash": parts[0],
                                    "short_hash": parts[1],
                                    "author": parts[2],
                                    "email": parts[3],
                                    "date": parts[4],
                                    "subject": parts[5],
                                    "body": parts.get(6).unwrap_or(&"").trim(),
                                }))
                            } else {
                                None
                            }
                        })
                        .collect();
                    Ok(CallToolResult::success(vec![Content::text(
                        serde_json::to_string(&commits).unwrap_or_else(|_| "[]".to_string()),
                    )]))
                } else {
                    Ok(CallToolResult::success(vec![Content::text(
                        output.to_result_string(),
                    )]))
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "Git - Stash",
        description = "Stash changes. Push, pop, list, drop, apply, or show stashed changes."
    )]
    async fn git_stash(
        &self,
        Parameters(req): Parameters<GitStashRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["stash".into()];

        match req.command.as_str() {
            "push" => {
                args.push("push".into());
                if let Some(msg) = &req.message {
                    args.push("-m".into());
                    args.push(msg.clone());
                }
            }
            "pop" => {
                args.push("pop".into());
                if let Some(idx) = req.index {
                    args.push(format!("stash@{{{}}}", idx));
                }
            }
            "list" => {
                args.push("list".into());
            }
            "drop" => {
                args.push("drop".into());
                if let Some(idx) = req.index {
                    args.push(format!("stash@{{{}}}", idx));
                }
            }
            "apply" => {
                args.push("apply".into());
                if let Some(idx) = req.index {
                    args.push(format!("stash@{{{}}}", idx));
                }
            }
            "show" => {
                args.push("show".into());
                args.push("-p".into()); // Show as patch
                if let Some(idx) = req.index {
                    args.push(format!("stash@{{{}}}", idx));
                }
            }
            _ => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Unknown command: {}. Use: push, pop, list, drop, apply, show",
                    req.command
                ))]))
            }
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self
            .executor
            .run_in_dir("git", &args_ref, req.path.as_deref())
            .await
        {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // CODE INTELLIGENCE TOOLS
    // ========================================================================

    #[tool(
        name = "Code - Symbols",
        description = "List code symbols (functions, classes, structs, etc.) in a file or directory. \
        Uses ast-grep for language-aware parsing."
    )]
    async fn symbols(
        &self,
        Parameters(req): Parameters<SymbolsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        // Detect language from path if not specified
        let lang = req.language.clone().unwrap_or_else(|| {
            let path = std::path::Path::new(&req.path);
            match path.extension().and_then(|e| e.to_str()) {
                Some("rs") => "rust".to_string(),
                Some("py") => "python".to_string(),
                Some("js") => "javascript".to_string(),
                Some("ts") | Some("tsx") => "typescript".to_string(),
                Some("go") => "go".to_string(),
                Some("java") => "java".to_string(),
                Some("c") | Some("h") => "c".to_string(),
                Some("cpp") | Some("cc") | Some("hpp") => "cpp".to_string(),
                _ => "rust".to_string(), // Default
            }
        });

        // Language-specific patterns for ast-grep
        let patterns: Vec<&str> = match lang.as_str() {
            "rust" => vec![
                "pub fn $NAME($$$)",
                "fn $NAME($$$)",
                "pub struct $NAME",
                "struct $NAME",
                "pub enum $NAME",
                "enum $NAME",
                "pub trait $NAME",
                "trait $NAME",
                "impl $NAME",
            ],
            "python" => vec!["def $NAME($$$)", "class $NAME"],
            "javascript" | "typescript" => vec![
                "function $NAME($$$)",
                "class $NAME",
                "const $NAME =",
                "let $NAME =",
            ],
            "go" => vec![
                "func $NAME($$$)",
                "type $NAME struct",
                "type $NAME interface",
            ],
            _ => vec!["fn $NAME", "struct $NAME", "class $NAME"],
        };

        let mut all_results = Vec::new();

        for pattern in patterns {
            let args = vec!["--pattern", pattern, "--lang", &lang, "--json", &req.path];
            match self.executor.run("sg", &args).await {
                Ok(output) => {
                    if output.exit_code == Some(0) && !output.stdout.is_empty() {
                        // Parse JSON output
                        if let Ok(matches) =
                            serde_json::from_str::<Vec<serde_json::Value>>(&output.stdout)
                        {
                            for m in matches {
                                let text = m.get("text").and_then(|v| v.as_str()).unwrap_or("");
                                let file = m.get("file").and_then(|v| v.as_str()).unwrap_or("");
                                let line = m
                                    .get("range")
                                    .and_then(|r| r.get("start"))
                                    .and_then(|s| s.get("line"))
                                    .and_then(|l| l.as_u64())
                                    .unwrap_or(0);

                                // Apply filter if specified
                                if let Some(filter) = &req.pattern {
                                    if !text.to_lowercase().contains(&filter.to_lowercase()) {
                                        continue;
                                    }
                                }

                                all_results.push(serde_json::json!({
                                    "text": text.lines().next().unwrap_or(text),
                                    "file": file,
                                    "line": line + 1,
                                    "pattern": pattern
                                }));
                            }
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        // Sort by file then line
        all_results.sort_by(|a, b| {
            let file_a = a.get("file").and_then(|v| v.as_str()).unwrap_or("");
            let file_b = b.get("file").and_then(|v| v.as_str()).unwrap_or("");
            let line_a = a.get("line").and_then(|v| v.as_u64()).unwrap_or(0);
            let line_b = b.get("line").and_then(|v| v.as_u64()).unwrap_or(0);
            (file_a, line_a).cmp(&(file_b, line_b))
        });

        let result = serde_json::json!({
            "path": req.path,
            "language": lang,
            "count": all_results.len(),
            "symbols": all_results
        });

        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    #[tool(
        name = "Code - References",
        description = "Find all references to a symbol across the codebase. \
        Uses ripgrep for fast text search with word boundaries."
    )]
    async fn references(
        &self,
        Parameters(req): Parameters<ReferencesRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let search_path = req.path.as_deref().unwrap_or(".");

        // Build ripgrep args for word-boundary search
        let pattern = format!(r"\b{}\b", regex::escape(&req.symbol));
        let mut args = vec![
            "--json",
            "-n", // Line numbers
            &pattern,
            search_path,
        ];

        // Add file type filter if language specified
        let type_flag;
        if let Some(lang) = &req.language {
            type_flag = match lang.as_str() {
                "rust" => "rust",
                "python" => "py",
                "javascript" => "js",
                "typescript" => "ts",
                "go" => "go",
                _ => "",
            };
            if !type_flag.is_empty() {
                args.insert(0, type_flag);
                args.insert(0, "-t");
            }
        }

        match self.executor.run("rg", &args).await {
            Ok(output) => {
                let mut references = Vec::new();

                // Parse JSON lines output from ripgrep
                for line in output.stdout.lines() {
                    if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) {
                        if obj.get("type").and_then(|t| t.as_str()) == Some("match") {
                            if let Some(data) = obj.get("data") {
                                let path = data
                                    .get("path")
                                    .and_then(|p| p.get("text"))
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("");
                                let line_num = data
                                    .get("line_number")
                                    .and_then(|l| l.as_u64())
                                    .unwrap_or(0);
                                let text = data
                                    .get("lines")
                                    .and_then(|l| l.get("text"))
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("");

                                references.push(serde_json::json!({
                                    "file": path,
                                    "line": line_num,
                                    "text": text.trim()
                                }));
                            }
                        }
                    }
                }

                let result = serde_json::json!({
                    "symbol": req.symbol,
                    "path": search_path,
                    "count": references.len(),
                    "references": references
                });

                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Search failed: {}",
                e
            ))])),
        }
    }

    // ========================================================================
    // FILE OPERATION TOOLS
    // ========================================================================

    #[tool(
        name = "File - Read",
        description = "Read file contents. Returns raw text with optional line offset/limit. \
        Supports any text file format."
    )]
    async fn file_read(
        &self,
        Parameters(req): Parameters<FileReadRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use tokio::fs;

        let path = std::path::Path::new(&req.path);

        if !path.is_absolute() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Path must be absolute",
            )]));
        }

        // Check .agentignore
        if let Err(msg) = self.ignore.validate_path(path) {
            return Ok(CallToolResult::error(vec![Content::text(msg)]));
        }

        match fs::read_to_string(path).await {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let total_lines = lines.len();

                let offset = req.offset.unwrap_or(1).saturating_sub(1);
                let limit = req.limit.unwrap_or(lines.len());

                let selected: Vec<String> = lines
                    .into_iter()
                    .skip(offset)
                    .take(limit)
                    .enumerate()
                    .map(|(i, line)| format!("{:6}\t{}", offset + i + 1, line))
                    .collect();

                let result = serde_json::json!({
                    "path": req.path,
                    "total_lines": total_lines,
                    "offset": offset + 1,
                    "lines_returned": selected.len(),
                    "content": selected.join("\n")
                });

                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to read file: {}",
                e
            ))])),
        }
    }

    #[tool(
        name = "File - Write",
        description = "Write content to a file. Creates file if it doesn't exist, overwrites if it does. \
        Use safe_overwrite=true to backup existing file to graveyard first."
    )]
    async fn file_write(
        &self,
        Parameters(req): Parameters<FileWriteRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use tokio::fs;

        let path = std::path::Path::new(&req.path);
        let mut graveyarded = false;

        if !path.is_absolute() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Path must be absolute",
            )]));
        }

        // Check .agentignore
        if let Err(msg) = self.ignore.validate_path(path) {
            return Ok(CallToolResult::error(vec![Content::text(msg)]));
        }

        // Safe overwrite: if file exists and safe_overwrite is true, rip it first
        if req.safe_overwrite.unwrap_or(false) && path.exists() {
            let mut rip_args: Vec<String> = vec![];
            if let Some(graveyard) = &req.graveyard {
                rip_args.push(format!("--graveyard={}", graveyard));
            }
            rip_args.push(req.path.clone());

            let args_ref: Vec<&str> = rip_args.iter().map(|s| s.as_str()).collect();
            match self.executor.run("rip", &args_ref).await {
                Ok(output) if output.success => {
                    graveyarded = true;
                }
                Ok(output) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to backup file to graveyard: {}",
                        output.to_result_string()
                    ))]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to backup file to graveyard: {}",
                        e
                    ))]));
                }
            }
        }

        if req.create_dirs.unwrap_or(false) {
            if let Some(parent) = path.parent() {
                if let Err(e) = fs::create_dir_all(parent).await {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to create directories: {}",
                        e
                    ))]));
                }
            }
        }

        match fs::write(path, &req.content).await {
            Ok(()) => {
                let result = serde_json::json!({
                    "success": true,
                    "path": req.path,
                    "bytes_written": req.content.len(),
                    "graveyarded_original": graveyarded
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to write file: {}",
                e
            ))])),
        }
    }

    #[tool(
        name = "File - Edit",
        description = "Edit file(s) by replacing text. Supports batch edits across multiple space-separated paths. \
        The old_text must be unique in each file unless replace_all is true. Use backup=true to save originals to graveyard."
    )]
    async fn file_edit(
        &self,
        Parameters(req): Parameters<FileEditRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use tokio::fs;

        let paths: Vec<&str> = req.path.split_whitespace().collect();
        let do_backup = req.backup.unwrap_or(false);
        let replace_all = req.replace_all.unwrap_or(false);
        let mut results = Vec::new();

        for path_str in &paths {
            let path = std::path::Path::new(path_str);
            let mut file_result = serde_json::json!({
                "path": path_str,
                "success": false
            });

            // Validate path
            if !path.is_absolute() {
                file_result["error"] = "Path must be absolute".into();
                results.push(file_result);
                continue;
            }

            // Check .agentignore
            if let Err(msg) = self.ignore.validate_path(path) {
                file_result["error"] = msg.into();
                results.push(file_result);
                continue;
            }

            // Backup if requested
            let mut backed_up = false;
            if do_backup && path.exists() {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let backup_path = if let Some(graveyard) = &req.graveyard {
                    let filename = path.file_name().unwrap_or_default().to_string_lossy();
                    format!("{}/{}.{}", graveyard, filename, timestamp)
                } else {
                    format!("{}.bak.{}", path_str, timestamp)
                };

                match fs::copy(path, &backup_path).await {
                    Ok(_) => backed_up = true,
                    Err(e) => {
                        file_result["error"] = format!("Backup failed: {}", e).into();
                        results.push(file_result);
                        continue;
                    }
                }
            }

            // Read file
            let content = match fs::read_to_string(path).await {
                Ok(c) => c,
                Err(e) => {
                    file_result["error"] = format!("Read failed: {}", e).into();
                    results.push(file_result);
                    continue;
                }
            };

            // Count and validate occurrences
            let occurrences = content.matches(&req.old_text).count();

            if occurrences == 0 {
                file_result["error"] = "old_text not found".into();
                results.push(file_result);
                continue;
            }

            if occurrences > 1 && !replace_all {
                file_result["error"] =
                    format!("old_text found {} times, use replace_all=true", occurrences).into();
                results.push(file_result);
                continue;
            }

            // Apply replacement
            let new_content = content.replace(&req.old_text, &req.new_text);

            match fs::write(path, &new_content).await {
                Ok(()) => {
                    file_result["success"] = true.into();
                    file_result["replacements"] = occurrences.into();
                    file_result["backed_up"] = backed_up.into();
                }
                Err(e) => {
                    file_result["error"] = format!("Write failed: {}", e).into();
                }
            }

            results.push(file_result);
        }

        let success_count = results
            .iter()
            .filter(|r| r["success"].as_bool() == Some(true))
            .count();
        let failed_count = results
            .iter()
            .filter(|r| r["success"].as_bool() == Some(false))
            .count();

        let response = if paths.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            serde_json::json!({
                "edited": success_count,
                "failed": failed_count,
                "results": results
            })
        };

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    #[tool(
        name = "File - Append",
        description = "Append content to a file. Creates file if it doesn't exist."
    )]
    async fn file_append(
        &self,
        Parameters(req): Parameters<FileAppendRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use tokio::fs::OpenOptions;
        use tokio::io::AsyncWriteExt;

        let path = std::path::Path::new(&req.path);

        if !path.is_absolute() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Path must be absolute",
            )]));
        }

        // Check .agentignore
        if let Err(msg) = self.ignore.validate_path(path) {
            return Ok(CallToolResult::error(vec![Content::text(msg)]));
        }

        let mut file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
        {
            Ok(f) => f,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to open file: {}",
                    e
                ))]))
            }
        };

        match file.write_all(req.content.as_bytes()).await {
            Ok(()) => {
                let result = serde_json::json!({
                    "success": true,
                    "path": req.path,
                    "bytes_appended": req.content.len()
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to append to file: {}",
                e
            ))])),
        }
    }

    #[tool(
        name = "File - Patch",
        description = "Apply a unified diff patch to a file."
    )]
    async fn file_patch(
        &self,
        Parameters(req): Parameters<FilePatchRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use std::io::Write;
        use tempfile::NamedTempFile;
        use tokio::fs;

        let path = std::path::Path::new(&req.path);
        let mut backed_up = false;

        if !path.is_absolute() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Path must be absolute",
            )]));
        }

        // Check .agentignore
        if let Err(msg) = self.ignore.validate_path(path) {
            return Ok(CallToolResult::error(vec![Content::text(msg)]));
        }

        if !path.exists() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "File not found: {}",
                req.path
            ))]));
        }

        // Backup: if backup is true, copy to backup location before patching
        if req.backup.unwrap_or(false) {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let backup_path = if let Some(graveyard) = &req.graveyard {
                let filename = path.file_name().unwrap_or_default().to_string_lossy();
                format!("{}/{}.{}", graveyard, filename, timestamp)
            } else {
                format!("{}.bak.{}", req.path, timestamp)
            };

            match fs::copy(path, &backup_path).await {
                Ok(_) => {
                    backed_up = true;
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to backup file: {}",
                        e
                    ))]));
                }
            }
        }

        // Write patch to temp file
        let mut patch_file = match NamedTempFile::new() {
            Ok(f) => f,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to create temp file: {}",
                    e
                ))]))
            }
        };

        if let Err(e) = patch_file.write_all(req.patch.as_bytes()) {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to write patch: {}",
                e
            ))]));
        }

        let patch_path = patch_file.path().to_string_lossy().to_string();

        // Run patch command
        let args = vec!["-u", "--input", &patch_path, &req.path];
        match self.executor.run("patch", &args).await {
            Ok(output) => {
                let result = serde_json::json!({
                    "success": output.exit_code == Some(0),
                    "path": req.path,
                    "backed_up": backed_up,
                    "output": output.to_result_string()
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Patch failed: {}",
                e
            ))])),
        }
    }

    // ========================================================================
    // FILESYSTEM OPERATION TOOLS
    // ========================================================================

    #[tool(
        name = "Filesystem - Mkdir",
        description = "Create a directory. Creates parent directories by default."
    )]
    async fn fs_mkdir(
        &self,
        Parameters(req): Parameters<FsMkdirRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use tokio::fs;

        let paths: Vec<&str> = req.path.split_whitespace().collect();
        let create_parents = req.parents.unwrap_or(true);

        let mut results = Vec::new();

        for path_str in &paths {
            let path = std::path::Path::new(path_str);

            let result = if create_parents {
                fs::create_dir_all(path).await
            } else {
                fs::create_dir(path).await
            };

            match result {
                Ok(()) => {
                    results.push(serde_json::json!({
                        "path": path_str,
                        "success": true
                    }));
                }
                Err(e) => {
                    results.push(serde_json::json!({
                        "path": path_str,
                        "success": false,
                        "error": e.to_string()
                    }));
                }
            }
        }

        let response = serde_json::json!({
            "created": results.iter().filter(|r| r["success"].as_bool() == Some(true)).count(),
            "failed": results.iter().filter(|r| r["success"].as_bool() == Some(false)).count(),
            "results": results
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    #[tool(
        name = "Filesystem - Copy",
        description = "Copy file(s) or directory(s). Supports multiple space-separated sources. Use safe_overwrite=true to backup dest to graveyard before overwriting."
    )]
    async fn fs_copy(
        &self,
        Parameters(req): Parameters<FsCopyRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use tokio::fs;

        // Parse multiple sources (space-separated)
        let sources: Vec<&str> = req.source.split_whitespace().collect();
        let dest = std::path::Path::new(&req.dest);

        // Validate dest
        if let Err(msg) = self.ignore.validate_path(dest) {
            return Ok(CallToolResult::error(vec![Content::text(msg)]));
        }

        // For multiple sources, dest must be an existing directory
        if sources.len() > 1 {
            if !dest.is_dir() {
                return Ok(CallToolResult::error(vec![Content::text(
                    "Multiple sources specified but destination is not a directory",
                )]));
            }
        }

        let mut results: Vec<serde_json::Value> = vec![];
        let mut graveyarded = false;

        for src_str in &sources {
            let source = std::path::Path::new(src_str);

            // Check .agentignore
            if let Err(msg) = self.ignore.validate_path(source) {
                results.push(serde_json::json!({
                    "source": src_str,
                    "success": false,
                    "error": msg
                }));
                continue;
            }

            // Determine actual destination
            let actual_dest = if dest.is_dir() {
                dest.join(source.file_name().unwrap_or_default())
            } else {
                dest.to_path_buf()
            };

            // Safe overwrite: backup existing dest to graveyard
            if req.safe_overwrite.unwrap_or(false) && actual_dest.exists() {
                let mut rip_args: Vec<String> = vec![];
                if let Some(graveyard) = &req.graveyard {
                    rip_args.push(format!("--graveyard={}", graveyard));
                }
                rip_args.push(actual_dest.to_string_lossy().to_string());

                let args_ref: Vec<&str> = rip_args.iter().map(|s| s.as_str()).collect();
                match self.executor.run("rip", &args_ref).await {
                    Ok(output) if output.success => {
                        graveyarded = true;
                    }
                    Ok(output) => {
                        results.push(serde_json::json!({
                            "source": src_str,
                            "success": false,
                            "error": format!("Failed to backup dest: {}", output.to_result_string())
                        }));
                        continue;
                    }
                    Err(e) => {
                        results.push(serde_json::json!({
                            "source": src_str,
                            "success": false,
                            "error": format!("Failed to backup dest: {}", e)
                        }));
                        continue;
                    }
                }
            }

            let metadata = match fs::metadata(source).await {
                Ok(m) => m,
                Err(e) => {
                    results.push(serde_json::json!({
                        "source": src_str,
                        "success": false,
                        "error": format!("Source not found: {}", e)
                    }));
                    continue;
                }
            };

            if metadata.is_dir() {
                if !req.recursive.unwrap_or(false) {
                    results.push(serde_json::json!({
                        "source": src_str,
                        "success": false,
                        "error": "Source is a directory. Use recursive=true."
                    }));
                    continue;
                }
                match copy_dir_recursive(source, &actual_dest).await {
                    Ok(count) => {
                        results.push(serde_json::json!({
                            "source": src_str,
                            "dest": actual_dest.to_string_lossy(),
                            "success": true,
                            "type": "directory",
                            "files_copied": count,
                            "graveyarded_dest": graveyarded
                        }));
                    }
                    Err(e) => {
                        results.push(serde_json::json!({
                            "source": src_str,
                            "success": false,
                            "error": format!("Failed to copy directory: {}", e)
                        }));
                    }
                }
            } else {
                match fs::copy(source, &actual_dest).await {
                    Ok(bytes) => {
                        results.push(serde_json::json!({
                            "source": src_str,
                            "dest": actual_dest.to_string_lossy(),
                            "success": true,
                            "bytes_copied": bytes,
                            "graveyarded_dest": graveyarded
                        }));
                    }
                    Err(e) => {
                        results.push(serde_json::json!({
                            "source": src_str,
                            "success": false,
                            "error": format!("Failed to copy: {}", e)
                        }));
                    }
                }
            }
        }

        let success_count = results.iter().filter(|r| r["success"] == true).count();
        let result = serde_json::json!({
            "total": sources.len(),
            "success_count": success_count,
            "results": results
        });
        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    #[tool(
        name = "Filesystem - Move",
        description = "Move or rename file(s) or directory(s). Supports multiple space-separated sources. Use safe_overwrite=true to backup dest to graveyard before overwriting."
    )]
    async fn fs_move(
        &self,
        Parameters(req): Parameters<FsMoveRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use tokio::fs;

        // Parse multiple sources (space-separated)
        let sources: Vec<&str> = req.source.split_whitespace().collect();
        let dest = std::path::Path::new(&req.dest);

        // Validate dest
        if let Err(msg) = self.ignore.validate_path(dest) {
            return Ok(CallToolResult::error(vec![Content::text(msg)]));
        }

        // For multiple sources, dest must be an existing directory
        if sources.len() > 1 {
            if !dest.is_dir() {
                return Ok(CallToolResult::error(vec![Content::text(
                    "Multiple sources specified but destination is not a directory",
                )]));
            }
        }

        let mut results: Vec<serde_json::Value> = vec![];
        let mut graveyarded = false;

        for src_str in &sources {
            let source = std::path::Path::new(src_str);

            // Check .agentignore
            if let Err(msg) = self.ignore.validate_path(source) {
                results.push(serde_json::json!({
                    "source": src_str,
                    "success": false,
                    "error": msg
                }));
                continue;
            }

            // Determine actual destination
            let actual_dest = if dest.is_dir() {
                dest.join(source.file_name().unwrap_or_default())
            } else {
                dest.to_path_buf()
            };

            // Safe overwrite: backup existing dest to graveyard
            if req.safe_overwrite.unwrap_or(false) && actual_dest.exists() {
                let mut rip_args: Vec<String> = vec![];
                if let Some(graveyard) = &req.graveyard {
                    rip_args.push(format!("--graveyard={}", graveyard));
                }
                rip_args.push(actual_dest.to_string_lossy().to_string());

                let args_ref: Vec<&str> = rip_args.iter().map(|s| s.as_str()).collect();
                match self.executor.run("rip", &args_ref).await {
                    Ok(output) if output.success => {
                        graveyarded = true;
                    }
                    Ok(output) => {
                        results.push(serde_json::json!({
                            "source": src_str,
                            "success": false,
                            "error": format!("Failed to backup dest: {}", output.to_result_string())
                        }));
                        continue;
                    }
                    Err(e) => {
                        results.push(serde_json::json!({
                            "source": src_str,
                            "success": false,
                            "error": format!("Failed to backup dest: {}", e)
                        }));
                        continue;
                    }
                }
            }

            match fs::rename(source, &actual_dest).await {
                Ok(()) => {
                    results.push(serde_json::json!({
                        "source": src_str,
                        "dest": actual_dest.to_string_lossy(),
                        "success": true,
                        "graveyarded_dest": graveyarded
                    }));
                }
                Err(e) => {
                    results.push(serde_json::json!({
                        "source": src_str,
                        "success": false,
                        "error": format!("Failed to move: {}", e)
                    }));
                }
            }
        }

        let success_count = results.iter().filter(|r| r["success"] == true).count();
        let result = serde_json::json!({
            "total": sources.len(),
            "success_count": success_count,
            "results": results
        });
        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    #[tool(
        name = "Filesystem - Stat",
        description = "Get file or directory metadata (size, permissions, timestamps)."
    )]
    async fn fs_stat(
        &self,
        Parameters(req): Parameters<FsStatRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use tokio::fs;

        let paths: Vec<&str> = req.path.split_whitespace().collect();
        let mut results = Vec::new();

        for path_str in &paths {
            match fs::metadata(path_str).await {
                Ok(meta) => {
                    let file_type = if meta.is_dir() {
                        "directory"
                    } else if meta.is_file() {
                        "file"
                    } else if meta.is_symlink() {
                        "symlink"
                    } else {
                        "other"
                    };

                    #[cfg(unix)]
                    let permissions = {
                        use std::os::unix::fs::PermissionsExt;
                        format!("{:o}", meta.permissions().mode() & 0o777)
                    };
                    #[cfg(not(unix))]
                    let permissions = if meta.permissions().readonly() {
                        "readonly"
                    } else {
                        "writable"
                    };

                    results.push(serde_json::json!({
                        "path": path_str,
                        "exists": true,
                        "type": file_type,
                        "size": meta.len(),
                        "permissions": permissions,
                        "readonly": meta.permissions().readonly()
                    }));
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        results.push(serde_json::json!({
                            "path": path_str,
                            "exists": false
                        }));
                    } else {
                        results.push(serde_json::json!({
                            "path": path_str,
                            "error": e.to_string()
                        }));
                    }
                }
            }
        }

        let response = if paths.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            serde_json::json!({ "results": results })
        };

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    #[tool(
        name = "Filesystem - Exists",
        description = "Check if path(s) exist. Returns boolean for single path, array for multiple."
    )]
    async fn fs_exists(
        &self,
        Parameters(req): Parameters<FsExistsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let paths: Vec<&str> = req.path.split_whitespace().collect();
        let mut results = Vec::new();

        for path_str in &paths {
            let path = std::path::Path::new(path_str);
            results.push(serde_json::json!({
                "path": path_str,
                "exists": path.exists()
            }));
        }

        let response = if paths.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            serde_json::json!({ "results": results })
        };

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    #[tool(
        name = "Filesystem - Symlink",
        description = "Create a symbolic link. Use safe_overwrite=true to backup existing link to graveyard."
    )]
    async fn fs_symlink(
        &self,
        Parameters(req): Parameters<FsSymlinkRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use tokio::fs;

        let link_path = std::path::Path::new(&req.link);
        let mut graveyarded = false;

        // Safe overwrite: if link exists and safe_overwrite is true, rip it first
        if req.safe_overwrite.unwrap_or(false) && (link_path.exists() || link_path.is_symlink()) {
            let mut rip_args: Vec<String> = vec![];
            if let Some(graveyard) = &req.graveyard {
                rip_args.push(format!("--graveyard={}", graveyard));
            }
            rip_args.push(req.link.clone());

            let args_ref: Vec<&str> = rip_args.iter().map(|s| s.as_str()).collect();
            match self.executor.run("rip", &args_ref).await {
                Ok(output) if output.success => {
                    graveyarded = true;
                }
                Ok(output) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to backup existing link to graveyard: {}",
                        output.to_result_string()
                    ))]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to backup existing link to graveyard: {}",
                        e
                    ))]));
                }
            }
        }

        match fs::symlink(&req.target, &req.link).await {
            Ok(()) => {
                let result = serde_json::json!({
                    "success": true,
                    "target": req.target,
                    "link": req.link,
                    "graveyarded_existing": graveyarded
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to create symlink: {}",
                e
            ))])),
        }
    }

    #[tool(
        name = "Filesystem - Hardlink",
        description = "Create a hard link. Use safe_overwrite=true to backup existing link to graveyard."
    )]
    async fn fs_hardlink(
        &self,
        Parameters(req): Parameters<FsHardlinkRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use tokio::fs;

        let link_path = std::path::Path::new(&req.link);
        let mut graveyarded = false;

        // Safe overwrite: if link exists and safe_overwrite is true, rip it first
        if req.safe_overwrite.unwrap_or(false) && link_path.exists() {
            let mut rip_args: Vec<String> = vec![];
            if let Some(graveyard) = &req.graveyard {
                rip_args.push(format!("--graveyard={}", graveyard));
            }
            rip_args.push(req.link.clone());

            let args_ref: Vec<&str> = rip_args.iter().map(|s| s.as_str()).collect();
            match self.executor.run("rip", &args_ref).await {
                Ok(output) if output.success => {
                    graveyarded = true;
                }
                Ok(output) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to backup existing file to graveyard: {}",
                        output.to_result_string()
                    ))]));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to backup existing file to graveyard: {}",
                        e
                    ))]));
                }
            }
        }

        match fs::hard_link(&req.source, &req.link).await {
            Ok(()) => {
                let result = serde_json::json!({
                    "success": true,
                    "source": req.source,
                    "link": req.link,
                    "graveyarded_existing": graveyarded
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to create hard link: {}",
                e
            ))])),
        }
    }

    // ========================================================================
    // MCP STATE TOOLS
    // ========================================================================

    #[tool(
        name = "MCP - Cache Get",
        description = "Get a cached value by key. Returns null if not found or expired."
    )]
    async fn mcp_cache_get(
        &self,
        Parameters(req): Parameters<McpCacheGetRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.state.cache_get(&req.key) {
            Ok(value) => {
                let result = serde_json::json!({
                    "key": req.key,
                    "value": value,
                    "found": value.is_some()
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "MCP - Cache Set",
        description = "Set a cached value with optional TTL (time-to-live in seconds)."
    )]
    async fn mcp_cache_set(
        &self,
        Parameters(req): Parameters<McpCacheSetRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.state.cache_set(&req.key, &req.value, req.ttl_secs) {
            Ok(()) => {
                let result = serde_json::json!({
                    "success": true,
                    "key": req.key,
                    "ttl_secs": req.ttl_secs
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "MCP - Task Create",
        description = "Create a new task in the MCP task list."
    )]
    async fn mcp_task_create(
        &self,
        Parameters(req): Parameters<McpTaskCreateRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.state.task_create(&req.content) {
            Ok(task) => {
                let result = serde_json::json!({
                    "success": true,
                    "task": {
                        "id": task.id,
                        "content": task.content,
                        "status": task.status.to_string()
                    }
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(name = "MCP - Task Update", description = "Update a task's status.")]
    async fn mcp_task_update(
        &self,
        Parameters(req): Parameters<McpTaskUpdateRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let status: TaskStatus = match req.status.parse() {
            Ok(s) => s,
            Err(e) => return Ok(CallToolResult::error(vec![Content::text(e)])),
        };

        match self.state.task_update_status(req.id, status) {
            Ok(()) => {
                let result = serde_json::json!({
                    "success": true,
                    "id": req.id,
                    "status": req.status
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "MCP - Task List",
        description = "List tasks, optionally filtered by status."
    )]
    async fn mcp_task_list(
        &self,
        Parameters(req): Parameters<McpTaskListRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let status_filter: Option<TaskStatus> = req.status.and_then(|s| s.parse().ok());

        match self.state.task_list(status_filter) {
            Ok(tasks) => {
                let task_json: Vec<serde_json::Value> = tasks
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "id": t.id,
                            "content": t.content,
                            "status": t.status.to_string()
                        })
                    })
                    .collect();

                let result = serde_json::json!({
                    "tasks": task_json,
                    "count": tasks.len()
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(name = "MCP - Task Delete", description = "Delete a task by ID.")]
    async fn mcp_task_delete(
        &self,
        Parameters(req): Parameters<McpTaskDeleteRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.state.task_delete(req.id) {
            Ok(()) => {
                let result = serde_json::json!({
                    "success": true,
                    "id": req.id
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "MCP - Context Get",
        description = "Get a context value by key and scope."
    )]
    async fn mcp_context_get(
        &self,
        Parameters(req): Parameters<McpContextGetRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let scope: ContextScope = req
            .scope
            .and_then(|s| s.parse().ok())
            .unwrap_or(ContextScope::Session);

        match self.state.context_get(&req.key, scope.clone()) {
            Ok(value) => {
                let result = serde_json::json!({
                    "key": req.key,
                    "scope": scope.to_string(),
                    "value": value,
                    "found": value.is_some()
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "MCP - Context Set",
        description = "Set a context value with specified scope."
    )]
    async fn mcp_context_set(
        &self,
        Parameters(req): Parameters<McpContextSetRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let scope: ContextScope = req
            .scope
            .and_then(|s| s.parse().ok())
            .unwrap_or(ContextScope::Session);

        match self.state.context_set(&req.key, &req.value, scope.clone()) {
            Ok(()) => {
                let result = serde_json::json!({
                    "success": true,
                    "key": req.key,
                    "scope": scope.to_string()
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "MCP - Context List",
        description = "List all context entries, optionally filtered by scope."
    )]
    async fn mcp_context_list(
        &self,
        Parameters(req): Parameters<McpContextListRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let scope_filter: Option<ContextScope> = req.scope.and_then(|s| s.parse().ok());

        match self.state.context_list(scope_filter) {
            Ok(entries) => {
                let entry_json: Vec<serde_json::Value> = entries
                    .iter()
                    .map(|e| {
                        serde_json::json!({
                            "key": e.key,
                            "scope": e.scope.to_string(),
                            "value": e.value
                        })
                    })
                    .collect();

                let result = serde_json::json!({
                    "entries": entry_json,
                    "count": entries.len()
                });
                Ok(CallToolResult::success(vec![Content::text(
                    result.to_string(),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        name = "MCP - Auth Check",
        description = "Check and refresh all auth states. Returns status for gh and glab."
    )]
    async fn mcp_auth_check(&self) -> Result<CallToolResult, ErrorData> {
        let mut auth_states = Vec::new();

        // Check GitHub auth
        let gh_args = vec!["auth", "status"];
        let gh_result = self.executor.run("gh", &gh_args).await;
        let gh_authenticated = gh_result.as_ref().map(|o| o.success).unwrap_or(false);

        let gh_state = crate::state::AuthState {
            provider: "gh:github.com".to_string(),
            authenticated: gh_authenticated,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            metadata: None,
        };
        let _ = self.state.set_auth_state(&gh_state);
        auth_states.push(serde_json::json!({
            "provider": "github",
            "hostname": "github.com",
            "authenticated": gh_authenticated
        }));

        // Check GitLab auth
        let glab_args = vec!["auth", "status"];
        let glab_result = self.executor.run("glab", &glab_args).await;
        let glab_authenticated = glab_result.as_ref().map(|o| o.success).unwrap_or(false);

        let glab_state = crate::state::AuthState {
            provider: "glab:gitlab.com".to_string(),
            authenticated: glab_authenticated,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            metadata: None,
        };
        let _ = self.state.set_auth_state(&glab_state);
        auth_states.push(serde_json::json!({
            "provider": "gitlab",
            "hostname": "gitlab.com",
            "authenticated": glab_authenticated
        }));

        let result = serde_json::json!({
            "auth_states": auth_states
        });
        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    // ========================================================================
    // VIRTUAL TOOL GROUP TOOLS
    // ========================================================================

    #[tool(
        name = "expand_tools",
        description = "List tools in a group. Groups: filesystem (eza/bat/fd/disk utils), \
        file_ops (read/write/edit), search (ripgrep/ast-grep/symbols), text (jq/yq/csv), \
        git (status/diff/log/branch), github (issues/PRs/releases), gitlab (MRs/pipelines), \
        kubernetes (kubectl/helm), container (podman/registry/scan), network (HTTP/SQL), \
        system (shell/benchmarks), archive (compress/decompress), reference (tldr/cheatsheets), \
        diff (delta/difftastic), mcp (task/context/cache)"
    )]
    async fn expand_tools(
        &self,
        Parameters(req): Parameters<ExpandToolsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let group = req.group.parse::<ToolGroup>().map_err(|e| {
            ErrorData::new(
                rmcp::model::ErrorCode::INVALID_REQUEST,
                e,
                None::<serde_json::Value>,
            )
        })?;

        let tools = group.tools();
        let tool_list = tools
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{}. {}", i + 1, t))
            .collect::<Vec<_>>()
            .join("\n");

        let output = format!(
            "## {} ({} tools)\n\n{}\n\n{}\n\nCall any tool directly by name.",
            group.name(),
            tools.len(),
            group.description(),
            tool_list
        );

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(
        name = "list_tool_groups",
        description = "List all available tool groups with descriptions and tool counts."
    )]
    async fn list_tool_groups(&self) -> Result<CallToolResult, ErrorData> {
        let mut output = String::from("## Available Tool Groups\n\n");

        // Show profile info if set
        if let Some(profile) = &self.profile {
            output.push_str(&format!(
                "**Active Profile:** {} - {}\n\n",
                profile.id(),
                profile.description()
            ));
            output.push_str("**Pre-expanded groups:** ");
            let pre_expanded: Vec<_> = profile
                .pre_expanded_groups()
                .iter()
                .map(|g| g.id())
                .collect();
            output.push_str(&pre_expanded.join(", "));
            output.push_str("\n\n---\n\n");
        }

        for group in ToolGroup::ALL {
            output.push_str(&format!(
                "### {} ({} tools)\n{}\n\n",
                group.name(),
                group.tool_count(),
                group.description()
            ));
        }

        output.push_str(&format!(
            "---\n**Total:** {} tools across {} groups\n\n\
            Use `expand_tools` with a group name to see individual tools.",
            ToolGroup::ALL.iter().map(|g| g.tool_count()).sum::<usize>(),
            ToolGroup::ALL.len()
        ));

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    // ========================================================================
    // DYNAMIC TOOLSETS (BETA)
    // ========================================================================

    #[tool(
        name = "list_available_toolsets",
        description = "List all available toolsets with their enabled status. \
        Use this to discover which toolsets can be enabled. \
        Only available when dynamic toolsets mode is active."
    )]
    async fn list_available_toolsets(&self) -> Result<CallToolResult, ErrorData> {
        if !self.dynamic_config.enabled {
            return Ok(CallToolResult::success(vec![Content::text(
                "Dynamic toolsets mode is not enabled. \
                Start the server with --dynamic-toolsets flag to use this feature.\n\n\
                Current mode: All tools are available. Use `list_tool_groups` or `expand_tools` instead.",
            )]));
        }

        let enabled_groups = self.dynamic_config.enabled_groups.read();
        let mut output = String::from("## Available Toolsets\n\n");
        output.push_str("| Toolset | Tools | Status | Description |\n");
        output.push_str("|---------|-------|--------|-------------|\n");

        for group in ToolGroup::ALL {
            let status = if enabled_groups.contains(group) {
                "✓ Enabled"
            } else {
                "○ Disabled"
            };
            output.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                group.id(),
                group.tool_count(),
                status,
                group.description()
            ));
        }

        let enabled_count = enabled_groups.len();
        let total_tools: usize = enabled_groups.iter().map(|g| g.tool_count()).sum();
        output.push_str(&format!(
            "\n**Enabled:** {}/{} toolsets ({} tools)\n\n\
            Use `enable_toolset` to activate a toolset.\n\
            Use `get_toolset_tools` to preview tools before enabling.",
            enabled_count,
            ToolGroup::ALL.len(),
            total_tools
        ));

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(
        name = "get_toolset_tools",
        description = "Get the list of tools in a specific toolset without enabling it. \
        Use this to preview what tools will become available when you enable a toolset."
    )]
    async fn get_toolset_tools(
        &self,
        Parameters(req): Parameters<GetToolsetToolsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let group = req.toolset.parse::<ToolGroup>().map_err(|e| {
            ErrorData::new(
                rmcp::model::ErrorCode::INVALID_REQUEST,
                e,
                None::<serde_json::Value>,
            )
        })?;

        let tools = group.tools();
        let enabled = self.is_group_enabled(group);

        let mut output = format!("## {} Toolset ({} tools)\n\n", group.name(), tools.len());

        if self.dynamic_config.enabled {
            output.push_str(&format!(
                "**Status:** {}\n\n",
                if enabled {
                    "✓ Enabled"
                } else {
                    "○ Disabled"
                }
            ));
        }

        output.push_str(&format!("**Description:** {}\n\n", group.description()));
        output.push_str("**Tools:**\n");

        for (i, tool) in tools.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, tool));
        }

        if self.dynamic_config.enabled && !enabled {
            output.push_str(&format!(
                "\n---\nUse `enable_toolset(\"{}\")` to activate these tools.",
                group.id()
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(
        name = "enable_toolset",
        description = "Enable a toolset to make its tools available. \
        Use 'all' to enable all toolsets at once. \
        Only available when dynamic toolsets mode is active. \
        After enabling, the tool list will be updated."
    )]
    async fn enable_toolset(
        &self,
        Parameters(req): Parameters<EnableToolsetRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        if !self.dynamic_config.enabled {
            return Ok(CallToolResult::success(vec![Content::text(
                "Dynamic toolsets mode is not enabled. \
                Start the server with --dynamic-toolsets flag to use this feature.\n\n\
                Current mode: All tools are already available.",
            )]));
        }

        // Handle 'all' special case
        if req.toolset.to_lowercase() == "all" {
            let mut enabled_groups = self.dynamic_config.enabled_groups.write();
            let already_enabled = enabled_groups.len();
            for group in ToolGroup::ALL {
                enabled_groups.insert(*group);
            }
            let newly_enabled = ToolGroup::ALL.len() - already_enabled;
            let total_tools: usize = ToolGroup::ALL.iter().map(|g| g.tool_count()).sum();

            // Note: Notification would be sent here if we had access to the peer
            // For now, we indicate that the client should refresh the tool list
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "## All Toolsets Enabled\n\n\
                Enabled {} new toolsets ({} were already enabled).\n\
                **Total tools now available:** {}\n\n\
                ⚠️ **Note:** The tool list has changed. \
                The client should receive a `tools/list_changed` notification.",
                newly_enabled, already_enabled, total_tools
            ))]));
        }

        // Parse specific toolset
        let group = req.toolset.parse::<ToolGroup>().map_err(|e| {
            ErrorData::new(
                rmcp::model::ErrorCode::INVALID_REQUEST,
                e,
                None::<serde_json::Value>,
            )
        })?;

        // Check if already enabled
        if self.is_group_enabled(group) {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Toolset '{}' is already enabled.\n\n\
                **Available tools:** {}",
                group.id(),
                group.tool_count()
            ))]));
        }

        // Enable the group
        self.enable_group(group);

        let tools = group.tools();
        let tool_list = tools
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{}. {}", i + 1, t))
            .collect::<Vec<_>>()
            .join("\n");

        // Note: Notification would be sent here if we had access to the peer
        Ok(CallToolResult::success(vec![Content::text(format!(
            "## Toolset '{}' Enabled\n\n\
            **Tools now available ({}):**\n{}\n\n\
            ⚠️ **Note:** The tool list has changed. \
            The client should receive a `tools/list_changed` notification.",
            group.id(),
            tools.len(),
            tool_list
        ))]))
    }
}

// Helper functions
fn octal_to_rwx(bits: u32) -> String {
    let r = if bits & 4 != 0 { 'r' } else { '-' };
    let w = if bits & 2 != 0 { 'w' } else { '-' };
    let x = if bits & 1 != 0 { 'x' } else { '-' };
    format!("{}{}{}", r, w, x)
}

fn symbolic_to_octal(symbolic: &str) -> u32 {
    let chars: Vec<char> = symbolic.chars().collect();
    let mut result = 0u32;
    for (i, chunk) in chars.chunks(3).enumerate() {
        let mut val = 0u32;
        for c in chunk {
            match c {
                'r' => val |= 4,
                'w' => val |= 2,
                'x' | 's' | 't' => val |= 1,
                _ => {}
            }
        }
        result |= val << (6 - i * 3);
    }
    result
}

fn describe_perms(rwx: &str) -> String {
    let mut perms = Vec::new();
    for c in rwx.chars() {
        match c {
            'r' => perms.push("read"),
            'w' => perms.push("write"),
            'x' => perms.push("execute"),
            _ => {}
        }
    }
    if perms.is_empty() {
        "no permissions".into()
    } else {
        perms.join(", ")
    }
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

/// Wrap text output in JSON envelope for consistent AI consumption
fn text_to_json_envelope(tool: &str, output: &str, success: bool) -> String {
    serde_json::json!({
        "success": success,
        "tool": tool,
        "output": output
    })
    .to_string()
}

/// Parse git status --porcelain=v2 output to JSON
fn parse_git_status_porcelain_v2(output: &str) -> serde_json::Value {
    let mut branch = serde_json::json!({});
    let mut files: Vec<serde_json::Value> = vec![];

    for line in output.lines() {
        if line.starts_with("# branch.oid ") {
            branch["oid"] = serde_json::json!(line.strip_prefix("# branch.oid ").unwrap_or(""));
        } else if line.starts_with("# branch.head ") {
            branch["head"] = serde_json::json!(line.strip_prefix("# branch.head ").unwrap_or(""));
        } else if line.starts_with("# branch.upstream ") {
            branch["upstream"] =
                serde_json::json!(line.strip_prefix("# branch.upstream ").unwrap_or(""));
        } else if line.starts_with("# branch.ab ") {
            let ab = line.strip_prefix("# branch.ab ").unwrap_or("");
            let parts: Vec<&str> = ab.split_whitespace().collect();
            if parts.len() >= 2 {
                branch["ahead"] =
                    serde_json::json!(parts[0].trim_start_matches('+').parse::<i32>().unwrap_or(0));
                branch["behind"] =
                    serde_json::json!(parts[1].trim_start_matches('-').parse::<i32>().unwrap_or(0));
            }
        } else if line.starts_with("1 ") || line.starts_with("2 ") {
            // Changed entries
            let parts: Vec<&str> = line.splitn(9, ' ').collect();
            if parts.len() >= 9 {
                let xy = parts[1];
                files.push(serde_json::json!({
                    "status": xy,
                    "staged": xy.chars().next().unwrap_or(' ') != '.',
                    "unstaged": xy.chars().nth(1).unwrap_or(' ') != '.',
                    "path": parts[8],
                }));
            }
        } else if line.starts_with("? ") {
            // Untracked
            files.push(serde_json::json!({
                "status": "?",
                "staged": false,
                "unstaged": false,
                "untracked": true,
                "path": line.strip_prefix("? ").unwrap_or(""),
            }));
        } else if line.starts_with("! ") {
            // Ignored
            files.push(serde_json::json!({
                "status": "!",
                "ignored": true,
                "path": line.strip_prefix("! ").unwrap_or(""),
            }));
        }
    }

    serde_json::json!({
        "branch": branch,
        "files": files,
    })
}

async fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<u64> {
    use tokio::fs;

    fs::create_dir_all(dst).await?;
    let mut count = 0u64;
    let mut entries = fs::read_dir(src).await?;

    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let file_type = entry.file_type().await?;

        if file_type.is_dir() {
            count += Box::pin(copy_dir_recursive(&src_path, &dst_path)).await?;
        } else {
            fs::copy(&src_path, &dst_path).await?;
            count += 1;
        }
    }

    Ok(count)
}

// Manual ServerHandler implementation for dynamic tool filtering
impl ServerHandler for ModernCliTools {
    fn get_info(&self) -> ServerInfo {
        let instructions = self.build_instructions();

        // Enable listChanged capability when dynamic toolsets are active
        let capabilities = if self.dynamic_config.enabled {
            ServerCapabilities::builder()
                .enable_tools()
                .enable_tool_list_changed()
                .build()
        } else {
            ServerCapabilities::builder().enable_tools().build()
        };

        ServerInfo {
            instructions: Some(instructions),
            capabilities,
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        // In non-dynamic mode, return all tools
        if !self.dynamic_config.enabled {
            return Ok(ListToolsResult::with_all_items(self.tool_router.list_all()));
        }

        // Dynamic mode: filter by enabled groups
        let enabled_groups = self.dynamic_config.enabled_groups.read();

        let filtered_tools: Vec<Tool> = self
            .tool_router
            .map
            .values()
            .filter(|route| {
                let tool_name = route.attr.name.as_ref();
                // Check if this tool belongs to an enabled group
                // Meta-tools (not in any group) are always visible
                self.tool_to_group
                    .get(tool_name)
                    .map(|group| enabled_groups.contains(group))
                    .unwrap_or(true) // Meta-tools always visible
            })
            .map(|route| route.attr.clone())
            .collect();

        Ok(ListToolsResult::with_all_items(filtered_tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let tcc = ToolCallContext::new(self, request, context);
        self.tool_router.call(tcc).await
    }
}

impl ModernCliTools {
    fn build_instructions(&self) -> String {
        let base = "Modern CLI Tools MCP Server - Exposes modern command-line utilities \
            like eza, bat, fd, rg, delta, jq, and many more for AI-assisted \
            file operations, text processing, system monitoring, and development tasks.";

        let mut instructions = String::from(base);

        // Dynamic toolsets mode (beta)
        if self.dynamic_config.enabled {
            instructions.push_str(
                "\n\n## Dynamic Toolsets Mode (Beta)\n\
                This server is running in dynamic toolsets mode. Tools are organized into \
                toolsets that can be enabled on demand.\n\n\
                **Available commands:**\n\
                - `list_available_toolsets` - Show all toolsets and their status\n\
                - `get_toolset_tools` - Preview tools in a toolset\n\
                - `enable_toolset` - Enable a toolset to activate its tools\n\n",
            );

            let enabled_groups = self.dynamic_config.enabled_groups.read();
            if enabled_groups.is_empty() {
                instructions.push_str(
                    "**No toolsets enabled yet.** Use `list_available_toolsets` to see \
                    available toolsets and `enable_toolset` to activate them.",
                );
            } else {
                instructions.push_str("**Currently enabled toolsets:**\n");
                for group in ToolGroup::ALL {
                    if enabled_groups.contains(group) {
                        instructions.push_str(&format!(
                            "- {} ({} tools)\n",
                            group.id(),
                            group.tool_count()
                        ));
                    }
                }
            }
            return instructions;
        }

        // Profile mode
        if let Some(profile) = &self.profile {
            instructions.push_str(&format!(
                "\n\n## Active Profile: {}\n{}",
                profile.id(),
                profile.description()
            ));

            instructions.push_str("\n\n### Pre-expanded Tool Groups:\n");
            for group in profile.pre_expanded_groups() {
                instructions.push_str(&format!(
                    "- **{}**: {}\n",
                    group.name(),
                    group.description()
                ));
            }

            instructions.push_str("\n### Other Groups (use `expand_tools` to explore):\n");
            let pre_expanded = profile.pre_expanded_groups();
            for group in ToolGroup::ALL {
                if !pre_expanded.contains(group) {
                    instructions.push_str(&format!("- {}\n", group.id()));
                }
            }
        } else {
            // Default mode - all tools available
            instructions.push_str(
                "\n\n## Tool Organization\n\
                Tools are organized into 15 groups. Use `list_tool_groups` to see all groups \
                or `expand_tools` with a group name to explore tools in that group.\n\n\
                **Quick Reference:**\n\
                - filesystem: eza, bat, fd, disk utilities\n\
                - file_ops: read, write, edit files\n\
                - search: ripgrep, ast-grep, code symbols\n\
                - git: status, diff, log, branches\n\
                - github/gitlab: issues, PRs, releases\n\
                - kubernetes: kubectl, helm\n\
                - container: podman, registries, security\n\
                - text: jq, yq, CSV processing\n\
                - network: HTTP, SQL\n\
                - system: shell, benchmarks",
            );
        }

        instructions
    }
}
