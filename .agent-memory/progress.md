---
title: progress
type: note
permalink: progress
tags:
- status
- progress
- blockers
---

# Progress: modern-cli-mcp

## What Works
- [done] MCP server with 90+ tool handlers
- [done] Git forge tools: gh_repo, gh_issue, gh_pr, gh_search, gh_release, gh_workflow, gh_run, gh_api, glab_issue, glab_mr, glab_pipeline
- [done] Container tools: podman, dive, skopeo, crane, trivy
- [done] Kubernetes tools: kubectl_get/describe/logs/apply/delete/exec, stern, helm, kustomize
- [done] Data transform tools: gron, htmlq, pup, miller, dasel
- [done] Web search tool using ddgr (DuckDuckGo CLI with native --json)
- [done] All tools output JSON where it makes sense for AI/LLM consumption
- [done] JSON output helpers in executor.rs: parse_eza_to_json, parse_fd_to_json, parse_diff_to_json, parse_trash_list_to_json, parse_file_to_json, parse_fzf_to_json, parse_dust_to_json
- [done] Tools converted to JSON output: eza, fd, dust, trash_list, fzf_filter, delta, file_type, ast_grep
- [done] Nix flake builds successfully
- [done] GitHub workflows: CI, Release (publish.yml), Claude Code (claude.yml), GitHub Pages (pages.yml)
- [done] CI paths-ignore for docs, .agent-memory, .claude/, website
- [done] .claude/ directory with agents and commands
- [done] RELEASE_NOTES.md and RELEASE_WORKFLOW.md
- [done] Static website: index.html, tools.html, docs.html, styles.css (htmx/hyperscript/UnoCSS)
- [done] Updated .gitignore, .editorconfig, .dockerignore
- [done] Internal SQLite state management (src/state.rs) - auth, cache, tasks, context tables
- [done] Shell execution tools: shell_exec (bash/zsh/fish/nu/dash), nix_shell_exec (devshell)
- [done] File operation tools: file_read, file_write, file_edit, file_append
- [done] Filesystem tools: fs_mkdir, fs_copy, fs_move, fs_stat, fs_exists
- [done] MCP state tools: mcp_cache_get/set, mcp_task_create/update/list/delete, mcp_context_get/set/list
- [done] Git forge auth tools: gh_auth_status, gh_auth_login, glab_auth_status, glab_auth_login, mcp_auth_check
- [done] CommandExecutor extended with ExecOptions (timeout, env vars, working_dir)
- [done] file_patch tool using system patch command for unified diffs
- [done] Git primitives: git_status, git_add, git_commit, git_branch, git_checkout, git_log, git_stash
- [done] Code intelligence: symbols (ast-grep), references (ripgrep with word boundaries)
- [done] Pretty names for all 90+ tools (e.g., "GitHub - Repo", "Filesystem - Copy")
- [done] Replaced trash-cli with rip (rm-improved) - graveyard-based safe delete
- [done] Safe overwrite for: fs_copy, fs_move, fs_symlink, fs_hardlink, file_write
- [done] Backup option for: file_edit, file_patch (creates .bak.{timestamp})
- [done] Modernized: sysinfo crate for system_info, native tokio::fs for fs_copy
- [done] Virtual tool groups: 15 groups organizing 104 tools
- [done] Agent profiles: 12 profiles (explore, architect, review, test, generator, reflector, curator, docs, lint, api, dev-deploy, full)
- [done] Meta-tools: expand_tools, list_tool_groups for tool discovery
- [done] CLI flags: --profile, --list-profiles, --list-groups
- [done] Profile-aware server instructions
- [done] Added clap for CLI argument parsing
## Observations
- [done] Added cargo integration tests for JSON output (tests/json_helpers.rs, tests/tool_integration.rs)
- [issue] App lacks 'meta' attribute (warning, non-blocking)
- [architecture] Hybrid memory: SQLite for operational state, basic-memory MCP for knowledge
- [architecture] Safe filesystem: graveyard backup before destructive ops, .bak for edits
- [architecture] Virtual tools: src/groups.rs with ToolGroup and AgentProfile enums
- [research] GitHub Copilot found 2-5% benchmark improvement with reduced toolsets
## Current Status
**Version**: 0.6.2 (released)
**Phase**: Active Development - Feature Complete

### What Works
- 104+ CLI tools exposed via MCP protocol
- Dynamic Toolsets Mode (beta) - on-demand tool activation
- Agent Profiles for role-based tool selection
- .agentignore for file access control
- Busybox-style CLI execution (`modern-cli-mcp eza -la`)
- install.sh script with --user/--system and --full/--binary options
- Dual-response mode (`--dual-response` flag) - COMPLETE
- CI/CD: Cachix caching, AppImage builds, MCP registry publishing

### In Progress
None - feature complete.


## Recent Session
- Released v0.6.2: CI/CD Infrastructure Fixes
- Migrated from FlakeHub to standard GitHub URLs
- Replaced Determinate Nix with Cachix (nacosolutions cache)
- Removed Docker builds from CI (simplified workflow)
- Fixed AppImage build issues:
  - Renamed wrapped package to match binary name
  - Use cp -L to handle Nix store symlinks
- Fixed mcp-publisher download URL for MCP registry
- Fixed clippy warnings (collapsible_if, dead_code)
- CI workflow: build (7m), test (3m)
- Publish workflow: build + appimage + mcp-registry + release-assets

## Previous Session
- Released v0.6.0: Dual-response mode complete
- All 104+ tools refactored to use build_response() pattern
- Tested dual-response mode - summary + embedded resource working
- Deployed to GitHub Releases

## Previous Session
- Released v0.5.0: Busybox-style CLI execution
- Released v0.4.0: Dynamic Toolsets & Batch Operations
- Deployed to FlakeHub, GitHub Releases, Docker

## Relations

- blocked_by [[CI Pipeline]] (awaiting verification)

- [done] Dynamic Toolsets Mode (beta): `--dynamic-toolsets` flag, `--toolsets` pre-enable, env vars support
- [done] Meta-tools for toolset management: `list_available_toolsets`, `get_toolset_tools`, `enable_toolset`
- [done] Custom `ServerHandler::list_tools` for dynamic tool filtering
- [done] Thread-safe toolset state with `parking_lot::RwLock<HashSet<ToolGroup>>`
- [done] Batch operations for trash, copy, move tools (space-separated paths)
- [done] Improved git tools path descriptions (clarifies `-C <path>` behavior)
- [done] Pretty names for all 104+ tools (consistent "Category - Name (tool)" format)
- [done] Extended batch support: mkdir, stat, exists, file_edit
- [done] Multi-file edit capability for applying same replacement across files
