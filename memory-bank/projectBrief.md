---
title: projectBrief
type: note
permalink: project-brief
tags:
- foundation
- requirements
- scope
---

# Project Brief: modern-cli-mcp

## Overview

MCP (Model Context Protocol) server that exposes modern CLI tools to AI/LLM agents. Enables Claude and other AI assistants to use powerful command-line utilities like eza, bat, fd, rg, delta, jq, and 30+ other tools.

## Core Requirements

- [requirement] Expose modern CLI tools via MCP protocol
- [requirement] Bundle all CLI tools with the server binary
- [requirement] Support stdio transport for MCP communication
- [requirement] Provide structured JSON schemas for all tool inputs
- [requirement] Handle errors gracefully with informative messages

## Scope

### In Scope
- Modern file operations (eza, bat, fd, dust, duf)
- Text search (ripgrep, ast-grep, fzf)
- Text processing (jq, yq, sd, hck, xsv)
- System monitoring (procs, tokei, hyperfine)
- Network tools (xh, dns/doggo)
- Diff tools (delta, difftastic)
- Archive handling (ouch)
- Task queue (pueue)
- Reference tools (tldr, grex, navi, sad)
- Testing (bats)

### Out of Scope
- Interactive TUI applications
- Tools requiring persistent state
- GUI applications

## Success Criteria

- [criteria] All 30+ tools accessible via MCP
- [criteria] CI passes (cargo test, clippy, nix flake check)
- [criteria] Docker image published to GHCR
- [criteria] Nix flake provides wrapped binary with tools in PATH

## Relations

- implements [[MCP Protocol]]
- uses [[Nix Flakes]]
- uses [[Rust]]
