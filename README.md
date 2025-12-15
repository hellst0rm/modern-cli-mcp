# Modern CLI MCP Server

[![CI](https://github.com/hellst0rm/modern-cli-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/hellst0rm/modern-cli-mcp/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/hellst0rm/modern-cli-mcp)](https://github.com/hellst0rm/modern-cli-mcp/releases)
[![Docker](https://img.shields.io/badge/Docker-ghcr.io-blue)](https://ghcr.io/hellst0rm/modern-cli-mcp)
[![FlakeHub](https://img.shields.io/endpoint?url=https://flakehub.com/f/hellst0rm/modern-cli-mcp/badge)](https://flakehub.com/flake/hellst0rm/modern-cli-mcp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**[Documentation](https://hellst0rm.github.io/modern-cli-mcp/)** · **[Tools](https://hellst0rm.github.io/modern-cli-mcp/tools.html)**

MCP server exposing **107 modern CLI tools** to AI/LLM agents. Provides structured JSON-RPC access to filesystem, Git forges, containers, Kubernetes, and data transformation tools—all optimized for AI consumption with JSON output.

## Features

- **107 Tools in 15 Groups**: Filesystem, search, Git (GitHub/GitLab), containers, Kubernetes, data processing
- **AI-Optimized Output**: JSON by default for structured parsing
- **Access Control**: `.agentignore` files to control which files AI agents can access
- **Zero Config**: Nix bundles all dependencies—no manual tool installation

## Quick Start

### Claude Desktop / Claude Code

Add to your MCP configuration:

**Option 1: Nix (Recommended)**
```json
{
  "mcpServers": {
    "modern-cli": {
      "command": "nix",
      "args": ["run", "github:hellst0rm/modern-cli-mcp", "--"]
    }
  }
}
```

**Option 2: Docker**
```json
{
  "mcpServers": {
    "modern-cli": {
      "command": "docker",
      "args": ["run", "--rm", "-i", "ghcr.io/hellst0rm/modern-cli-mcp"]
    }
  }
}
```

**Option 3: Binary**
```json
{
  "mcpServers": {
    "modern-cli": {
      "command": "/path/to/modern-cli-mcp"
    }
  }
}
```

## Available Tools (70+)

### Filesystem
| Tool | Description |
|------|-------------|
| `eza` | Modern ls with icons and git integration |
| `bat` | Cat with syntax highlighting |
| `fd` | Fast find alternative |
| `duf` | Disk usage viewer (JSON) |
| `dust` | Directory size analyzer |
| `trash_*` | Safe file deletion |

### Search
| Tool | Description |
|------|-------------|
| `rg` | Ripgrep for fast content search (JSON) |
| `fzf_filter` | Fuzzy filtering |
| `ast_grep` | AST-based code search (JSON) |
| `web_search` | DuckDuckGo web search (JSON) |

### Git Forges
| Tool | Description |
|------|-------------|
| `gh_repo` | GitHub repository operations (JSON) |
| `gh_issue` | GitHub issues (JSON) |
| `gh_pr` | GitHub pull requests (JSON) |
| `gh_search` | Search repos, issues, PRs, code (JSON) |
| `gh_release` | GitHub releases (JSON) |
| `gh_workflow` | GitHub Actions workflows (JSON) |
| `gh_run` | GitHub Actions runs (JSON) |
| `gh_api` | Direct GitHub API access (JSON) |
| `glab_issue` | GitLab issues (JSON) |
| `glab_mr` | GitLab merge requests (JSON) |
| `glab_pipeline` | GitLab CI/CD pipelines (JSON) |

### Containers
| Tool | Description |
|------|-------------|
| `podman` | Container operations (JSON) |
| `dive` | Image layer analysis |
| `skopeo` | Registry operations (JSON) |
| `crane` | Low-level registry tool (JSON) |
| `trivy` | Security vulnerability scanner (JSON) |

### Kubernetes
| Tool | Description |
|------|-------------|
| `kubectl_get` | Get resources (JSON) |
| `kubectl_describe` | Describe resources |
| `kubectl_logs` | Pod logs |
| `kubectl_apply` | Apply manifests |
| `kubectl_delete` | Delete resources |
| `kubectl_exec` | Execute in pods |
| `stern` | Multi-pod log aggregation (JSON) |
| `helm` | Chart management (JSON) |
| `kustomize` | Manifest building |

### Data Transformation
| Tool | Description |
|------|-------------|
| `jq` | JSON processor |
| `yq` | YAML/JSON/XML processor |
| `gron` | JSON→greppable format |
| `htmlq` | jq for HTML |
| `pup` | HTML parser (JSON) |
| `miller` | Multi-format processor |
| `dasel` | Universal data selector |

### Network
| Tool | Description |
|------|-------------|
| `http` | HTTP requests (xh) |
| `dns` | DNS lookups (doggo) |
| `usql` | Universal SQL client |

### System
| Tool | Description |
|------|-------------|
| `procs` | Process viewer (JSON) |
| `tokei` | Code statistics (JSON) |
| `hyperfine` | Benchmarking (JSON) |

### Diff/Git
| Tool | Description |
|------|-------------|
| `delta` | Syntax-highlighted diffs |
| `difft` | Structural diff |
| `git_diff` | Git diff with highlighting |

### Utilities
| Tool | Description |
|------|-------------|
| `tldr` | Command cheatsheets |
| `grex` | Regex generator |
| `ouch_*` | Archive handling |
| `pueue_*` | Task queue |

## Installation

### Nix with Flakes (Recommended)

Flakes are enabled by default in [Determinate Nix](https://determinate.systems/nix-installer/). For standard Nix, enable with `--experimental-features 'nix-command flakes'` or add to `~/.config/nix/nix.conf`:

```bash
experimental-features = nix-command flakes
```

**Run directly (no installation):**
```bash
nix run github:hellst0rm/modern-cli-mcp
```

**Install to profile:**
```bash
nix profile install github:hellst0rm/modern-cli-mcp
```

**Add to flake.nix:**
```nix
{
  inputs.modern-cli-mcp.url = "github:hellst0rm/modern-cli-mcp";

  # Use in outputs
  outputs = { self, nixpkgs, modern-cli-mcp }: {
    # Add to packages, devShells, etc.
  };
}
```

### Nix with FlakeHub

[FlakeHub](https://flakehub.com) provides versioned flake references with semantic versioning.

**Add to flake.nix via CLI:**
```bash
fh add hellst0rm/modern-cli-mcp
```

**Or manually:**
```nix
{
  inputs.modern-cli-mcp.url = "https://flakehub.com/f/hellst0rm/modern-cli-mcp/*.tar.gz";
}
```

**Run specific version:**
```bash
nix run "https://flakehub.com/f/hellst0rm/modern-cli-mcp/0.2.tar.gz"
```

### Nix without Flakes (Classic)

For Nix installations without flakes enabled, the repository includes a `default.nix` via [flake-compat](https://github.com/edolstra/flake-compat):

**Build from tarball:**
```bash
nix-build https://github.com/hellst0rm/modern-cli-mcp/archive/main.tar.gz -A defaultNix.default
./result/bin/modern-cli-mcp
```

**Or clone and build locally:**
```bash
git clone https://github.com/hellst0rm/modern-cli-mcp
cd modern-cli-mcp
nix-build
./result/bin/modern-cli-mcp
```

**Install to profile:**
```bash
nix-env -if https://github.com/hellst0rm/modern-cli-mcp/archive/main.tar.gz -A defaultNix.default
```

### Docker

```bash
docker pull ghcr.io/hellst0rm/modern-cli-mcp
docker run --rm -i ghcr.io/hellst0rm/modern-cli-mcp
```

### From Source (Cargo)

Requires CLI tools to be installed separately in PATH:

```bash
cargo install --git https://github.com/hellst0rm/modern-cli-mcp
```

## Development

### With Nix (Recommended)

```bash
# Enter dev shell
nix develop

# Available commands (run 'menu' for full list)
build         # Build release binary
run           # Run the server
test          # Run tests
check         # Cargo check
clippy        # Lint with clippy
fmt           # Format code
flake-check   # Nix flake checks
```

### Without Nix

```bash
# Requires Rust toolchain and CLI tools installed separately
cargo build --release
cargo run
cargo test
```

## Architecture

```
src/
├── main.rs           # Entry point, MCP server setup
└── tools/
    ├── mod.rs        # Tool registration and routing
    └── executor.rs   # Tool execution logic
```

The server wraps CLI tools with structured JSON input/output. Each tool:
1. Validates parameters via JSON Schema
2. Constructs the appropriate command
3. Executes and captures output
4. Returns structured results

## Configuration

The Nix package bundles all CLI tools. When running via cargo, ensure tools are in PATH.

### Environment Variables

- `RUST_LOG` - Logging level (default: `info`)

### .agentignore

Control which files AI agents can access using `.agentignore` files. Uses gitignore syntax but operates independently—tools respect `.agentignore` only, not `.gitignore`.

**Pattern sources (in order of precedence):**
1. `~/.config/agent/ignore` - Global patterns for all projects
2. `.agentignore` - Per-directory patterns (walked up from working directory)

**Example `.agentignore`:**
```gitignore
# Secrets and credentials
*.secret
.env*
secrets/
credentials.json

# Large generated files
node_modules/
target/
*.min.js

# Sensitive data
*.pem
*.key
```

**Behavior:**
- Blocked paths return an error: `Path is blocked by .agentignore: /path/to/file`
- Search tools (fd, rg, ast-grep) automatically apply ignore patterns
- Patterns in child directories extend (not replace) parent patterns

## License

MIT
