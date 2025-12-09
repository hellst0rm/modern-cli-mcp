---
title: systemPatterns
type: note
permalink: system-patterns
tags:
- architecture
- patterns
- decisions
---

# System Patterns: modern-cli-mcp

## Architecture

```
┌─────────────────────────────────────────┐
│           MCP Client (Claude)           │
└────────────────────┬────────────────────┘
                     │ stdio
┌────────────────────▼────────────────────┐
│         modern-cli-mcp server           │
│  ┌─────────────────────────────────┐    │
│  │    rmcp framework (Rust)        │    │
│  │  - Tool registration            │    │
│  │  - JSON schema generation       │    │
│  │  - Request/response handling    │    │
│  └─────────────────────────────────┘    │
│  ┌─────────────────────────────────┐    │
│  │    CommandExecutor              │    │
│  │  - Spawns CLI processes         │    │
│  │  - Captures stdout/stderr       │    │
│  │  - Handles timeouts             │    │
│  └─────────────────────────────────┘    │
└────────────────────┬────────────────────┘
                     │ subprocess
┌────────────────────▼────────────────────┐
│         CLI Tools (in PATH)             │
│  eza, bat, fd, rg, jq, yq, gh, kubectl  │
└─────────────────────────────────────────┘
```

## Observations

- [decision] Use rmcp crate for MCP protocol implementation
- [decision] Wrap binary with Nix makeWrapper to inject tool PATH
- [decision] Async executor with tokio for concurrent tool calls
- [decision] Schemars for automatic JSON schema generation
- [decision] Tool categories: filesystem, search, git-forges, containers, kubernetes, data-transform, network, system, diff, utilities
- [decision] JSON output preferred for AI/LLM consumption (vs JSONL)
- [decision] TUI tools excluded - require interactive terminals
- [decision] Website: Bun/ElysiaJS/HTMX/Hyperscript/UnoCSS (hypermedia-driven)
- [pattern] Tool Handler: #[tool(description)] async fn with Parameters<T>
- [pattern] Executor: CommandExecutor wraps tokio::process::Command
- [pattern] Nix: buildRustPackage + symlinkJoin + wrapProgram
- [pattern] Website: ElysiaJS → Server-rendered HTML → HTMX/Hyperscript
- [pattern] CI/CD: paths-ignore for non-code files, multi-job workflows

## Relations

- uses [[rmcp Crate]]
- uses [[Nix makeWrapper]]
- implements [[MCP Tool Protocol]]
- uses [[ElysiaJS]]
- uses [[HTMX]]
- publishes_to [[FlakeHub]]
- publishes_to [[GitHub Container Registry]]