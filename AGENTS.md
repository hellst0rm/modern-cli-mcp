# AGENTS.md

Project context for AI coding assistants (Claude Code, Cursor, Copilot, Aider, etc.).

## MCP Configuration

This project IS an MCP server. For development, configure complementary MCP servers in your tool's config.

### Global Config (recommended for all projects)

Add to your AI tool's global config (e.g., `~/.claude.json`):

```json
{
  "mcpServers": {
    "basic-memory": { "command": "uvx", "args": ["basic-memory", "mcp"] },
    "sequentialthinking": { "command": "npx", "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"] },
    "modern-cli": { "command": "nix", "args": ["run", "github:hellst0rm/modern-cli-mcp", "--"] }
  }
}
```

This MCP server provides GitHub/GitLab tools (`gh_*`, `glab_*`), replacing the need for separate git forge MCPs.

### Per-Repo Config (optional, gitignored)

Create `.mcp.json` in repo root for project-specific MCPs:

```json
{
  "mcpServers": {
    "nixos": { "command": "uvx", "args": ["mcp-nixos"] }
  }
}
```

## Project Overview

Modern CLI MCP is a Model Context Protocol server written in Rust that exposes modern command-line utilities to AI/LLM agents. It bundles 70+ CLI tools (eza, bat, fd, rg, jq, etc.) and provides structured JSON-RPC access to them.

## Key Architecture

```
src/
├── main.rs           # MCP server setup using rmcp crate
└── tools/
    ├── mod.rs        # Tool definitions, schemas, routing
    └── executor.rs   # Command execution, output capture
```

### Core Components

- **rmcp**: Rust MCP SDK for server implementation
- **Tool Registration**: Each tool is defined with JSON Schema for parameters
- **Executor**: Constructs shell commands, captures stdout/stderr
- **Nix Wrapper**: Bundles all CLI tools in PATH via `makeWrapper`

### Data Flow

1. Client sends JSON-RPC tool call
2. Server validates params against schema
3. Executor builds command with arguments
4. Tool runs, output captured
5. Structured response returned

## Development Commands

### With Nix (Recommended)

```bash
nix develop          # Enter dev shell
menu                 # Show all commands

# Core commands
build                # cargo build --release
run                  # cargo run
test                 # cargo test
check                # cargo check
clippy               # cargo clippy
fmt                  # cargo fmt
fmt-nix              # nixfmt .

# Nix commands
nix-build            # nix build
flake-check          # nix flake check
flake-show           # nix flake show
update               # nix flake update

# Utility
tools                # List bundled CLI tools and versions
```

### Without Nix

```bash
cargo build --release
cargo run
cargo test
cargo clippy -- -W clippy::pedantic
cargo fmt
```

## File Layout

| File | Purpose |
|------|---------|
| `flake.nix` | Nix flake: packages, devShell, apps |
| `shell.nix` | Devshell configuration (numtide/devshell) |
| `pkgs.nix` | CLI tool lists (cliTools, devTools) |
| `checks.nix` | Nix checks: format, deadnix, statix, rustfmt |
| `githooks.nix` | Pre-commit hooks configuration |
| `scripts/` | Pog-based helper scripts |
| `Cargo.toml` | Rust dependencies |

## Adding New Tools

1. Add tool to `cliTools` in `pkgs.nix`
2. Define tool schema in `src/tools/mod.rs`
3. Implement execution in `src/tools/executor.rs`
4. Update `scripts/tools.nix` tool list

## Testing

```bash
# Unit tests
cargo test

# Integration test (requires tools in PATH)
nix develop -c cargo test

# Manual test
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | cargo run
```

## CI/CD

- **CI**: Runs on PRs - cargo test, clippy, rustfmt, nix flake check
- **Release**: On version tags - builds binaries, Docker image, GitHub release

## Distribution Methods

1. **Nix**: `nix run github:hellst0rm/modern-cli-mcp`
2. **Docker**: `ghcr.io/hellst0rm/modern-cli-mcp`
3. **Binary**: GitHub releases (Linux x86_64)
4. **Cargo**: `cargo install --git ...`

## Environment Variables

- `RUST_LOG`: Log level (trace/debug/info/warn/error)

## Important Notes

1. **Tool Bundling**: Nix wraps binary with all tools in PATH. Cargo builds require manual tool installation.

2. **Cross-Platform**: Primary target is Linux x86_64. macOS support via Nix.

3. **No Caching**: Tools execute directly, no result caching.

4. **JSON Output**: Most tools return structured JSON for AI/LLM consumption.
