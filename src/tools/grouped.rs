// modern-cli-mcp/src/tools/grouped.rs
//! Grouped tool interface - 15 tools instead of 106.
//!
//! Each tool group becomes a single MCP tool with subcommands.

use rmcp::{model::CallToolResult, schemars, ErrorData};
use serde::Deserialize;

// ============================================================================
// FILESYSTEM GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FilesystemParams {
    #[schemars(description = "Subcommand: list, view, find, disk_usage, dir_size, trash, trash_list, trash_restore, copy, move, mkdir, stat, exists, symlink, hardlink, file_type, permissions")]
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

    // find (fd) options
    #[schemars(description = "[find] Search pattern (regex)")]
    pub pattern: Option<String>,
    #[schemars(description = "[find] File extension filter")]
    pub extension: Option<String>,
    #[schemars(description = "[find] Type: f(ile), d(irectory), l(ink), x(executable)")]
    pub file_type: Option<String>,
    #[schemars(description = "[find] Maximum search depth")]
    pub max_depth: Option<u32>,
    #[schemars(description = "[find] Include hidden files")]
    pub hidden: Option<bool>,

    // view (bat) options
    #[schemars(description = "[view] Line range (e.g., '10:20')")]
    pub range: Option<String>,
    #[schemars(description = "[view] Language for syntax highlighting")]
    pub language: Option<String>,
    #[schemars(description = "[view] Show line numbers")]
    pub number: Option<bool>,

    // copy/move options
    #[schemars(description = "[copy/move] Destination path")]
    pub dest: Option<String>,
    #[schemars(description = "[copy/move/symlink/hardlink] Backup dest to graveyard before overwriting")]
    pub safe_overwrite: Option<bool>,

    // symlink/hardlink options
    #[schemars(description = "[symlink] Target path (what link points to)")]
    pub target: Option<String>,
    #[schemars(description = "[symlink/hardlink] Link path to create")]
    pub link: Option<String>,

    // trash options
    #[schemars(description = "[trash] Custom graveyard directory")]
    pub graveyard: Option<String>,

    // permissions
    #[schemars(description = "[permissions] Mode to explain (e.g., '755')")]
    pub mode: Option<String>,
}

// ============================================================================
// GIT GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitParams {
    #[schemars(description = "Subcommand: status, diff, log, add, commit, checkout, branch, stash")]
    pub command: String,

    #[schemars(description = "Working directory path")]
    pub path: Option<String>,

    // status options
    #[schemars(description = "[status] Short format")]
    pub short: Option<bool>,

    // diff options
    #[schemars(description = "[diff] Show staged changes")]
    pub staged: Option<bool>,
    #[schemars(description = "[diff] Compare with specific commit")]
    pub commit: Option<String>,
    #[schemars(description = "[diff] Specific file to diff")]
    pub file: Option<String>,

    // log options
    #[schemars(description = "[log] Number of commits to show")]
    pub count: Option<u32>,
    #[schemars(description = "[log] One line per commit")]
    pub oneline: Option<bool>,

    // add options
    #[schemars(description = "[add] Files to stage (space-separated, or '.' for all)")]
    pub files: Option<String>,

    // commit options
    #[schemars(description = "[commit] Commit message")]
    pub message: Option<String>,
    #[schemars(description = "[commit] Stage all modified files")]
    pub all: Option<bool>,

    // checkout/branch options
    #[schemars(description = "[checkout/branch] Branch name or commit")]
    pub target: Option<String>,
    #[schemars(description = "[checkout] Create new branch")]
    pub create: Option<bool>,
    #[schemars(description = "[branch] Subcommand: list, create, delete, rename")]
    pub branch_command: Option<String>,

    // stash options
    #[schemars(description = "[stash] Subcommand: push, pop, list, drop, apply, show")]
    pub stash_command: Option<String>,
    #[schemars(description = "[stash] Stash message")]
    pub stash_message: Option<String>,
}

// ============================================================================
// GITHUB GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GithubParams {
    #[schemars(description = "Subcommand: repo, issue, pr, search, release, workflow, run, api, auth_status, auth_login")]
    pub command: String,

    #[schemars(description = "Repository in owner/repo format")]
    pub repo: Option<String>,

    // issue/pr options
    #[schemars(description = "[issue/pr] Issue or PR number")]
    pub number: Option<u32>,
    #[schemars(description = "[issue/pr] State filter: open, closed, all")]
    pub state: Option<String>,
    #[schemars(description = "[issue/pr] Title for create")]
    pub title: Option<String>,
    #[schemars(description = "[issue/pr] Body for create")]
    pub body: Option<String>,

    // pr specific
    #[schemars(description = "[pr] Base branch")]
    pub base: Option<String>,
    #[schemars(description = "[pr] Head branch")]
    pub head: Option<String>,
    #[schemars(description = "[pr] Merge method: merge, squash, rebase")]
    pub merge_method: Option<String>,

    // search
    #[schemars(description = "[search] Search query")]
    pub query: Option<String>,
    #[schemars(description = "[search] Type: repos, issues, prs, code, commits")]
    pub search_type: Option<String>,

    // workflow/run
    #[schemars(description = "[workflow/run] Workflow ID or filename")]
    pub workflow: Option<String>,
    #[schemars(description = "[run] Run ID")]
    pub run_id: Option<u64>,

    // api
    #[schemars(description = "[api] API endpoint")]
    pub endpoint: Option<String>,
    #[schemars(description = "[api] HTTP method")]
    pub method: Option<String>,

    // common
    #[schemars(description = "Maximum results")]
    pub limit: Option<u32>,
}

// ============================================================================
// SEARCH GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    #[schemars(description = "Subcommand: content, fuzzy, web, ast, symbols, references")]
    pub command: String,

    #[schemars(description = "Search pattern")]
    pub pattern: String,

    #[schemars(description = "Path to search in")]
    pub path: Option<String>,

    // content (ripgrep) options
    #[schemars(description = "[content] Case-insensitive")]
    pub ignore_case: Option<bool>,
    #[schemars(description = "[content] File type (e.g., 'rust', 'py')")]
    pub file_type: Option<String>,
    #[schemars(description = "[content] Context lines")]
    pub context: Option<u32>,
    #[schemars(description = "[content] Only show filenames")]
    pub files_only: Option<bool>,

    // fuzzy (fzf) options
    #[schemars(description = "[fuzzy] Input to filter (newline-separated)")]
    pub input: Option<String>,
    #[schemars(description = "[fuzzy] Exact match (no fuzzy)")]
    pub exact: Option<bool>,

    // web options
    #[schemars(description = "[web] Number of results")]
    pub num_results: Option<u32>,

    // ast options
    #[schemars(description = "[ast/symbols] Language")]
    pub lang: Option<String>,

    // symbols options
    #[schemars(description = "[references] Symbol name")]
    pub symbol: Option<String>,
}

// ============================================================================
// KUBERNETES GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KubernetesParams {
    #[schemars(description = "Subcommand: get, apply, delete, describe, logs, exec, helm, kustomize, multi_logs")]
    pub command: String,

    #[schemars(description = "Resource type: pods, deployments, services, configmaps, etc.")]
    pub resource: Option<String>,

    #[schemars(description = "Resource name")]
    pub name: Option<String>,

    #[schemars(description = "Namespace")]
    pub namespace: Option<String>,

    // get options
    #[schemars(description = "[get] All namespaces")]
    pub all_namespaces: Option<bool>,
    #[schemars(description = "[get] Label selector")]
    pub selector: Option<String>,
    #[schemars(description = "[get] Output format: json, yaml, wide")]
    pub output: Option<String>,

    // apply options
    #[schemars(description = "[apply] YAML/JSON manifest content")]
    pub manifest: Option<String>,
    #[schemars(description = "[apply] Dry run: none, client, server")]
    pub dry_run: Option<String>,

    // logs options
    #[schemars(description = "[logs] Pod name")]
    pub pod: Option<String>,
    #[schemars(description = "[logs] Container name")]
    pub container: Option<String>,
    #[schemars(description = "[logs] Number of lines")]
    pub tail: Option<u32>,
    #[schemars(description = "[logs] Show since duration (e.g., '1h')")]
    pub since: Option<String>,

    // exec options
    #[schemars(description = "[exec] Command to execute")]
    pub exec_command: Option<String>,

    // helm options
    #[schemars(description = "[helm] Helm subcommand: list, status, install, upgrade, uninstall")]
    pub helm_command: Option<String>,
    #[schemars(description = "[helm] Release name")]
    pub release: Option<String>,
    #[schemars(description = "[helm] Chart reference")]
    pub chart: Option<String>,

    // kustomize options
    #[schemars(description = "[kustomize] Path to kustomization directory")]
    pub kustomize_path: Option<String>,

    // multi_logs (stern) options
    #[schemars(description = "[multi_logs] Pod query (regex)")]
    pub query: Option<String>,
}

// ============================================================================
// FILE OPERATIONS GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FileOpsParams {
    #[schemars(description = "Subcommand: read, write, edit, append, patch")]
    pub command: String,

    #[schemars(description = "Absolute file path")]
    pub path: String,

    // read options
    #[schemars(description = "[read] Starting line (1-indexed)")]
    pub offset: Option<u32>,
    #[schemars(description = "[read] Number of lines to read")]
    pub limit: Option<u32>,

    // write/append options
    #[schemars(description = "[write/append] Content to write")]
    pub content: Option<String>,
    #[schemars(description = "[write] Create parent directories")]
    pub create_dirs: Option<bool>,
    #[schemars(description = "[write] Backup to graveyard before overwriting")]
    pub safe_overwrite: Option<bool>,

    // edit options
    #[schemars(description = "[edit] Text to find (must be unique)")]
    pub old_text: Option<String>,
    #[schemars(description = "[edit] Replacement text")]
    pub new_text: Option<String>,
    #[schemars(description = "[edit] Replace all occurrences")]
    pub replace_all: Option<bool>,
    #[schemars(description = "[edit] Backup before editing")]
    pub backup: Option<bool>,

    // patch options
    #[schemars(description = "[patch] Unified diff patch content")]
    pub patch: Option<String>,
}

// ============================================================================
// TEXT PROCESSING GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TextParams {
    #[schemars(description = "Subcommand: jq, yq, dasel, htmlq, pup, sd, hck, gron, miller, csv, find_replace")]
    pub command: String,

    #[schemars(description = "Input data")]
    pub input: String,

    // jq/yq/dasel options
    #[schemars(description = "[jq/yq] Filter/expression")]
    pub filter: Option<String>,
    #[schemars(description = "[dasel] Selector query")]
    pub selector: Option<String>,

    // format options
    #[schemars(description = "Input format: json, yaml, toml, xml, csv")]
    pub input_format: Option<String>,
    #[schemars(description = "Output format: json, yaml, toml, xml, csv")]
    pub output_format: Option<String>,

    // htmlq/pup options
    #[schemars(description = "[htmlq/pup] CSS selector")]
    pub css_selector: Option<String>,
    #[schemars(description = "[htmlq] Extract text only")]
    pub text: Option<bool>,
    #[schemars(description = "[htmlq] Extract attribute")]
    pub attribute: Option<String>,

    // sd/find_replace options
    #[schemars(description = "[sd/find_replace] Pattern to find")]
    pub find: Option<String>,
    #[schemars(description = "[sd/find_replace] Replacement")]
    pub replace: Option<String>,
    #[schemars(description = "[sd/find_replace] Fixed string (not regex)")]
    pub fixed: Option<bool>,

    // hck options
    #[schemars(description = "[hck] Fields to extract (e.g., '1,3')")]
    pub fields: Option<String>,
    #[schemars(description = "[hck/csv] Delimiter")]
    pub delimiter: Option<String>,

    // miller/csv options
    #[schemars(description = "[miller] Verb: cat, cut, sort, filter, stats1")]
    pub verb: Option<String>,
    #[schemars(description = "[csv] Command: stats, select, search, sort")]
    pub csv_command: Option<String>,
}

// ============================================================================
// NETWORK GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NetworkParams {
    #[schemars(description = "Subcommand: http, sql, dns")]
    pub command: String,

    // http options
    #[schemars(description = "[http] URL to request")]
    pub url: Option<String>,
    #[schemars(description = "[http] HTTP method: GET, POST, PUT, DELETE")]
    pub method: Option<String>,
    #[schemars(description = "[http] Request body (JSON)")]
    pub body: Option<String>,
    #[schemars(description = "[http] Headers as JSON object")]
    pub headers: Option<String>,
    #[schemars(description = "[http] Bearer token")]
    pub bearer: Option<String>,

    // sql options
    #[schemars(description = "[sql] Database URL (postgres://, mysql://, sqlite:)")]
    pub db_url: Option<String>,
    #[schemars(description = "[sql] SQL query")]
    pub query: Option<String>,

    // dns options
    #[schemars(description = "[dns] Domain to query")]
    pub domain: Option<String>,
    #[schemars(description = "[dns] Record type: A, AAAA, MX, NS, TXT")]
    pub record_type: Option<String>,
}

// ============================================================================
// CONTAINER GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ContainerParams {
    #[schemars(description = "Subcommand: podman, registry, registry_low, analyze, scan")]
    pub command: String,

    #[schemars(description = "Image name or reference")]
    pub image: Option<String>,

    // podman options
    #[schemars(description = "[podman] Podman subcommand: ps, images, run, stop, rm")]
    pub podman_command: Option<String>,
    #[schemars(description = "[podman] Container/image target")]
    pub target: Option<String>,
    #[schemars(description = "[podman] Show all")]
    pub all: Option<bool>,

    // registry (skopeo) options
    #[schemars(description = "[registry] Operation: inspect, copy, list-tags")]
    pub registry_command: Option<String>,
    #[schemars(description = "[registry] Source image reference")]
    pub source: Option<String>,
    #[schemars(description = "[registry] Destination image reference")]
    pub dest: Option<String>,

    // scan (trivy) options
    #[schemars(description = "[scan] Scan type: image, fs, repo")]
    pub scan_type: Option<String>,
    #[schemars(description = "[scan] Severity filter: HIGH,CRITICAL")]
    pub severity: Option<String>,
}

// ============================================================================
// SYSTEM GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SystemParams {
    #[schemars(description = "Subcommand: shell, nix_shell, procs, benchmark, info, test, tokei")]
    pub command: String,

    // shell options
    #[schemars(description = "[shell/nix_shell] Command to execute")]
    pub exec_command: Option<String>,
    #[schemars(description = "[shell] Shell: bash, zsh, fish, nu, dash")]
    pub shell: Option<String>,
    #[schemars(description = "[shell] Working directory")]
    pub working_dir: Option<String>,
    #[schemars(description = "[shell] Timeout in seconds")]
    pub timeout: Option<u64>,

    // nix_shell options
    #[schemars(description = "[nix_shell] Flake reference")]
    pub flake: Option<String>,
    #[schemars(description = "[nix_shell] Devshell name")]
    pub devshell: Option<String>,

    // procs options
    #[schemars(description = "[procs] Filter by keyword")]
    pub keyword: Option<String>,
    #[schemars(description = "[procs] Show tree view")]
    pub tree: Option<bool>,

    // benchmark options
    #[schemars(description = "[benchmark] Command to benchmark")]
    pub bench_command: Option<String>,
    #[schemars(description = "[benchmark] Warmup runs")]
    pub warmup: Option<u32>,

    // test (bats) options
    #[schemars(description = "[test] Test file or directory")]
    pub test_path: Option<String>,

    // tokei options
    #[schemars(description = "[tokei] Path to analyze")]
    pub path: Option<String>,
}

// ============================================================================
// DIFF GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DiffParams {
    #[schemars(description = "Subcommand: files, structural")]
    pub command: String,

    #[schemars(description = "First file path")]
    pub file_a: String,

    #[schemars(description = "Second file path")]
    pub file_b: String,

    #[schemars(description = "[structural] Override language detection")]
    pub language: Option<String>,

    #[schemars(description = "Context lines around changes")]
    pub context: Option<u32>,
}

// ============================================================================
// REFERENCE GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReferenceParams {
    #[schemars(description = "Subcommand: tldr, cheatsheet, regex")]
    pub command: String,

    // tldr options
    #[schemars(description = "[tldr] Command name to get help for")]
    pub cmd: Option<String>,

    // cheatsheet (navi) options
    #[schemars(description = "[cheatsheet] Search query")]
    pub query: Option<String>,

    // regex (grex) options
    #[schemars(description = "[regex] Test strings (one per line)")]
    pub input: Option<String>,
    #[schemars(description = "[regex] Use anchors")]
    pub anchors: Option<bool>,
}

// ============================================================================
// ARCHIVE GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ArchiveParams {
    #[schemars(description = "Subcommand: compress, decompress, list")]
    pub command: String,

    #[schemars(description = "Archive file path")]
    pub archive: Option<String>,

    #[schemars(description = "[compress] Files to compress (comma-separated)")]
    pub files: Option<String>,

    #[schemars(description = "[compress] Output archive path")]
    pub output: Option<String>,

    #[schemars(description = "[decompress] Output directory")]
    pub output_dir: Option<String>,
}

// ============================================================================
// MCP STATE GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct McpParams {
    #[schemars(description = "Subcommand: task_create, task_list, task_update, task_delete, context_get, context_set, context_list, cache_get, cache_set, auth_check")]
    pub command: String,

    // task options
    #[schemars(description = "[task] Task ID")]
    pub task_id: Option<i64>,
    #[schemars(description = "[task_create] Task content")]
    pub content: Option<String>,
    #[schemars(description = "[task_update] Status: pending, in_progress, completed")]
    pub status: Option<String>,

    // context options
    #[schemars(description = "[context] Key")]
    pub key: Option<String>,
    #[schemars(description = "[context_set] Value")]
    pub value: Option<String>,
    #[schemars(description = "[context] Scope: session, project, global")]
    pub scope: Option<String>,

    // cache options
    #[schemars(description = "[cache_set] TTL in seconds")]
    pub ttl_secs: Option<i64>,
}

// ============================================================================
// GITLAB GROUP
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitlabParams {
    #[schemars(description = "Subcommand: issue, mr, pipeline, auth_status, auth_login")]
    pub command: String,

    #[schemars(description = "Project path (group/project)")]
    pub project: Option<String>,

    // issue/mr options
    #[schemars(description = "[issue/mr] IID")]
    pub iid: Option<u32>,
    #[schemars(description = "[issue/mr] State: opened, closed, all")]
    pub state: Option<String>,
    #[schemars(description = "[issue/mr] Title")]
    pub title: Option<String>,
    #[schemars(description = "[issue/mr] Description")]
    pub description: Option<String>,

    // mr specific
    #[schemars(description = "[mr] Source branch")]
    pub source_branch: Option<String>,
    #[schemars(description = "[mr] Target branch")]
    pub target_branch: Option<String>,

    // pipeline options
    #[schemars(description = "[pipeline] Pipeline ID")]
    pub pipeline_id: Option<u64>,
    #[schemars(description = "[pipeline] Subcommand: list, view, run")]
    pub pipeline_command: Option<String>,
}
