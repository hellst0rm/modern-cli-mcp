# MCP Registry Publication

This document describes the MCP registry publication setup for this project.

## Overview

This MCP server is automatically published to the [GitHub MCP Registry](https://github.com/mcp) on every release, making it discoverable and installable through MCP clients.

## server.json

The `server.json` file contains metadata about this MCP server for registry publication. It includes:

- **Name**: `io.github.nacosolutions/modern-cli-mcp`
- **Version**: Synchronized with `Cargo.toml` version
- **Installation methods**: Nix flakes, Docker, and binary downloads
- **Transport**: stdio-based communication
- **Features**: 107 CLI tools across 15 categories

## Publication Workflow

The publication is automated via GitHub Actions in `.github/workflows/publish.yml`:

1. **Trigger**: Runs automatically when a new GitHub release is published
2. **Authentication**: Uses GitHub OIDC (automatic in GitHub Actions)
3. **Publisher tool**: Downloads the official `mcp-publisher` CLI
4. **Publication**: Submits `server.json` to the MCP registry

## Manual Publication

To manually publish to the registry:

```bash
# Install mcp-publisher
curl -L "https://github.com/modelcontextprotocol/registry/releases/download/latest/mcp-publisher_$(uname -s | tr '[:upper:]' '[:lower:]')_$(uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/').tar.gz" | tar xz

# Login (if not in GitHub Actions)
./mcp-publisher login

# Publish
./mcp-publisher publish --server-json server.json
```

## Namespace Ownership

The namespace `io.github.nacosolutions/modern-cli-mcp` is automatically validated through:

- **GitHub OIDC**: When publishing from GitHub Actions on this repository
- **GitHub OAuth**: When publishing manually with GitHub login

This ensures only authorized maintainers can publish updates.

## Registry Discovery

Once published, the server becomes discoverable through:

- [GitHub MCP Registry](https://github.com/mcp)
- MCP clients with registry integration
- Community MCP registries

Users can install it directly from the registry using their MCP client's installation interface.

## Updating server.json

When updating `server.json`, ensure:

1. Version matches `Cargo.toml`
2. Docker image tag is updated
3. All installation methods are tested
4. Description accurately reflects features

The registry validates the submission and may reject invalid configurations.

## References

- [MCP Registry Documentation](https://github.com/modelcontextprotocol/registry)
- [server.json Specification](https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/server-json/generic-server-json.md)
- [Publishing Guide](https://modelcontextprotocol.info/tools/registry/publishing/)
