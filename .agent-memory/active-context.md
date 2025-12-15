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

Added new container tools: podman-compose, buildx, buildah. Container group now has 8 tools for comprehensive container workflow support.

## Recent Events (Last 10)
1. [2025-12-15] Added container tools: compose (podman-compose), buildx (multi-platform builds), buildah (OCI image builder)
2. [2025-12-15] Updated pkgs.nix with podman-compose, docker-buildx, buildah dependencies
3. [2025-12-15] Updated website tools.html with new container tools (8 tools total)
4. [2025-12-14] Migrated memory-bank/ → .agent-memory/ with kebab-case filenames
5. [2025-12-14] Integrated Determinate Nix: release.yml now uses determinate-nix-action@v3 + magic-nix-cache-action@v8
6. [2025-12-14] Updated flake.nix to use FlakeHub URLs for nixpkgs and flake-utils
7. [2025-12-14] Fixed clippy warnings causing CI failure (dead code, format strings, useless conversion)
8. [2025-12-14] Implemented .agentignore support for path filtering (src/ignore.rs)
9. [2025-12-13] Added virtual tool groups: 15 groups (filesystem, file_ops, search, text, git, github, gitlab, kubernetes, container, network, system, archive, reference, diff, mcp)
10. [2025-12-13] Added agent profiles and expand_tools/list_tool_groups meta-tools

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
- Create v0.3.1 release with new container tools
- Consider adding more container tools: docker-compose (native), podman-tui, lazydocker

## Relations

- tracks [[CI Pipeline]]
- tracks [[GitHub Release]]
