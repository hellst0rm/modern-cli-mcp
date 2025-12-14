---
title: activeContext
type: note
permalink: active-context
tags:
- active
- current
- focus
---

# Active Context: modern-cli-mcp

## Current Focus

Implementing virtual tool groups and agent profiles for reduced cognitive load on LLMs. Following GitHub Copilot's approach of tool clustering.

## Recent Events (Last 10)
1. [2025-12-13] Added virtual tool groups: 15 groups (filesystem, file_ops, search, text, git, github, gitlab, kubernetes, container, network, system, archive, reference, diff, mcp)
2. [2025-12-13] Added agent profiles: explore, architect, review, test, generator, reflector, curator, docs, lint, api, dev-deploy, full
3. [2025-12-13] Added expand_tools and list_tool_groups meta-tools for tool discovery
4. [2025-12-13] Added CLI flags: --profile, --list-profiles, --list-groups
5. [2025-12-13] Profile-aware server instructions in get_info()
6. [2025-12-12] Added safe filesystem ops: fs_symlink, fs_hardlink with safe_overwrite graveyard backup
7. [2025-12-12] Added pretty names to all 90+ tools (e.g., "GitHub - Repo", "Git - Status")
8. [2025-12-12] Added code intelligence tools: symbols (ast-grep), references (ripgrep)
9. [2025-12-12] Added git primitives: git_status, git_add, git_commit, git_branch, git_checkout, git_log, git_stash
10. [2025-12-12] Replaced trash-cli with rip (rm-improved) - graveyard-based safe delete
## Observations
- [decision] Virtual tool groups reduce cognitive load (GitHub Copilot research: 2-5% improvement)
- [decision] Agent profiles pre-expand relevant tool groups per use case
- [decision] expand_tools meta-tool lets models discover tools within groups
- [decision] All tools remain callable regardless of profile (informational, not restrictive)
- [decision] JSON preferred over JSONL for MCP (single response, nested structure, jq compatible)
- [decision] TUI tools excluded (k9s, lazydocker, jnv) - require interactive terminals
- [decision] Hybrid memory: SQLite for operational state, basic-memory for knowledge
- [decision] Internal state stored at ~/.local/share/modern-cli-mcp/state.db
- [pattern] Pretty tool names: "Category - Action (tool)" format for AI clarity
- [pattern] Profile-aware instructions highlight relevant groups
- [architecture] ToolGroup enum with static tool mappings in src/groups.rs
- [architecture] AgentProfile enum with pre-expanded group sets
## Next Steps
- Benchmark profile impact on agent performance
- Consider embedding-based tool routing (GitHub's approach)
- Add profile autodetection from conversation context
- Document virtual tools in README
## Relations

- tracks [[CI Pipeline]]
- tracks [[GitHub Release]]