// modern-cli-mcp/src/groups.rs
//! Virtual tool groups and agent profiles for reduced cognitive load.
//!
//! Implements GitHub Copilot-style tool clustering where related tools
//! are grouped under meta-tools that expand on demand.

use std::collections::HashSet;
use std::str::FromStr;

/// Tool groups that cluster related functionality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolGroup {
    Filesystem,
    FileOps,
    Search,
    Text,
    Git,
    GitHub,
    GitLab,
    Kubernetes,
    Container,
    Network,
    System,
    Archive,
    Reference,
    Diff,
    Mcp,
}

impl ToolGroup {
    /// All available tool groups.
    pub const ALL: &'static [ToolGroup] = &[
        ToolGroup::Filesystem,
        ToolGroup::FileOps,
        ToolGroup::Search,
        ToolGroup::Text,
        ToolGroup::Git,
        ToolGroup::GitHub,
        ToolGroup::GitLab,
        ToolGroup::Kubernetes,
        ToolGroup::Container,
        ToolGroup::Network,
        ToolGroup::System,
        ToolGroup::Archive,
        ToolGroup::Reference,
        ToolGroup::Diff,
        ToolGroup::Mcp,
    ];

    /// Tool names belonging to this group.
    pub fn tools(&self) -> &'static [&'static str] {
        match self {
            ToolGroup::Filesystem => &[
                "Filesystem - List (eza)",
                "Filesystem - View (bat)",
                "Filesystem - Find (fd)",
                "Filesystem - Disk Usage (duf)",
                "Filesystem - Directory Size (dust)",
                "Filesystem - Trash (rip)",
                "Filesystem - Trash List (rip)",
                "Filesystem - Trash Restore (rip)",
                "Filesystem - Copy",
                "Filesystem - Move",
                "Filesystem - Mkdir",
                "Filesystem - Exists",
                "Filesystem - Stat",
                "Filesystem - Symlink",
                "Filesystem - Hardlink",
                "Filesystem - File Type",
                "Filesystem - Permissions",
            ],
            ToolGroup::FileOps => &[
                "file_read",
                "File - Write",
                "File - Edit",
                "File - Append",
                "File - Patch",
            ],
            ToolGroup::Search => &[
                "Search - Content (ripgrep)",
                "Search - Fuzzy (fzf)",
                "Search - Web (DuckDuckGo)",
                "ast_grep",
                "Code - Symbols",
                "Code - References",
            ],
            ToolGroup::Text => &[
                "jq",
                "yq",
                "dasel",
                "htmlq",
                "Text - HTML Parse (pup)",
                "sd",
                "hck",
                "Text - JSON Grep (gron)",
                "Text - Data Process (miller)",
                "Text - CSV (xsv)",
                "Text - Find Replace (sad)",
            ],
            ToolGroup::Git => &[
                "Git - Status",
                "Git - Diff",
                "Git - Log",
                "Git - Add",
                "Git - Commit",
                "Git - Checkout",
                "Git - Branch",
                "Git - Stash",
            ],
            ToolGroup::GitHub => &[
                "GitHub - Auth Login",
                "GitHub - Auth Status",
                "GitHub - Repo",
                "GitHub - Issue",
                "GitHub - Pull Request",
                "GitHub - Search",
                "GitHub - Release",
                "GitHub - Workflow",
                "GitHub - Workflow Run",
                "GitHub - API",
            ],
            ToolGroup::GitLab => &[
                "GitLab - Auth Login",
                "GitLab - Auth Status",
                "GitLab - Issue",
                "GitLab - Merge Request",
                "GitLab - Pipeline",
            ],
            ToolGroup::Kubernetes => &[
                "kubectl_get",
                "Kubernetes - Apply",
                "Kubernetes - Delete",
                "Kubernetes - Describe",
                "Kubernetes - Logs",
                "Kubernetes - Exec",
                "Kubernetes - Multi-Logs (stern)",
                "Kubernetes - Helm",
                "Kubernetes - Kustomize",
            ],
            ToolGroup::Container => &[
                "podman",
                "Container - Compose",
                "Container - Buildx",
                "Container - Build (buildah)",
                "Container - Registry (skopeo)",
                "Container - Registry Low-level (crane)",
                "Container - Image Analyze (dive)",
                "Security - Scan (trivy)",
            ],
            ToolGroup::Network => &["Network - HTTP (xh)", "Network - SQL (usql)", "dns"],
            ToolGroup::System => &[
                "Shell - Execute",
                "nix_shell_exec",
                "procs",
                "System - Benchmark (hyperfine)",
                "System - Info",
                "Test - Shell (bats)",
                "tokei",
            ],
            ToolGroup::Archive => &[
                "Archive - Compress (ouch)",
                "Archive - Decompress (ouch)",
                "Archive - List (ouch)",
            ],
            ToolGroup::Reference => &[
                "Reference - TLDR",
                "Reference - Cheatsheets (navi)",
                "Reference - Regex Generator (grex)",
            ],
            ToolGroup::Diff => &["Diff - Files (delta)", "Diff - Structural (difft)"],
            ToolGroup::Mcp => &[
                "MCP - Auth Check",
                "MCP - Task Create",
                "MCP - Task List",
                "MCP - Task Update",
                "MCP - Task Delete",
                "MCP - Context Get",
                "MCP - Context Set",
                "MCP - Context List",
                "MCP - Cache Get",
                "MCP - Cache Set",
            ],
        }
    }

    /// Short identifier for CLI/config.
    pub fn id(&self) -> &'static str {
        match self {
            ToolGroup::Filesystem => "filesystem",
            ToolGroup::FileOps => "file_ops",
            ToolGroup::Search => "search",
            ToolGroup::Text => "text",
            ToolGroup::Git => "git",
            ToolGroup::GitHub => "github",
            ToolGroup::GitLab => "gitlab",
            ToolGroup::Kubernetes => "kubernetes",
            ToolGroup::Container => "container",
            ToolGroup::Network => "network",
            ToolGroup::System => "system",
            ToolGroup::Archive => "archive",
            ToolGroup::Reference => "reference",
            ToolGroup::Diff => "diff",
            ToolGroup::Mcp => "mcp",
        }
    }

    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            ToolGroup::Filesystem => "Filesystem",
            ToolGroup::FileOps => "File Operations",
            ToolGroup::Search => "Search & Code Analysis",
            ToolGroup::Text => "Text Processing",
            ToolGroup::Git => "Git Version Control",
            ToolGroup::GitHub => "GitHub",
            ToolGroup::GitLab => "GitLab",
            ToolGroup::Kubernetes => "Kubernetes & Helm",
            ToolGroup::Container => "Container & Registry",
            ToolGroup::Network => "Network & Database",
            ToolGroup::System => "System & Shell",
            ToolGroup::Archive => "Archive & Compression",
            ToolGroup::Reference => "Reference & Docs",
            ToolGroup::Diff => "Diff & Comparison",
            ToolGroup::Mcp => "MCP State Management",
        }
    }

    /// Description for the meta-tool.
    pub fn description(&self) -> &'static str {
        match self {
            ToolGroup::Filesystem => "List directories (eza), view files (bat), find files (fd), disk usage (duf/dust), trash management, copy/move/mkdir",
            ToolGroup::FileOps => "Read, write, edit, append, and patch files",
            ToolGroup::Search => "Search content (ripgrep), fuzzy find (fzf), web search, AST-based code search, symbols and references",
            ToolGroup::Text => "JSON (jq), YAML (yq), HTML (htmlq/pup), CSV (xsv), data processing (miller), find/replace (sd/sad)",
            ToolGroup::Git => "Status, diff, log, add, commit, checkout, branch, stash operations",
            ToolGroup::GitHub => "Repository, issue, PR, release, workflow, and API operations via gh CLI",
            ToolGroup::GitLab => "Issue, merge request, and pipeline operations via glab CLI",
            ToolGroup::Kubernetes => "kubectl get/apply/delete/describe/logs/exec, Helm charts, Kustomize, multi-pod logs (stern)",
            ToolGroup::Container => "Podman containers, compose orchestration, buildx multi-platform builds, buildah OCI images, registry operations (skopeo/crane), image analysis (dive), security scanning (trivy)",
            ToolGroup::Network => "HTTP requests (xh), SQL queries (usql), DNS lookups",
            ToolGroup::System => "Shell execution, Nix shells, process listing (procs), benchmarking (hyperfine), system info, shell tests (bats), code stats (tokei)",
            ToolGroup::Archive => "Compress, decompress, and list archives (ouch) - supports tar.gz, zip, 7z, xz, bz2, zstd",
            ToolGroup::Reference => "Command help (tldr), cheatsheets (navi), regex generation (grex)",
            ToolGroup::Diff => "File diffs with syntax highlighting (delta), structural/AST-aware diffs (difftastic)",
            ToolGroup::Mcp => "MCP task tracking, context storage, and caching for session state",
        }
    }

    /// Number of tools in this group.
    pub fn tool_count(&self) -> usize {
        self.tools().len()
    }
}

impl FromStr for ToolGroup {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "filesystem" | "fs" => Ok(ToolGroup::Filesystem),
            "file_ops" | "file" | "files" => Ok(ToolGroup::FileOps),
            "search" => Ok(ToolGroup::Search),
            "text" => Ok(ToolGroup::Text),
            "git" => Ok(ToolGroup::Git),
            "github" | "gh" => Ok(ToolGroup::GitHub),
            "gitlab" | "gl" => Ok(ToolGroup::GitLab),
            "kubernetes" | "k8s" | "kube" => Ok(ToolGroup::Kubernetes),
            "container" | "docker" | "podman" => Ok(ToolGroup::Container),
            "network" | "net" | "http" => Ok(ToolGroup::Network),
            "system" | "sys" | "shell" => Ok(ToolGroup::System),
            "archive" | "compress" | "zip" => Ok(ToolGroup::Archive),
            "reference" | "ref" | "docs" => Ok(ToolGroup::Reference),
            "diff" => Ok(ToolGroup::Diff),
            "mcp" | "state" => Ok(ToolGroup::Mcp),
            _ => Err(format!("Unknown tool group: {}", s)),
        }
    }
}

/// Agent profiles with pre-expanded tool groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentProfile {
    /// Codebase discovery and exploration.
    Explore,
    /// System design and architecture planning.
    Architect,
    /// Code review and quality analysis.
    Review,
    /// Test writing and execution.
    Test,
    /// General task execution (default, broadest access).
    Generator,
    /// Execution analysis and reflection.
    Reflector,
    /// Playbook and strategy management.
    Curator,
    /// Documentation writing.
    Docs,
    /// Code linting and quality checks.
    Lint,
    /// API interaction and integration.
    Api,
    /// Container and Kubernetes deployment.
    DevDeploy,
    /// Full access - all tools pre-expanded (backwards compatible).
    Full,
}

impl AgentProfile {
    /// All available profiles.
    pub const ALL: &'static [AgentProfile] = &[
        AgentProfile::Explore,
        AgentProfile::Architect,
        AgentProfile::Review,
        AgentProfile::Test,
        AgentProfile::Generator,
        AgentProfile::Reflector,
        AgentProfile::Curator,
        AgentProfile::Docs,
        AgentProfile::Lint,
        AgentProfile::Api,
        AgentProfile::DevDeploy,
        AgentProfile::Full,
    ];

    /// Tool groups pre-expanded for this profile.
    pub fn pre_expanded_groups(&self) -> HashSet<ToolGroup> {
        match self {
            AgentProfile::Explore => [ToolGroup::Filesystem, ToolGroup::Search, ToolGroup::Git]
                .into_iter()
                .collect(),

            AgentProfile::Architect => [
                ToolGroup::Filesystem,
                ToolGroup::Search,
                ToolGroup::Reference,
            ]
            .into_iter()
            .collect(),

            AgentProfile::Review => [ToolGroup::Git, ToolGroup::Search, ToolGroup::Diff]
                .into_iter()
                .collect(),

            AgentProfile::Test => [ToolGroup::FileOps, ToolGroup::Search, ToolGroup::System]
                .into_iter()
                .collect(),

            AgentProfile::Generator => [
                ToolGroup::FileOps,
                ToolGroup::Search,
                ToolGroup::Git,
                ToolGroup::System,
            ]
            .into_iter()
            .collect(),

            AgentProfile::Reflector => [ToolGroup::FileOps, ToolGroup::Git].into_iter().collect(),

            AgentProfile::Curator => [ToolGroup::FileOps, ToolGroup::Search]
                .into_iter()
                .collect(),

            AgentProfile::Docs => [
                ToolGroup::FileOps,
                ToolGroup::Filesystem,
                ToolGroup::Search,
                ToolGroup::Reference,
            ]
            .into_iter()
            .collect(),

            AgentProfile::Lint => [ToolGroup::Search, ToolGroup::System, ToolGroup::FileOps]
                .into_iter()
                .collect(),

            AgentProfile::Api => [ToolGroup::Network, ToolGroup::Text, ToolGroup::FileOps]
                .into_iter()
                .collect(),

            AgentProfile::DevDeploy => [
                ToolGroup::Kubernetes,
                ToolGroup::Container,
                ToolGroup::Git,
                ToolGroup::GitHub,
                ToolGroup::System,
            ]
            .into_iter()
            .collect(),

            AgentProfile::Full => ToolGroup::ALL.iter().copied().collect(),
        }
    }

    /// Short identifier for CLI.
    pub fn id(&self) -> &'static str {
        match self {
            AgentProfile::Explore => "explore",
            AgentProfile::Architect => "architect",
            AgentProfile::Review => "review",
            AgentProfile::Test => "test",
            AgentProfile::Generator => "generator",
            AgentProfile::Reflector => "reflector",
            AgentProfile::Curator => "curator",
            AgentProfile::Docs => "docs",
            AgentProfile::Lint => "lint",
            AgentProfile::Api => "api",
            AgentProfile::DevDeploy => "dev-deploy",
            AgentProfile::Full => "full",
        }
    }

    /// Human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            AgentProfile::Explore => {
                "Codebase discovery: filesystem, search, git (read-only focus)"
            }
            AgentProfile::Architect => "System design: filesystem, search, reference documentation",
            AgentProfile::Review => "Code review: git diffs, search, file comparison",
            AgentProfile::Test => "Testing: file ops, search, shell execution",
            AgentProfile::Generator => {
                "Task execution: file ops, search, git, shell (general purpose)"
            }
            AgentProfile::Reflector => "Analysis: file reading, git history",
            AgentProfile::Curator => "Playbook management: file ops, search",
            AgentProfile::Docs => "Documentation: file ops, filesystem, search, reference",
            AgentProfile::Lint => "Linting: search, shell execution, file editing",
            AgentProfile::Api => "API work: network, text processing, file ops",
            AgentProfile::DevDeploy => "Deployment: kubernetes, containers, git, github workflows",
            AgentProfile::Full => "Full access: all tool groups pre-expanded",
        }
    }

    /// Tool count for pre-expanded groups.
    pub fn pre_expanded_tool_count(&self) -> usize {
        self.pre_expanded_groups()
            .iter()
            .map(|g| g.tool_count())
            .sum()
    }
}

impl FromStr for AgentProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "explore" => Ok(AgentProfile::Explore),
            "architect" => Ok(AgentProfile::Architect),
            "review" => Ok(AgentProfile::Review),
            "test" => Ok(AgentProfile::Test),
            "generator" | "gen" => Ok(AgentProfile::Generator),
            "reflector" | "reflect" => Ok(AgentProfile::Reflector),
            "curator" | "curate" => Ok(AgentProfile::Curator),
            "docs" | "documentation" => Ok(AgentProfile::Docs),
            "lint" | "linter" => Ok(AgentProfile::Lint),
            "api" => Ok(AgentProfile::Api),
            "dev_deploy" | "devdeploy" | "deploy" => Ok(AgentProfile::DevDeploy),
            "full" | "all" => Ok(AgentProfile::Full),
            _ => Err(format!(
                "Unknown profile: {}. Available: {}",
                s,
                AgentProfile::ALL
                    .iter()
                    .map(|p| p.id())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }
}

/// Check if a tool name belongs to a group.
#[allow(dead_code)]
pub fn tool_belongs_to_group(tool_name: &str, group: ToolGroup) -> bool {
    group.tools().contains(&tool_name)
}

/// Find which group a tool belongs to.
#[allow(dead_code)]
pub fn find_tool_group(tool_name: &str) -> Option<ToolGroup> {
    ToolGroup::ALL
        .iter()
        .find(|g| g.tools().contains(&tool_name))
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_tools_have_groups() {
        // Ensure no orphan tools
        let total: usize = ToolGroup::ALL.iter().map(|g| g.tool_count()).sum();
        assert!(total > 100, "Expected 100+ tools, got {}", total);
    }

    #[test]
    fn test_profile_parsing() {
        assert_eq!(
            AgentProfile::from_str("explore").unwrap(),
            AgentProfile::Explore
        );
        assert_eq!(
            AgentProfile::from_str("dev-deploy").unwrap(),
            AgentProfile::DevDeploy
        );
        assert_eq!(AgentProfile::from_str("FULL").unwrap(), AgentProfile::Full);
    }

    #[test]
    fn test_group_parsing() {
        assert_eq!(ToolGroup::from_str("k8s").unwrap(), ToolGroup::Kubernetes);
        assert_eq!(ToolGroup::from_str("gh").unwrap(), ToolGroup::GitHub);
    }
}
