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

Major v0.2.0 expansion with 70+ tools, project infrastructure, and website.

## Recent Events (Last 10)

1. [2025-12-09] Added 31 new tools: Git forges (gh, glab), containers (podman, dive, skopeo, crane, trivy), Kubernetes (kubectl, helm, stern, kustomize), data transform (gron, htmlq, pup, miller, dasel)
2. [2025-12-09] Created GitHub workflows: ci.yml (with paths-ignore), publish.yml (binaries, Docker, FlakeHub, SBOM), claude.yml
3. [2025-12-09] Created .claude/ directory: settings.json, commands/release.md, agents/rust-mcp-expert.md, agents/cli-tools-expert.md
4. [2025-12-09] Created RELEASE_NOTES.md (v0.1.0, v0.2.0) and RELEASE_WORKFLOW.md
5. [2025-12-09] Updated README.md with 70+ tools documentation in table format
6. [2025-12-09] Created Dockerfile with multi-stage Nix build
7. [2025-12-09] Created website scaffold: Bun/ElysiaJS/HTMX/Hyperscript/UnoCSS
8. [2025-12-09] Website files: package.json, src/index.ts, uno.config.ts, public/styles.css
9. [2025-12-09] Updated .gitignore, .editorconfig, .dockerignore for new project structure
10. [2025-12-09] All tools default to JSON output for AI/LLM consumption

## Observations

- [decision] JSON preferred over JSONL for MCP (single response, nested structure, jq compatible)
- [decision] TUI tools excluded (k9s, lazydocker, jnv) - require interactive terminals
- [decision] Website stack: Bun/ElysiaJS/HTMX/Hyperscript/UnoCSS (not Next.js)
- [decision] CI paths-ignore: docs, memory-bank, .claude/, website, scripts
- [decision] FlakeHub publishing for Nix distribution

## Next Steps

- [ ] Test website with `bun run dev`
- [ ] Create website deployment workflow
- [ ] Tag and release v0.2.0
- [ ] Verify Docker image builds correctly

## Relations

- tracks [[CI Pipeline]]
- tracks [[GitHub Release]]