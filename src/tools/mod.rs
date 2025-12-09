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
            .split(|c| c == '\n' || c == ',')
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
