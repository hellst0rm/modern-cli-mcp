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

## [Unreleased]

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
