---
title: active-context
type: note
permalink: active-context
tags:
- active
- current
- focus
---

# Active Context: modern-cli-mcp

## Current Focus

Integrated Determinate Nix and FlakeHub for improved CI/CD and flake management. Migrated project memory to standard `.agent-memory/` structure.

## Recent Events (Last 10)
1. [2025-12-14] Migrated memory-bank/ → .agent-memory/ with kebab-case filenames
2. [2025-12-14] Integrated Determinate Nix: release.yml now uses determinate-nix-action@v3 + magic-nix-cache-action@v8
3. [2025-12-14] Updated flake.nix to use FlakeHub URLs for nixpkgs and flake-utils
4. [2025-12-14] Added fh (FlakeHub CLI) to devshell with commands: fh-search, fh-list, fh-resolve, fh-init, fh-add
5. [2025-12-14] Fixed clippy warnings causing CI failure (dead code, format strings, useless conversion)
6. [2025-12-14] Implemented .agentignore support for path filtering (src/ignore.rs)
7. [2025-12-13] Added virtual tool groups: 15 groups (filesystem, file_ops, search, text, git, github, gitlab, kubernetes, container, network, system, archive, reference, diff, mcp)
8. [2025-12-13] Added agent profiles: explore, architect, review, test, generator, reflector, curator, docs, lint, api, dev-deploy, full
9. [2025-12-13] Added expand_tools and list_tool_groups meta-tools for tool discovery
10. [2025-12-13] Added CLI flags: --profile, --list-profiles, --list-groups

## Observations
- [decision] Determinate Nix provides faster, more reliable CI builds
- [decision] FlakeHub URLs enable semantic versioning with wildcards (0.1.*)
- [decision] fh CLI in devshell for FlakeHub operations without host changes
- [decision] .agentignore respected instead of .gitignore (different use cases)
- [decision] Memory bank uses kebab-case and .agent-memory/ to align with global conventions
- [pattern] CI workflows use magic-nix-cache-action for GitHub Actions caching
- [pattern] FlakeHub publish on release for public flake discovery
- [architecture] Global memory: ~/.agent-memory/, Project memory: ./.agent-memory/

## Next Steps
- Verify CI passes with Determinate Nix integration
- Test FlakeHub publish workflow on next release
- Document .agentignore usage in README

## Relations

- tracks [[CI Pipeline]]
- tracks [[GitHub Release]]
