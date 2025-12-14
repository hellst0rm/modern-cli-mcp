# Virtual Tool Groups Design

## Overview

Implement GitHub Copilot-style virtual tool groups to reduce cognitive load on models while maintaining full capability. Instead of exposing 107 tools, expose ~10 group meta-tools that expand on demand.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     TOOL EXPOSURE LAYER                         │
├─────────────────────────────────────────────────────────────────┤
│  Initial View (10 meta-tools):                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │filesystem│ │  search  │ │   git    │ │  github  │ ...       │
│  │  _tools  │ │  _tools  │ │  _tools  │ │  _tools  │           │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘           │
│       │            │            │            │                  │
│       ▼            ▼            ▼            ▼                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              EXPANDED TOOLS (on demand)                  │   │
│  │  eza, bat, fd, dust, duf, trash, copy, move, mkdir...   │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Tool Groups (10 categories → 107 tools)

| Group | Meta-tool | Tools | Description |
|-------|-----------|-------|-------------|
| **filesystem** | `filesystem_tools` | 17 | eza, bat, fd, duf, dust, trash×3, copy, move, mkdir, exists, stat, symlink, hardlink, file_type, permissions |
| **file_ops** | `file_tools` | 5 | read, write, edit, append, patch |
| **search** | `search_tools` | 5 | ripgrep, fzf, ast-grep, symbols, references |
| **text** | `text_tools` | 8 | jq, yq, dasel, htmlq, pup, sd, hck, gron, miller, xsv |
| **git** | `git_tools` | 8 | status, diff, log, add, commit, checkout, branch, stash |
| **github** | `github_tools` | 10 | repo, issue, pr, search, release, workflow×2, api, auth×2 |
| **gitlab** | `gitlab_tools` | 5 | auth×2, issue, mr, pipeline |
| **kubernetes** | `kubernetes_tools` | 8 | get, apply, delete, describe, logs, exec, helm, kustomize, stern |
| **container** | `container_tools` | 5 | podman, skopeo, crane, dive, trivy |
| **network** | `network_tools` | 3 | xh, usql, dns |
| **system** | `system_tools` | 6 | shell-exec, nix-shell, procs, hyperfine, system-info, bats |
| **archive** | `archive_tools` | 3 | compress, decompress, list |
| **reference** | `reference_tools` | 3 | tldr, navi, grex |
| **diff** | `diff_tools` | 2 | delta, difft |
| **mcp** | `mcp_tools` | 10 | task×4, context×3, cache×2, auth |

## Agent Profiles (pre-expanded groups)

Each profile defines which groups are **pre-expanded** (tools visible immediately):

```rust
pub enum AgentProfile {
    Explore,      // filesystem, search, git (read-only)
    Architect,    // filesystem, search, code analysis
    Review,       // git, search, diff
    Test,         // file_ops, search, system (shell)
    Generator,    // ALL groups available, core pre-expanded
    Reflector,    // file_ops (read), search, git (log)
    Curator,      // file_ops, search
    Docs,         // file_ops, filesystem, search
    Lint,         // search, system (shell), file_ops
    Api,          // network, text, file_ops
    DevDeploy,    // kubernetes, container, git, github
}
```

### Profile Tool Mappings

| Profile | Pre-expanded Groups | Virtual Groups |
|---------|---------------------|----------------|
| `explore` | filesystem, search, git | file_ops, text, diff |
| `architect` | filesystem, search | git, text, reference |
| `review` | git, search, diff | file_ops, github |
| `test` | file_ops, search, system | git, text |
| `generator` | file_ops, search, git, system | ALL others |
| `reflector` | file_ops, git | search |
| `curator` | file_ops, search | - |
| `docs` | file_ops, filesystem, search | reference |
| `lint` | search, system, file_ops | git |
| `api` | network, text, file_ops | search |
| `dev-deploy` | kubernetes, container, git, github | system, file_ops |

## Implementation

### 1. ToolGroup enum

```rust
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
    pub fn tools(&self) -> &'static [&'static str] {
        match self {
            Self::Filesystem => &["eza", "bat", "fd", "duf", "dust", ...],
            Self::Git => &["git_status", "git_diff", "git_log", ...],
            // ...
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Filesystem => "File listing, finding, disk usage, trash management",
            Self::Git => "Version control: status, diff, log, branches, stash",
            // ...
        }
    }
}
```

### 2. Meta-tool for expansion

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExpandGroupRequest {
    #[schemars(description = "Tool group to expand: filesystem, file_ops, search, text, git, github, gitlab, kubernetes, container, network, system, archive, reference, diff, mcp")]
    pub group: String,
}

#[tool(
    name = "expand_tools",
    description = "Expand a tool group to see available tools. Groups: filesystem (file listing/finding), file_ops (read/write/edit), search (ripgrep/ast-grep), text (jq/yq/csv), git (version control), github (issues/PRs), gitlab, kubernetes (k8s/helm), container (podman/registry), network (HTTP/SQL), system (shell/benchmarks), archive (compress), reference (tldr/cheatsheets), diff (file comparison), mcp (task/context management)"
)]
async fn expand_tools(&self, req: ExpandGroupRequest) -> Result<CallToolResult, ErrorData> {
    let group = ToolGroup::from_str(&req.group)?;
    let tools = group.tools();
    let descriptions = tools.iter()
        .map(|t| format!("- {}: {}", t, self.get_tool_description(t)))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(CallToolResult::success(vec![Content::text(format!(
        "## {} Tools\n\n{}\n\nCall any tool directly by name.",
        group.name(), descriptions
    ))]))
}
```

### 3. Tool visibility state

```rust
pub struct ModernCliTools {
    tool_router: ToolRouter<Self>,
    executor: CommandExecutor,
    state: Arc<StateManager>,
    expanded_groups: Arc<RwLock<HashSet<ToolGroup>>>,  // NEW
    profile: Option<AgentProfile>,                      // NEW
}

impl ModernCliTools {
    pub fn new() -> Self { /* all groups collapsed */ }

    pub fn with_profile(profile: AgentProfile) -> Self {
        let mut tools = Self::new();
        for group in profile.pre_expanded_groups() {
            tools.expanded_groups.write().insert(group);
        }
        tools
    }
}
```

### 4. Dynamic tool list

Override `list_tools` to return only:
1. All meta-tools (expand_tools)
2. Tools from expanded groups
3. Tools from pre-expanded groups (based on profile)

```rust
#[tool_handler]
impl ServerHandler for ModernCliTools {
    async fn list_tools(&self) -> Result<Vec<Tool>, ErrorData> {
        let expanded = self.expanded_groups.read();
        let mut tools = vec![self.expand_tools_meta()];

        for group in ToolGroup::all() {
            if expanded.contains(&group) {
                tools.extend(group.tool_definitions());
            }
        }

        Ok(tools)
    }
}
```

## CLI Interface

```bash
# Default: all groups collapsed, only expand_tools visible
modern-cli-mcp

# With profile: pre-expands relevant groups
modern-cli-mcp --profile explore
modern-cli-mcp --profile dev-deploy

# List profiles
modern-cli-mcp --list-profiles
```

## MCP Configuration Example

```json
{
  "mcpServers": {
    "cli": {
      "command": "modern-cli-mcp",
      "args": ["--profile", "generator"]
    },
    "cli-explore": {
      "command": "modern-cli-mcp",
      "args": ["--profile", "explore"]
    },
    "cli-deploy": {
      "command": "modern-cli-mcp",
      "args": ["--profile", "dev-deploy"]
    }
  }
}
```

## Benefits

1. **Reduced cognitive load**: Model sees 10-15 tools instead of 107
2. **Semantic clustering**: Related tools grouped together
3. **Profile optimization**: Sub-agents get only relevant tools
4. **Cache efficiency**: Related tools expand together (matches usage patterns)
5. **Backwards compatible**: `--profile generator` or no args gives full access

## Migration Path

1. Add `ToolGroup` enum and mappings
2. Add `expand_tools` meta-tool
3. Add `--profile` CLI flag
4. Keep all existing tools working (no breaking changes)
5. Default behavior unchanged (all tools available)
