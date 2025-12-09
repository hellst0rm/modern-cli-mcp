# Modern CLI MCP Server

[![CI](https://github.com/hellst0rm/modern-cli-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/hellst0rm/modern-cli-mcp/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/hellst0rm/modern-cli-mcp)](https://github.com/hellst0rm/modern-cli-mcp/releases)
[![Docker](https://img.shields.io/badge/Docker-ghcr.io-blue)](https://ghcr.io/hellst0rm/modern-cli-mcp)
[![FlakeHub](https://img.shields.io/endpoint?url=https://flakehub.com/f/hellst0rm/modern-cli-mcp/badge)](https://flakehub.com/flake/hellst0rm/modern-cli-mcp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**[Documentation](https://hellst0rm.github.io/modern-cli-mcp/)** · **[Tools](https://hellst0rm.github.io/modern-cli-mcp/tools.html)**

MCP server exposing **70+ modern CLI tools** to AI/LLM agents. Provides structured JSON-RPC access to filesystem, Git forges, containers, Kubernetes, and data transformation tools—all optimized for AI consumption with JSON output.

## Features

- **70+ Tools**: Filesystem, search, Git (GitHub/GitLab), containers, Kubernetes, data processing
- **AI-Optimized Output**: JSON by default for structured parsing
- **Zero Config**: Nix bundles all dependencies—no manual tool installation
- **Stateless**: No caching, no state files, just tools

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

### From Source (Cargo)

```bash
cargo install --git https://github.com/hellst0rm/modern-cli-mcp
```

### From Source (Nix)

```bash
# Run directly
nix run github:hellst0rm/modern-cli-mcp

# Install to profile
nix profile install github:hellst0rm/modern-cli-mcp

# Development shell
nix develop github:hellst0rm/modern-cli-mcp
```

### Docker

```bash
docker pull ghcr.io/hellst0rm/modern-cli-mcp
docker run --rm -i ghcr.io/hellst0rm/modern-cli-mcp
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

Environment variables:
- `RUST_LOG` - Logging level (default: `info`)

## License

MIT
