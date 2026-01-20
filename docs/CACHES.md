# Global Caches Guide

null-e can detect and clean global developer caches - those shared caches that accumulate over time as you work with different package managers and tools.

## Overview

**Command:** `null-e caches`

Global caches are different from project-specific artifacts:
- **Project artifacts**: `node_modules/`, `target/`, `.venv/` inside projects
- **Global caches**: Shared caches in `~/.cache/`, `~/.local/`, etc.

| Category | Caches | Typical Size |
|----------|--------|--------------|
| **Node.js** | npm, yarn, pnpm, bun | 1-10 GB |
| **Python** | pip, pipx, uv, poetry | 1-5 GB |
| **Rust** | cargo registry, git checkouts | 2-10 GB |
| **Go** | module cache, build cache | 1-5 GB |
| **Java** | Maven, Gradle | 2-10 GB |
| **Ruby** | gem, bundler | 500 MB-2 GB |
| **PHP** | composer | 500 MB-2 GB |

---

## Quick Start

```bash
# Show all global caches with sizes
null-e caches

# Clean selected caches interactively
null-e caches --clean

# Clean all caches at once
null-e caches --clean-all
```

---

## Supported Caches

### Node.js Ecosystem

| Cache | Location | Clean Command |
|-------|----------|---------------|
| **npm cache** | `~/.npm/_cacache` | `npm cache clean --force` |
| **yarn cache** | `~/Library/Caches/Yarn` | `yarn cache clean` |
| **pnpm store** | `~/.pnpm-store` | `pnpm store prune` |
| **bun cache** | `~/.bun/install/cache` | `bun pm cache rm` |

### Python Ecosystem

| Cache | Location | Clean Command |
|-------|----------|---------------|
| **pip cache** | `~/Library/Caches/pip` | `pip cache purge` |
| **pipx cache** | `~/.local/pipx/cache` | - |
| **uv cache** | `~/.cache/uv` | `uv cache clean` |
| **poetry cache** | `~/Library/Caches/pypoetry` | `poetry cache clear --all .` |

### Rust Ecosystem

| Cache | Location | Clean Command |
|-------|----------|---------------|
| **Cargo registry** | `~/.cargo/registry` | - |
| **Cargo git** | `~/.cargo/git` | - |
| **Cargo cache (all)** | `~/.cargo` | `cargo cache -a` (if installed) |

### Go Ecosystem

| Cache | Location | Clean Command |
|-------|----------|---------------|
| **Go modules** | `~/go/pkg/mod` | `go clean -modcache` |
| **Go build cache** | `~/Library/Caches/go-build` | `go clean -cache` |

### Java/JVM Ecosystem

| Cache | Location | Clean Command |
|-------|----------|---------------|
| **Maven** | `~/.m2/repository` | - |
| **Gradle caches** | `~/.gradle/caches` | - |
| **Gradle wrapper** | `~/.gradle/wrapper` | - |

### Ruby Ecosystem

| Cache | Location | Clean Command |
|-------|----------|---------------|
| **Gem cache** | `~/.gem` | `gem cleanup` |
| **Bundler cache** | `~/.bundle/cache` | `bundle cache --all` |

### PHP Ecosystem

| Cache | Location | Clean Command |
|-------|----------|---------------|
| **Composer cache** | `~/.composer/cache` | `composer clear-cache` |

---

## Usage Examples

### Viewing Caches

```bash
# Show all detected caches
null-e caches

# Example output:
🗂️  null-e Caches v0.1.0

✓ Found 12 caches with 8.45 GiB total

       Cache                    Size         Last Used   Clean Command
   ────────────────────────────────────────────────────────────────────
   [1] 📦 npm cache          2.34 GiB      2 days ago   npm cache clean --force
   [2] 🐍 pip cache          1.23 GiB     1 week ago    pip cache purge
   [3] 🦀 Cargo registry     1.89 GiB        3 hours    -
   ...
```

### Interactive Cleaning

```bash
null-e caches --clean

# You'll see:
Enter cache numbers to clean (e.g., 1,3,5 or 1-5 or all):
> 1,3

# Then null-e runs the official clean commands
```

### Clean All

```bash
# With confirmation
null-e caches --clean-all

# Force (no confirmation)
null-e caches --clean-all --force
```

---

## Cache Details

### npm Cache

**Location:** `~/.npm/_cacache`

The npm cache stores downloaded packages to avoid re-downloading. It's safe to clear but will slow down your next install.

```bash
# Official clean command
npm cache clean --force

# Verify cache integrity
npm cache verify
```

**Tip:** npm automatically manages cache size, but it can grow large over time.

### yarn Cache

**Location:**
- Yarn 1: `~/Library/Caches/Yarn` (macOS) or `~/.cache/yarn` (Linux)
- Yarn 2+: `.yarn/cache` (per-project)

```bash
# Clean yarn 1 cache
yarn cache clean

# List cache contents
yarn cache list
```

### pnpm Store

**Location:** `~/.pnpm-store` or `~/.local/share/pnpm/store`

pnpm uses a content-addressable store. Unlike npm/yarn, identical packages are stored only once.

```bash
# Prune unreferenced packages
pnpm store prune

# Check store status
pnpm store status
```

**Note:** pnpm's store is already space-efficient. Only prune if you need space.

### pip Cache

**Location:**
- macOS: `~/Library/Caches/pip`
- Linux: `~/.cache/pip`

```bash
# Clear pip cache
pip cache purge

# Show cache info
pip cache info
```

### uv Cache

**Location:** `~/.cache/uv`

uv (fast Python package manager) uses its own cache.

```bash
# Clean uv cache
uv cache clean
```

### Cargo (Rust)

**Locations:**
- Registry index: `~/.cargo/registry/index`
- Registry cache: `~/.cargo/registry/cache`
- Git checkouts: `~/.cargo/git`

Cargo doesn't have a built-in clean command for the global cache. Consider installing `cargo-cache`:

```bash
# Install cargo-cache
cargo install cargo-cache

# Show cache usage
cargo cache

# Clean all caches
cargo cache -a
```

### Go Caches

**Module cache:** `~/go/pkg/mod`
**Build cache:** `~/Library/Caches/go-build` (macOS)

```bash
# Clean module cache
go clean -modcache

# Clean build cache
go clean -cache

# Clean both
go clean -cache -modcache
```

### Maven Repository

**Location:** `~/.m2/repository`

Maven's local repository stores downloaded dependencies. Be careful - removing it will require re-downloading everything.

```bash
# No official clean command
# Manual cleanup:
rm -rf ~/.m2/repository

# Or selective cleanup (old versions only)
# Consider using maven-dependency-plugin
```

### Gradle Caches

**Location:** `~/.gradle/caches`

```bash
# No official global clean command
# Per-project:
./gradlew clean

# Manual cleanup:
rm -rf ~/.gradle/caches
```

---

## Safety Considerations

### Safe to Clean

These caches can be safely cleaned - they'll be rebuilt on next use:

| Cache | Consequence |
|-------|-------------|
| npm cache | Next install downloads packages again |
| pip cache | Next install downloads packages again |
| Go build cache | Next build takes longer |

### Clean with Caution

These might need consideration:

| Cache | Consideration |
|-------|---------------|
| Cargo registry | Large, slow to re-download |
| Maven repository | Very large, slow to re-download |
| Gradle caches | Includes build outputs |

### Check Before Cleaning

| Cache | What to Check |
|-------|---------------|
| pnpm store | Using `pnpm store status` first |
| Go mod cache | `go mod verify` after cleaning |

---

## Official Commands

null-e uses official package manager commands when available:

| Cache | Official Command |
|-------|-----------------|
| npm | `npm cache clean --force` |
| yarn | `yarn cache clean` |
| pnpm | `pnpm store prune` |
| pip | `pip cache purge` |
| uv | `uv cache clean` |
| poetry | `poetry cache clear --all .` |
| go modules | `go clean -modcache` |
| go build | `go clean -cache` |
| composer | `composer clear-cache` |
| gem | `gem cleanup` |

For caches without official commands (like Cargo or Maven), null-e uses safe manual deletion.

---

## Best Practices

### Regular Maintenance

```bash
# Weekly or monthly cleanup
null-e caches --clean-all
```

### Before Major Work

```bash
# Check what's using space
null-e caches

# Clean if needed
null-e caches --clean
```

### After Upgrading Tools

```bash
# Old cache versions may no longer be needed
null-e caches --clean-all
```

### Low Disk Space Emergency

```bash
# Quick space recovery
null-e caches --clean-all --force
null-e sweep --clean
```

---

## Comparison: Caches vs Sweep

| Command | What It Cleans | Use Case |
|---------|---------------|----------|
| `null-e caches` | Global package manager caches | Regular maintenance |
| `null-e sweep` | System-wide cleanable items | Comprehensive cleanup |
| `null-e scan` | Project artifacts (node_modules, etc.) | Project-specific cleanup |

### Recommended Order

1. `null-e caches` - Clean global caches first
2. `null-e sweep` - Clean system-wide items
3. `null-e stale --clean` - Clean artifacts from old projects
4. `null-e duplicates` - Identify and fix duplicates

---

## Automation

### Shell Alias

```bash
# Add to ~/.bashrc or ~/.zshrc
alias devclean='null-e caches --clean-all && null-e sweep --clean'
```

### Cron Job

```bash
# Monthly cleanup (add to crontab -e)
0 0 1 * * /usr/local/bin/null-e caches --clean-all --force
```

### Git Hook

```bash
# Post-merge hook to clean old caches
#!/bin/sh
null-e caches 2>/dev/null | head -5
```

---

## Troubleshooting

### Cache Not Detected

If a cache isn't showing up:

1. Check if the tool is installed
2. Verify the cache location exists
3. Check if the cache is empty

### Clean Command Failed

If a clean command fails:

1. Check if the tool is in your PATH
2. Try running the command manually
3. Check file permissions

### Disk Space Not Freed

After cleaning:

1. Run `null-e caches` again to verify
2. Empty your system trash
3. Some space may be held by open files (restart apps)
