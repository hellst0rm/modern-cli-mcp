---
title: techContext
type: note
permalink: tech-context
tags:
- tech
- stack
- setup
---

# Tech Context: modern-cli-mcp

## Technology Stack

### Core
- [tech] Rust 2021 edition
- [tech] rmcp 0.8 (MCP server framework)
- [tech] tokio (async runtime)
- [tech] serde/serde_json (serialization)
- [tech] schemars (JSON schema generation)

### Build & Package
- [tech] Nix flakes for reproducible builds
- [tech] cargo for Rust builds
- [tech] Docker for containerization

### Development
- [tech] numtide/devshell for dev environment
- [tech] jpetrucciani/pog for CLI scripts
- [tech] cachix/git-hooks.nix for pre-commit hooks
- [tech] rust-analyzer for IDE support

## CLI Tools Bundled

### Filesystem
eza, bat, fd, duf, dust, trash-cli

### Search
ripgrep (rg), fzf, ast-grep (sg)

### Text Processing
sd, jq, yq, qsv, hck

### System
procs, tokei, hyperfine, file

### Network
xh, doggo (dns), usql

### Diff
delta, difftastic

### Reference
tealdeer (tldr), grex, sad, navi

### Other
ouch (archives), pueue (task queue), bats (testing)

## Development Setup

```bash
# Enter dev shell
nix develop

# Build
nix build

# Run
nix run

# Check
nix flake check
```

## Constraints

- [constraint] Must work on Linux x86_64 (primary)
- [constraint] CLI tools must be in PATH at runtime
- [constraint] No interactive TTY support

## Relations

- depends_on [[Nix Ecosystem]]
- depends_on [[Rust Toolchain]]
