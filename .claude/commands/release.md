---
allowed-tools: Bash, Read, Edit, Glob, Grep, Write, TodoWrite
description: Perform a version release with automated binary builds and Docker image publishing
---

# Release

Automate the release process: version bump, changelog, tag creation, and trigger CI/CD for binaries and Docker deployments.

## Workflow

1. Review commits since last release
2. Determine version bump (patch/minor/major)
3. Update `Cargo.toml` and `RELEASE_NOTES.md`
4. Commit, tag, and create GitHub release
5. Verify binary and Docker deployments

## Key Files

- `Cargo.toml` - Package version
- `RELEASE_NOTES.md` - Release changelog
- `.github/workflows/publish.yml` - Binary & Docker publishing

## Execute

### 1. Review Changes

```bash
# Get current version and recent tags
grep '^version = ' Cargo.toml
git tag --list 'v*' --sort=-version:refname | head -5

# Review commits since last release
git log $(git tag --sort=-version:refname | head -1)..HEAD --oneline
```

### 2. Update Version

Version bump types:
- **Patch** (x.y.Z): Bug fixes, CI/CD, docs
- **Minor** (x.Y.0): New tools, backward-compatible features
- **Major** (X.0.0): Breaking changes

Edit `Cargo.toml`:
```toml
version = "X.Y.Z"
```

### 3. Update Release Notes

Add new section at top of `RELEASE_NOTES.md`:

```markdown
# Modern CLI MCP: vX.Y.Z Release Notes - [Title]

## Overview
Brief description (1-2 sentences).

## Changes in vX.Y.Z
### 🚀 New Tools
- **tool_name**: Description

### 🔧 Improvements
- **Feature**: Description

### 📦 Dependencies
- Changes or "No changes from previous version"

## Installation
[Standard installation commands]

---
```

### 4. Commit and Tag

```bash
git add Cargo.toml Cargo.lock RELEASE_NOTES.md
git commit -m "chore: release vX.Y.Z"
git push

git tag -a vX.Y.Z -m "Release vX.Y.Z: [description]"
git push origin vX.Y.Z
```

### 5. Create GitHub Release

```bash
gh release create vX.Y.Z \
  --title "vX.Y.Z: [Title]" \
  --notes "## Overview

[Brief description]

## Highlights
- 🚀 [New tools]
- 🔧 [Improvements]

## Installation
\`\`\`bash
nix run github:hellst0rm/modern-cli-mcp
\`\`\`

See [RELEASE_NOTES.md](RELEASE_NOTES.md) for details."
```

### 6. Monitor Pipeline

```bash
gh run list --workflow=publish.yml --limit 3
gh run watch <RUN_ID>
```

## Verify

### Nix
```bash
nix run github:hellst0rm/modern-cli-mcp@vX.Y.Z
```

### Docker
```bash
docker pull ghcr.io/hellst0rm/modern-cli-mcp:X.Y.Z
```

## Report

```
✅ Release vX.Y.Z Complete!

**Version:** vX.Y.Z
**Release URL:** https://github.com/hellst0rm/modern-cli-mcp/releases/tag/vX.Y.Z

### Verified Deployments
- ✅ Nix Flake
- ✅ GitHub Release Binary
- ✅ GHCR: ghcr.io/hellst0rm/modern-cli-mcp:X.Y.Z
```
