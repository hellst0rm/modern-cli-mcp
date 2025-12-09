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

Initial release and CI stabilization.

## Recent Events (Last 10)

1. [2025-12-09] Created project structure with Nix flake, devshell, pog scripts
2. [2025-12-09] Centralized package lists in pkgs.nix
3. [2025-12-09] Added standard project files (.gitignore, .editorconfig, .envrc)
4. [2025-12-09] Added checks.nix with formatCheck, deadnixCheck, statixCheck, rustfmtCheck
5. [2025-12-09] Added githooks.nix for pre-commit hooks
6. [2025-12-09] Created README.md, CLAUDE.md, LICENSE (MIT)
7. [2025-12-09] Created Dockerfile and CI/CD workflows
8. [2025-12-09] Published to GitHub as hellst0rm/modern-cli-mcp
9. [2025-12-09] Fixed CI: dtolnay/rust-action → rust-toolchain, deadnix unused arg, formatting
10. [2025-12-09] Created memory-bank and registered project

## Active Decisions

- [decision] Using flake-utils instead of flake-parts (simpler)
- [decision] MIT license
- [decision] Categories: filesystem, search, text, system, network, diff, test, reference, archive, queue

## Next Steps

- [ ] Verify CI passes on GitHub
- [ ] Add more comprehensive tests
- [ ] Consider adding more tools
- [ ] Create release v0.1.0

## Relations

- tracks [[CI Pipeline]]
- tracks [[GitHub Release]]
