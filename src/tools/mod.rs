// modern-cli-mcp/src/tools/mod.rs
mod executor;

pub use executor::CommandExecutor;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router, ErrorData, ServerHandler,
};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ModernCliTools {
    tool_router: ToolRouter<Self>,
    executor: CommandExecutor,
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
    #[schemars(description = "File or directory path to trash")]
    pub path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TrashEmptyRequest {
    #[schemars(description = "Only empty items older than days")]
    pub days: Option<u32>,
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
    #[schemars(description = "Maximum number of results")]
    pub max_count: Option<u32>,
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
    #[schemars(description = "JSON output")]
    pub json: Option<bool>,
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

// --- Utility ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeltaRequest {
    #[schemars(description = "First file path")]
    pub file_a: String,
    #[schemars(description = "Second file path")]
    pub file_b: String,
    #[schemars(description = "Side by side view")]
    pub side_by_side: Option<bool>,
    #[schemars(description = "Show line numbers")]
    pub line_numbers: Option<bool>,
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

// ============================================================================
// TOOL IMPLEMENTATIONS
// ============================================================================

#[tool_router]
impl ModernCliTools {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            executor: CommandExecutor::new(),
        }
    }

    // ========================================================================
    // FILESYSTEM TOOLS
    // ========================================================================

    #[tool(
        description = "List directory contents with eza (modern ls replacement). \
        Features: icons, git integration, tree view, extended attributes."
    )]
    async fn eza(
        &self,
        Parameters(req): Parameters<EzaRequest>,
    ) -> Result<CallToolResult, ErrorData> {
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
        if let Some(ref path) = req.path {
            args.push(path.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("eza", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        description = "Display file contents with syntax highlighting using bat (modern cat replacement). \
        Features: syntax highlighting, line numbers, git integration, line ranges."
    )]
    async fn bat(
        &self,
        Parameters(req): Parameters<BatRequest>,
    ) -> Result<CallToolResult, ErrorData> {
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
        description = "Find files and directories with fd (modern find replacement). \
        Features: regex patterns, respects gitignore, type filtering, parallel execution."
    )]
    async fn fd(
        &self,
        Parameters(req): Parameters<FdRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["--color=never".into()];

        if req.hidden.unwrap_or(false) {
            args.push("-H".into());
        }
        if req.no_ignore.unwrap_or(false) {
            args.push("-I".into());
        }
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
        if let Some(ref pattern) = req.pattern {
            args.push(pattern.clone());
        }
        if let Some(ref path) = req.path {
            args.push(path.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("fd", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Show disk usage with duf (modern df replacement). \
        Features: colorful output, all mount points, JSON output.")]
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
        description = "Analyze directory sizes with dust (modern du replacement). \
        Features: visual bars, tree view, customizable depth."
    )]
    async fn dust(
        &self,
        Parameters(req): Parameters<DustRequest>,
    ) -> Result<CallToolResult, ErrorData> {
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
        if let Some(ref path) = req.path {
            args.push(path.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("dust", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Move file or directory to trash (safe delete).")]
    async fn trash_put(
        &self,
        Parameters(req): Parameters<TrashRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.executor.run("trash-put", &[&req.path]).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                if output.success {
                    format!("Moved '{}' to trash", req.path)
                } else {
                    output.to_result_string()
                },
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "List files in trash.")]
    async fn trash_list(&self) -> Result<CallToolResult, ErrorData> {
        match self.executor.run("trash-list", &[]).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Empty the trash.")]
    async fn trash_empty(
        &self,
        Parameters(req): Parameters<TrashEmptyRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec![];
        if let Some(days) = req.days {
            args.push(format!("--days={}", days));
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("trash-empty", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                if output.success {
                    "Trash emptied".into()
                } else {
                    output.to_result_string()
                },
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // SEARCH TOOLS
    // ========================================================================

    #[tool(
        description = "Search file contents with ripgrep (rg) - extremely fast grep replacement. \
        Features: regex, respects gitignore, parallel search, many output formats."
    )]
    async fn rg(
        &self,
        Parameters(req): Parameters<RipgrepRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["--color=never".into()];

        if req.ignore_case.unwrap_or(false) {
            args.push("-i".into());
        }
        if req.smart_case.unwrap_or(false) {
            args.push("-S".into());
        }
        if req.hidden.unwrap_or(false) {
            args.push("--hidden".into());
        }
        if req.no_ignore.unwrap_or(false) {
            args.push("--no-ignore".into());
        }
        if req.files_with_matches.unwrap_or(false) {
            args.push("-l".into());
        }
        if req.count.unwrap_or(false) {
            args.push("-c".into());
        }
        if req.line_number.unwrap_or(true) {
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
        if req.json.unwrap_or(false) {
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
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Filter and fuzzy-find items with fzf. \
        Pass a list of items and a query to get fuzzy-matched results.")]
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
                Ok(CallToolResult::success(vec![Content::text(
                    if result.is_empty() {
                        "(no matches)".into()
                    } else {
                        result
                    },
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
        description = "Search code with ast-grep (sg) - AST-based structural search. \
        Find code patterns semantically, not just textually."
    )]
    async fn ast_grep(
        &self,
        Parameters(req): Parameters<AstGrepRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["run".into(), "--pattern".into(), req.pattern.clone()];

        if let Some(ref lang) = req.lang {
            args.push(format!("--lang={}", lang));
        }
        if let Some(ref rewrite) = req.rewrite {
            args.push(format!("--rewrite={}", rewrite));
        }
        if req.json.unwrap_or(false) {
            args.push("--json".into());
        }
        if let Some(ref path) = req.path {
            args.push(path.clone());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.executor.run("sg", &args_ref).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // TEXT PROCESSING TOOLS
    // ========================================================================

    #[tool(
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
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
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

    #[tool(description = "Process CSV with xsv - fast CSV toolkit. \
        Commands: stats, select, search, sort, slice, frequency, count, headers.")]
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
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ========================================================================
    // SYSTEM TOOLS
    // ========================================================================

    #[tool(
        description = "List and filter processes with procs (modern ps replacement). \
        Features: tree view, sorting, filtering, colorful output."
    )]
    async fn procs(
        &self,
        Parameters(req): Parameters<ProcsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut args: Vec<String> = vec!["--color=never".into()];

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
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(
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
        if let Some(ref output) = req.output {
            args.push(format!("--output={}", output));
        }
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
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Benchmark commands with hyperfine. \
        Precise timing with warmup, statistical analysis, comparison.")]
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

    #[tool(description = "Get system resource usage snapshot (memory, CPU, uptime).")]
    async fn system_info(&self) -> Result<CallToolResult, ErrorData> {
        let mut result = String::new();

        if let Ok(output) = self.executor.run("free", &["-h"]).await {
            result.push_str("=== Memory ===\n");
            result.push_str(&output.stdout);
            result.push('\n');
        }

        if let Ok(output) = self.executor.run("uptime", &[]).await {
            result.push_str("=== Uptime ===\n");
            result.push_str(&output.stdout);
        }

        Ok(CallToolResult::success(vec![Content::text(
            if result.is_empty() {
                "Could not retrieve system info".into()
            } else {
                result
            },
        )]))
    }

    // ========================================================================
    // NETWORK TOOLS
    // ========================================================================

    #[tool(description = "Make HTTP requests with xh (HTTPie-compatible). \
        Features: JSON by default, syntax highlighting, intuitive syntax.")]
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
            Err(e) => {
                // Fallback to curl
                if e.contains("not found") {
                    match self.executor.run("curl", &["-s", &req.url]).await {
                        Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                            output.to_result_string(),
                        )])),
                        Err(e2) => Ok(CallToolResult::error(vec![Content::text(e2)])),
                    }
                } else {
                    Ok(CallToolResult::error(vec![Content::text(e)]))
                }
            }
        }
    }

    #[tool(
        description = "DNS lookup with dog (modern dig replacement) or dig fallback. \
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
            Err(e) => {
                // Fallback to dig
                if e.contains("not found") {
                    let mut dig_args = vec![req.domain.as_str()];
                    if let Some(ref rt) = req.record_type {
                        dig_args.push(rt);
                    }
                    dig_args.push("+short");
                    match self.executor.run("dig", &dig_args).await {
                        Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                            output.to_result_string(),
                        )])),
                        Err(e2) => Ok(CallToolResult::error(vec![Content::text(e2)])),
                    }
                } else {
                    Ok(CallToolResult::error(vec![Content::text(e)]))
                }
            }
        }
    }

    #[tool(description = "Execute SQL across multiple databases with usql. \
        Supports PostgreSQL, MySQL, SQLite, SQL Server, Oracle, and more.")]
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
    // UTILITY TOOLS
    // ========================================================================

    #[tool(description = "View file differences with delta or diff. \
        Features: syntax highlighting, side-by-side, word-level diffs.")]
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
                let mut delta_args = vec!["--color-only"];
                if req.side_by_side.unwrap_or(false) {
                    delta_args.push("--side-by-side");
                }
                if req.line_numbers.unwrap_or(true) {
                    delta_args.push("--line-numbers");
                }

                match self
                    .executor
                    .run_with_stdin("delta", &delta_args, &output.stdout)
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

    #[tool(description = "Git diff with syntax highlighting (uses delta if available).")]
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

    #[tool(description = "Run shell tests with bats (Bash Automated Testing System).")]
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

    #[tool(description = "Detect file type using magic bytes with file command.")]
    async fn file_type(
        &self,
        Parameters(req): Parameters<FileTypeRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.executor.run("file", &["-b", &req.path]).await {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Explain Unix file permissions in human readable format.")]
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

    #[tool(description = "Get simplified command help with tldr (tealdeer). \
        Shows practical examples for common commands.")]
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

    #[tool(description = "Generate regex from test strings with grex. \
        Provide example strings and get a regex that matches them.")]
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
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Batch find and replace across files with sad. \
        Like sed but for multiple files with preview support.")]
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

    #[tool(description = "Structural diff with difftastic (difft). \
        Understands code syntax for better diffs than line-based tools.")]
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

    #[tool(description = "Compress files with ouch. \
        Supports many formats: tar.gz, zip, 7z, xz, bz2, zstd, etc.")]
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

    #[tool(description = "Decompress archives with ouch. \
        Auto-detects format from file extension or magic bytes.")]
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

    #[tool(description = "List archive contents with ouch.")]
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

    #[tool(description = "Add task to pueue queue for background execution.")]
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

    #[tool(description = "Get pueue task queue status.")]
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

    #[tool(description = "Get logs from a pueue task.")]
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

    #[tool(description = "Search command cheatsheets with navi. \
        Interactive cheatsheet tool for command-line.")]
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

    #[tool(description = "GitHub repository operations via gh CLI. \
        Returns JSON for structured data. Subcommands: list, view, clone, create, fork, delete.")]
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

    #[tool(description = "GitHub issue operations. Returns JSON. \
        Subcommands: list, view, create, close, reopen, edit, comment.")]
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

    #[tool(description = "GitHub pull request operations. Returns JSON. \
        Subcommands: list, view, create, close, reopen, merge, checkout, diff, checks.")]
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

    #[tool(description = "GitHub search across repos, issues, PRs, code, commits. Returns JSON.")]
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

    #[tool(description = "GitHub release operations. Returns JSON for list/view.")]
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

    #[tool(description = "GitHub Actions workflow operations. Returns JSON.")]
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

    #[tool(description = "GitHub Actions workflow run operations. Returns JSON.")]
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

    #[tool(description = "Direct GitHub API access. Returns JSON. \
        Supports any API endpoint with optional jq filtering.")]
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

    #[tool(description = "GitLab issue operations via glab CLI. \
        Subcommands: list, view, create, close, reopen.")]
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

    #[tool(description = "GitLab merge request operations. \
        Subcommands: list, view, create, close, reopen, merge, approve.")]
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

    #[tool(description = "GitLab CI/CD pipeline operations. \
        Subcommands: list, view, run, cancel, retry, delete.")]
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

    #[tool(description = "Transform JSON to greppable format with gron. \
        Makes JSON amenable to grep. Use ungron to convert back.")]
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
                output.to_result_string(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Parse HTML with CSS selectors using pup. \
        Supports display filters like 'a attr{href}' or 'div text{}'.")]
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

    #[tool(description = "Process structured data with miller (mlr). \
        Like awk but for CSV, JSON, and other formats. Returns JSON by default.")]
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

    #[tool(description = "Analyze container image layers with dive. \
        CI mode returns efficiency score. JSON mode exports full analysis.")]
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

    #[tool(description = "Container registry operations with skopeo. \
        Inspect images without pulling, copy between registries. Returns JSON for inspect.")]
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

    #[tool(description = "Low-level container registry operations with crane. \
        Get digests, manifests, configs, list tags. Returns JSON.")]
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

    #[tool(description = "Security vulnerability scanner with trivy. \
        Scan images, filesystems, repos, configs. Returns JSON by default.")]
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

    // ========================================================================
    // KUBERNETES TOOLS
    // ========================================================================

    #[tool(
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

    #[tool(description = "Describe Kubernetes resource in detail. Returns human-readable text.")]
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

    #[tool(description = "Get logs from a Kubernetes pod.")]
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

    #[tool(description = "Apply Kubernetes manifest. Supports dry-run modes. \
        Pass YAML/JSON content directly.")]
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

    #[tool(description = "Delete a Kubernetes resource.")]
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

    #[tool(description = "Execute command in a Kubernetes pod container.")]
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

    #[tool(description = "Multi-pod log tailing with stern. \
        Aggregates logs from multiple pods matching a query. JSON output available.")]
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

    #[tool(description = "Helm chart operations. Returns JSON for list/status. \
        Subcommands: list, status, get, install, upgrade, uninstall, search, show, repo.")]
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

    #[tool(description = "Build and manage Kubernetes manifests with kustomize. \
        Build outputs YAML/JSON for applying to cluster.")]
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

#[tool_handler]
impl ServerHandler for ModernCliTools {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Modern CLI Tools MCP Server - Exposes modern command-line utilities \
                 like eza, bat, fd, rg, delta, jq, and many more for AI-assisted \
                 file operations, text processing, system monitoring, and development tasks."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
