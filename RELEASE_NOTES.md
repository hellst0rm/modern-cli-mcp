# Modern CLI MCP: v0.2.0 Release Notes - Tool Expansion

## Overview

Modern CLI MCP v0.2.0 massively expands the tool collection with Git forge integrations, container/Kubernetes tools, and data transformation utilities optimized for AI/LLM consumption.

## Changes in v0.2.0

### 🚀 New Tool Categories

#### Git Forges (11 tools)
- **gh_repo, gh_issue, gh_pr, gh_search, gh_release, gh_workflow, gh_run, gh_api** - Full GitHub CLI coverage with JSON output
- **glab_issue, glab_mr, glab_pipeline** - GitLab CI/CD operations

#### Data Transformation (5 tools)
- **gron** - Transform JSON to greppable format for deep searching
- **htmlq** - jq for HTML with CSS selectors
- **pup** - HTML parser with display filters and JSON output
- **miller** - Multi-format data processor (CSV, JSON, etc.)
- **dasel** - Universal selector for JSON/YAML/TOML/XML

#### Containers (5 tools)
- **podman** - Full container operations with JSON output
- **dive** - Image layer analysis and efficiency scoring
- **skopeo** - Registry operations without pulling images
- **crane** - Low-level registry tool for manifests/digests
- **trivy** - Security vulnerability scanner with JSON reports

#### Kubernetes (10 tools)
- **kubectl_get, kubectl_describe, kubectl_logs, kubectl_apply, kubectl_delete, kubectl_exec** - Core K8s operations
- **stern** - Multi-pod log aggregation with JSON output
- **helm** - Chart management with JSON for list/status
- **kustomize** - Manifest building

### 🔧 AI/LLM Optimization

- All tools default to JSON output where supported
- Consistent output formats for reliable parsing
- Tool descriptions document output format expectations

### 📦 Total Tools

**70 tools** now available covering:
- Filesystem, search, text processing
- Git forges (GitHub, GitLab)
- Containers and registries
- Kubernetes cluster management
- Data transformation and parsing

## Installation

```bash
# Nix (recommended)
nix run github:hellst0rm/modern-cli-mcp

# Docker
docker pull ghcr.io/hellst0rm/modern-cli-mcp:0.2.0
```

## Configuration

Add to Claude Desktop config:
```json
{
  "mcpServers": {
    "modern-cli": {
      "command": "nix",
      "args": ["run", "github:hellst0rm/modern-cli-mcp"]
    }
  }
}
```

---

# Modern CLI MCP: v0.1.0 Release Notes - Initial Release

## Overview

Modern CLI MCP is an MCP server that exposes modern command-line utilities to AI/LLM agents. It bundles 40+ CLI tools and provides structured JSON-RPC access to them.

## Features

### Core Tools
- **eza, bat, fd, dust, duf** - Modern filesystem utilities
- **ripgrep, fzf, ast-grep** - Powerful search tools
- **jq, yq, sd, hck, qsv** - Data processing
- **git, delta, difftastic** - Version control
- **xh, doggo** - Network utilities
- **procs, tokei, hyperfine** - System tools

### Architecture
- Built with Rust and rmcp for high performance
- Nix-based distribution bundles all dependencies
- Stateless operation - no caching layer

## Installation

```bash
nix run github:hellst0rm/modern-cli-mcp
```
