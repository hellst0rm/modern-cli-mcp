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

- [done] MCP server with 70+ tool handlers
- [done] Git forge tools: gh_repo, gh_issue, gh_pr, gh_search, gh_release, gh_workflow, gh_run, gh_api, glab_issue, glab_mr, glab_pipeline
- [done] Container tools: podman, dive, skopeo, crane, trivy
- [done] Kubernetes tools: kubectl_get/describe/logs/apply/delete/exec, stern, helm, kustomize
- [done] Data transform tools: gron, htmlq, pup, miller, dasel
- [done] All tools default to JSON output for AI/LLM consumption
- [done] Nix flake builds successfully
- [done] GitHub workflows: CI, Release (publish.yml), Claude Code (claude.yml)
- [done] CI paths-ignore for docs, memory-bank, .claude/, website
- [done] .claude/ directory with agents and commands
- [done] RELEASE_NOTES.md and RELEASE_WORKFLOW.md
- [done] Website scaffold: Bun/ElysiaJS/HTMX/Hyperscript/UnoCSS
- [done] Updated .gitignore, .editorconfig, .dockerignore

## Observations

- [todo] Test website locally with `bun run dev`
- [todo] Create website deployment workflow
- [todo] Tag and release v0.2.0
- [todo] Verify Docker image builds correctly
- [todo] Add cargo integration tests
- [issue] App lacks 'meta' attribute (warning, non-blocking)
- [issue] Website not yet tested

## Current Status

**Phase**: v0.2.0 Release Preparation
**Health**: Green

## Relations

- blocked_by [[CI Pipeline]] (awaiting verification)