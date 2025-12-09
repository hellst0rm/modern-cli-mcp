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
│  eza, bat, fd, rg, jq, yq, ...          │
└─────────────────────────────────────────┘
```

## Key Design Decisions

- [decision] Use rmcp crate for MCP protocol implementation
- [decision] Wrap binary with Nix makeWrapper to inject tool PATH
- [decision] Async executor with tokio for concurrent tool calls
- [decision] Schemars for automatic JSON schema generation
- [decision] Tool categories for organization (filesystem, search, text, etc.)

## Patterns

### Tool Handler Pattern
```rust
#[tool(description = "Tool description")]
async fn tool_name(&self, Parameters(req): Parameters<RequestType>) -> Result<CallToolResult, ErrorData>
```

### Executor Pattern
- CommandExecutor wraps tokio::process::Command
- Returns CommandOutput with stdout, stderr, exit_code
- Handles shell escaping via shellwords

### Nix Packaging Pattern
- buildRustPackage for the server binary
- symlinkJoin + wrapProgram for bundled distribution
- Separate packages: default (wrapped), full (with tools), server-only

## Relations

- uses [[rmcp Crate]]
- uses [[Nix makeWrapper]]
- implements [[MCP Tool Protocol]]
