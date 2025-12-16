---
title: active-context
type: note
permalink: active-context
tags:
- active
- current
- focus
---

- [2025-12-17] Completed dual-response refactoring for all 104+ tools
- [2025-12-16] Added dual-response mode (--dual-response flag) - returns formatted summary + raw data
- [2025-12-16] Released v0.5.0 with busybox-style CLI execution and install.sh script
- [2025-12-16] Created src/format.rs with summary formatters for tool output

# Active Context: modern-cli-mcp

## Current Focus
Completed dual-response mode for all 104+ tools. Every tool now uses `build_response()` pattern:
- Normal mode: Returns raw JSON/text data
- Dual-response mode (`--dual-response`): Returns summary text + embedded resource with raw data

## Recent Events (Last 10)
1. [2025-12-16] Released v0.4.0 - Dynamic Toolsets, pretty names, batch operations
2. [2025-12-16] Added pretty names to all tools (14 tools updated)
3. [2025-12-16] Extended batch support to mkdir, stat, exists, file_edit tools
4. [2025-12-16] Multi-file edit: file_edit now accepts space-separated paths
5. [2025-12-16] Added batch support to trash, copy, move tools
6. [2025-12-16] Implemented Dynamic Toolsets Mode (beta) - `--dynamic-toolsets` flag
7. [2025-12-16] Added `--toolsets` CLI flag and `MCP_TOOLSETS` env var
8. [2025-12-16] Updated flake.nix to use `self` instead of `./` for src paths
9. [2025-12-15] Updated Container - Compose tool for podman-compose and docker compose v2
10. [2025-12-15] Added container tools: compose, buildx, buildah

## Observations
- [decision] Determinate Nix provides faster, more reliable CI builds
- [decision] FlakeHub URLs enable semantic versioning with wildcards (0.1.*)
- [decision] fh CLI in devshell for FlakeHub operations without host changes
- [decision] .agentignore respected instead of .gitignore (different use cases)
- [decision] Memory bank uses kebab-case and .agent-memory/ to align with global conventions
- [pattern] CI workflows use magic-nix-cache-action for GitHub Actions caching
- [pattern] FlakeHub publish on release for public flake discovery
- [architecture] Global memory: ~/.agent-memory/, Project memory: ./.agent-memory/
- [pattern] Batch operations: parse space-separated paths, iterate with per-item error handling, return JSON array results
- [pattern] Git tools use `run_in_dir()` with `-C <path>` instead of requiring `cd && git`
- [important] Wrapped binary required for CLI tools - `nix build .#default` includes all tools in PATH via makeWrapper

## Next Steps
- Add `tools/list_changed` notification when toolsets are enabled (requires rmcp peer access)
- Consider adding disable_toolset for symmetry
- Fix mcp-registry CI job (mcp-publisher download failing)
- Document that MCP config must use wrapped binary (`nix build .#default`) for CLI tools in PATH

## Relations

- tracks [[CI Pipeline]]
- tracks [[GitHub Release]]

- [architecture] Dynamic Toolsets: DynamicToolsetConfig with RwLock<HashSet<ToolGroup>> for thread-safe runtime group management
- [pattern] Tool filtering: Manual ServerHandler::list_tools implementation checks enabled_groups against tool_to_group reverse lookup
- [decision] Meta-tools (expand_tools, list_tool_groups, list_available_toolsets, etc.) always visible in dynamic mode