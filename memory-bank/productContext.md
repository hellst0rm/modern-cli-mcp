---
title: productContext
type: note
permalink: product-context
tags:
- context
- goals
- ux
---

# Product Context: modern-cli-mcp

## Why This Project Exists

AI assistants like Claude Code need access to modern CLI tools for effective software development assistance. Traditional Unix tools (find, grep, cat) are powerful but verbose. Modern alternatives (fd, rg, bat) provide better defaults, structured output, and superior UX.

## Problems Solved

- [problem] AI agents lack access to modern CLI tools
- [problem] Traditional tools require complex flag combinations
- [problem] No consistent interface for AI to use shell utilities
- [problem] Tool availability varies across systems

## User Experience Goals

- [ux] Zero-config deployment via Nix or Docker
- [ux] All tools available without manual PATH setup
- [ux] Consistent JSON schemas for tool inputs
- [ux] Helpful error messages when tools fail
- [ux] Structured output for easy parsing

## Target Users

- AI/LLM agents (primary)
- Developers using Claude Code
- MCP-compatible AI assistants

## Relations

- solves [[AI Tool Access Problem]]
- improves [[Developer Experience]]
