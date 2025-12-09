---
title: changelog
type: note
permalink: changelog
tags:
- changelog
- history
- versions
---

# Changelog: modern-cli-mcp

## [0.2.0] - 2025-12-09

### Added
- 31 new tools (70+ total):
  - Git Forges: gh_repo, gh_issue, gh_pr, gh_search, gh_release, gh_workflow, gh_run, gh_api, glab_issue, glab_mr, glab_pipeline
  - Containers: podman, dive, skopeo, crane, trivy
  - Kubernetes: kubectl_get, kubectl_describe, kubectl_logs, kubectl_apply, kubectl_delete, kubectl_exec, stern, helm, kustomize
  - Data Transform: gron, htmlq, pup, miller, dasel
- GitHub workflows: ci.yml, publish.yml (binaries, Docker, FlakeHub, SBOM), claude.yml
- .claude/ directory: settings.json, commands/release.md, agents/rust-mcp-expert.md, agents/cli-tools-expert.md
- RELEASE_NOTES.md and RELEASE_WORKFLOW.md
- Website scaffold: Bun/ElysiaJS/HTMX/Hyperscript/UnoCSS

### Changed
- All tools default to JSON output for AI/LLM consumption
- README.md updated with 70+ tools in table format
- CI paths-ignore: docs, memory-bank, .claude/, website, scripts
- Updated .gitignore, .editorconfig, .dockerignore

## [0.1.0] - 2025-12-09

### Added
- MCP server exposing 30+ modern CLI tools
- Nix flake with three package variants (default, full, server-only)
- numtide/devshell development environment
- pog scripts for tooling (tools script)
- Pre-commit hooks (nixfmt, deadnix, statix, rustfmt, clippy)
- Flake checks (formatCheck, deadnixCheck, statixCheck, rustfmtCheck)
- GitHub Actions CI (test, nix, docker jobs)
- GitHub Actions release workflow
- Dockerfile for container builds
- README with installation and usage docs
- CLAUDE.md for AI assistant context
- MIT License

### Fixed
- CI workflow: corrected rust-toolchain action name
- deadnix: removed unused pkgs argument in githooks.nix
- Formatting: applied nixfmt-rfc-style and cargo fmt

## Relations

- documents [[Project History]]