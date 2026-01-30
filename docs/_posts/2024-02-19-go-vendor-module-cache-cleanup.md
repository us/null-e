---
layout: post
title: "Go vendor/ Directory and Module Cache Cleanup: Reclaim 5-20GB from Go Projects"
description: "Go developers lose disk space to vendor/ directories and module cache. Learn how to safely clean Go build artifacts, go mod cache, and old projects. Complete guide for Go developers."
date: 2024-02-19
author: us
tags: [golang, go-modules, vendor-directory, disk-cleanup, go-mod-cache, build-artifacts]
---

[![null-e - Disk Cleanup Tool for Developers](https://img.shields.io/crates/v/null-e.svg)](https://crates.io/crates/null-e)

**[View on GitHub →](https://github.com/us/null-e)**

If you're a Go developer, you might think you're safe from the dependency hell that plagues Node.js or Python. And you'd be... partially right.

Go modules are efficient. Go binaries are static. But Go still has disk space problems.

> *"Go is amazing, but even Go projects accumulate cruft. Vendor directories, module cache, test binaries—they all add up."* — **Go developer community**

The `vendor/` directory. The `~/go/pkg/mod/` cache. Test binaries. Build artifacts.

A typical Go developer with 20, 30 projects easily has **5-20GB** of Go-related disk usage.

Let's fix that.

---

## The Go Disk Space Problem

Go is better than most languages, but not perfect:

| Component | Location | Typical Size | When It Grows |
|-----------|----------|--------------|---------------|
| **Module cache** | `~/go/pkg/mod/` | 1-5GB | Every `go mod download` |
| **Vendor directory** | `./vendor/` | 10-100MB per project | When vendoring |
| **Build cache** | `~/go/cache/` | 100MB-1GB | Every build |
| **Test binaries** | `*.test` | 1-10MB each | Running tests |
| **Old Go versions** | `~/go/` (GOPATH) | 100MB-500MB | Multiple installs |

Not as bad as Rust's `target/` or Node's `node_modules`, but it accumulates.

**<!-- TODO: INSERT IMAGE - Visual showing Go module cache and vendor directories -->

---

## Why Go Projects Still Use Disk Space

### The Module Cache

Go downloads modules to a shared cache:

```bash
# Where Go stores all modules
ls ~/go/pkg/mod/

# Contents:
cache/
download/
github.com/
golang.org/
google.golang.org/
... (hundreds more)
```

Every module you've ever used. Every version. Forever.

```bash
# Check size
du -sh ~/go/pkg/mod/

# Output: 3.2G    /Users/you/go/pkg/mod
```

3.2GB. For a "lightweight" language.

### The Vendor Directory

Some projects vendor dependencies:

```
my-project/
├── go.mod
├── go.sum
└── vendor/
    ├── github.com/
    │   ├── gin-gonic/
    │   ├── stretchr/
    │   └── ...
    └── golang.org/
        └── ...
```

Vendoring means: **full copy of all dependencies in project**.

| Project Type | Vendor Size | Notes |
|-------------|-------------|-------|
| Simple CLI | 5-20MB | Few deps |
| Web server | 20-50MB | HTTP framework, router |
| Complex service | 50-200MB | Many integrations |
| Enterprise app | 200MB-1GB | Everything vendored |

20 projects with vendor directories? **400MB-4GB**.

### Build Cache

Go keeps a build cache for faster rebuilds:

```bash
# Build cache location
~/go/cache/
or
~/Library/Caches/go-build/  # macOS
```

```bash
# Check size
du -sh ~/Library/Caches/go-build/

# Output: 450M    /Users/you/Library/Caches/go-build
```

450MB. Of build cache. Multiplied by CI machines, multiple computers...

### Test Binaries

Running tests creates test binaries:

```bash
# After go test
ls -la *.test

# Output:
-rwxr-xr-x  1 you  you  12M  main.test
-rwxr-xr-x  1 you  you  8M   utils.test
```

12MB + 8MB = 20MB of test binaries. Per project. Often not cleaned.

**<!-- TODO: INSERT IMAGE - File tree showing Go project structure with vendor/ -->

---

## The GOPATH Legacy

Old Go projects used GOPATH:

```bash
# The old way
export GOPATH=~/go

# Structure:
~/go/
├── src/
│   └── github.com/
│       └── you/
│           └── project/  # All projects here
├── pkg/
│   └── ...  # Compiled packages
└── bin/
    └── ...  # Installed binaries
```

Modern Go uses modules, but some still have:

- Old `~/go/src/` directories
- Legacy GOPATH projects
- Multiple Go versions installed

All taking space.

**<!-- TODO: INSERT IMAGE - Comparison: GOPATH vs Go Modules structure -->

---

## The Manual Cleanup Trap

Go cleanup is scattered:

### Clean Module Cache

```bash
# Go's built-in command
go clean -modcache
```

What it does:
- ✅ Deletes `~/go/pkg/mod/`
- ⚠️ **Deletes ALL modules** (aggressive)
- ❌ No size information
- ❌ No "what will break" check

> *"I ran go clean -modcache and now every build re-downloads everything."* — **Common Go experience**

### Clean Build Cache

```bash
go clean -cache
```

Better, but:
- ❌ No size info
- ❌ No selective cleaning
- ❌ All or nothing

### Clean Vendor

```bash
# Delete vendor directory
rm -rf vendor/

# Then re-download (if needed)
go mod vendor
```

Manual. Risky if you actually need vendoring.

### Find All Go Projects

```bash
# Find go.mod files
find ~ -name "go.mod" -type f

# Check sizes
find ~ -name "vendor" -type d -exec du -sh {} \;
```

Slow. No context. No safety.

**<!-- TODO: INSERT IMAGE - Terminal showing scattered Go cleanup commands -->

---

## The Many Tools Problem

| What You Want | Tool | Command |
|--------------|------|---------|
| Clean module cache | Go | `go clean -modcache` |
| Clean build cache | Go | `go clean -cache` |
| Clean vendor | Manual | `rm -rf vendor/` |
| Find old projects | Manual | `find` |
| Check sizes | Manual | `du -sh` |

Multiple commands. No unified view. No safety.

---

## The Real Solution: null-e for Go Developers

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
```

null-e understands Go projects and cleans them safely.

### What null-e Does Better

| Feature | null-e | go clean | Manual |
|---------|--------|----------|--------|
| **Multi-project** | ✅ Scans all | ❌ Single | ❌ Manual |
| **Selective cleaning** | ✅ Choose what | ❌ All/nothing | ⚠️ Risky |
| **Size info** | ✅ Shows MB/GB | ❌ No info | ❌ Slow |
| **Stale detection** | ✅ Finds old | ❌ No | ❌ No |
| **Git protection** | ✅ Checks status | ❌ No | ❌ No |
| **Safety levels** | ✅ Every item | ❌ No | ❌ No |

### Find All Go Bloat

```bash
# Scan for all Go projects
null-e ~/projects

# Output:
✓ Found 15 Go projects with 8.7 GB cleanable

   [1] ○ api-server (450 MB) - 2 weeks ago, vendor/: 380MB
   [2] ○ cli-tool (120 MB) - 3 months ago, vendor/: 95MB
   [3] ○ old-service (200 MB) - 8 months ago, vendor/: 180MB
   ...
```

See every Go project. Vendored or not. Exact sizes. What's safe to clean.

**<!-- TODO: INSERT IMAGE - Screenshot of null-e showing Go projects with sizes -->

### Check Module Cache

```bash
# Check Go caches
null-e caches

# Output:
✓ Found caches (4.2 GB)
   [1] 🐹 Go module cache    3.2 GB  (~/go/pkg/mod/)
   [2] 🐹 Go build cache      450 MB  (~/Library/Caches/go-build/)
```

Module cache and build cache. Both visible. Both cleanable.

### Find Stale Projects

```bash
# Projects not touched in 180 days
null-e stale ~/projects --days 180

# Safe to clean - you haven't touched them in 6 months
```

Old experiments. Finished services. Safe to clean.

### Clean with Safety

```bash
# Clean with git protection (default)
null-e clean ~/projects

# Block if uncommitted changes
null-e clean -p block ~/projects

# Dry run first
null-e clean --dry-run ~/projects
```

- ✅ Git protection enabled
- ✅ Moves to trash (recoverable)
- ✅ Safety levels on every item

**<!-- TODO: INSERT IMAGE - Screenshot of null-e clean with Go projects -->

---

## Go-Specific Cleanup with null-e

### What null-e Finds in Go Projects

```bash
null-e ~/projects

# Shows:
✓ Found 15 Go projects with 8.7 GB cleanable
   [1] ○ api-server (450 MB) - last touched: 2 weeks ago
       ├── vendor/           380 MB
       ├── *.test binaries   15 MB
       └── go.mod            (tracked)
   [2] ○ cli-tool (120 MB) - last touched: 3 months ago
       └── vendor/           95 MB
```

Vendor directories. Test binaries. Size breakdown. Project age.

### Selective Vendor Cleaning

null-e doesn't blindly delete vendor:

```bash
null-e ~/projects --clean

# Interactive prompt:
✓ Found 15 Go projects with 8.7 GB total

   [1] ○ api-server (450 MB) - 2 weeks ago, has vendor/
   [2] ○ cli-tool (120 MB) - 3 months ago, has vendor/
   [3] ○ old-service (200 MB) - 8 months ago, has vendor/

Clean which projects? (e.g., 2,3 or all)
> 2,3

⚠️ Note: These projects have vendor/ directories.
   You can re-create with: go mod vendor

Continue? [Y/n]
> Y

✓ Cleaned 2 projects, freed 320 MB
```

Old projects cleaned. Recent ones preserved. Clear warnings.

**<!-- TODO: INSERT IMAGE - Screenshot showing vendor directory cleaning with warnings -->

### Module Cache Cleanup

```bash
null-e caches --clean

# Shows:
Clean which caches?
   [1] 🐹 Go module cache    3.2 GB
   [2] 🐹 Go build cache      450 MB

> 1

⚠️ Cleaning module cache will require re-downloading modules on next build.
   This is safe but may slow down the next build.

Continue? [Y/n]
> Y

✓ Cleaned Go module cache, freed 3.2 GB
```

Clear warnings. Informed decisions.

### Build Cache Cleanup

```bash
null-e caches --clean

# Clean build cache (always safe)
> 2

✓ Cleaned Go build cache, freed 450 MB
```

Build cache is always safe—it just regenerates.

**<!-- TODO: INSERT IMAGE - Before/After showing Go cache cleanup -->

---

## Real Results from Real Go Developers

### Case Study: The Vendor Directory Collector

> *"I checked my Go projects and found 25 of them had vendor directories. Total size: 2.1GB. Most were old."* — **Go developer**

25 projects. 2.1GB vendor directories. null-e cleans old ones safely.

### Case Study: The Module Cache Explosion

> *"My ~/go/pkg/mod/ was 5GB. I had modules from projects I worked on 2 years ago."* — **Go developer**

5GB module cache. Years old dependencies. null-e identifies and cleans.

### Case Study: The CI Machine

> *"Our CI runners had 10GB of Go module cache across builds."* — **DevOps engineer**

10GB CI cache. null-e's selective cleaning fixes this.

**<!-- TODO: INSERT IMAGE - Before/After comparison showing Go disk space reclaimed -->

---

## The Go Developer's Cleanup Workflow

### Step 1: Scan Everything

```bash
# Find all Go bloat
null-e ~/projects ~/work ~/go-projects
```

See the full picture. Vendored and non-vendored.

### Step 2: Identify Stale Projects

```bash
# Find old projects
null-e stale ~/projects --days 180

# Safe to clean - you haven't touched them in 6 months
```

### Step 3: Clean Global Caches

```bash
# Clean Go module and build cache
null-e caches --clean
```

Reclaim 2-6GB instantly.

### Step 4: Clean Safely

```bash
# Clean with full protection
null-e clean ~/projects

# Or deep sweep everything
null-e sweep --clean
```

### Step 5: Make It Automatic

```bash
# Add to your shell profile
alias goclean='null-e caches --clean-all && null-e stale ~/projects --days 90 --clean'

# Run monthly
# Or add to cron:
0 0 1 * * /usr/local/bin/null-e caches --clean-all --force
```

**<!-- TODO: INSERT IMAGE - Workflow diagram: Scan → Identify → Clean → Automate -->

---

## Optimizing Go Build Storage

### Avoid Vendoring (If Possible)

```bash
# Instead of vendor/, use go.mod
# Go will download modules to shared cache
# No duplication across projects
```

Vendoring is useful for:
- Air-gapped environments
- Reproducible builds
- But it costs disk space

### Use Go 1.13+ Module Proxy

```bash
# Set up module proxy (corporate or public)
export GOPROXY=https://proxy.golang.org,direct

# Faster downloads, shared cache
```

### Clean Test Binaries

```bash
# Add to .gitignore
*.test

# Then clean regularly
null-e ~/projects --clean
```

### Multiple Go Versions

```bash
# If using g or goversion
# Clean old versions
rm -rf ~/.go/versions/1.19.*  # Old versions
```

null-e finds these too.

**<!-- TODO: INSERT IMAGE - Code snippets showing Go optimization tips -->

---

## Take Back Your Disk Space Today

Don't let vendor/ and module cache own your machine.

**[Install null-e →](https://github.com/us/null-e)**

```bash
# Install
cargo install null-e

# Scan your Go projects
null-e ~/projects

# Find stale projects (6+ months old)
null-e stale ~/projects --days 180

# Clean safely with git protection
null-e clean ~/projects
```

### What You'll Reclaim

| Category | Typical Savings |
|----------|---------------|
| Stale vendor/ directories | 1-5 GB |
| Go module cache | 2-8 GB |
| Go build cache | 500MB-2GB |
| Test binaries | 100MB-500MB |
| Old GOPATH artifacts | 100MB-1GB |
| **Total** | **3.7-16.5 GB** |

Not as massive as Rust or Java, but significant. And it matters.

That's not just disk space. That's:
- ✅ Cleaner project directories
- ✅ Faster file operations
- ✅ No old vendor/ directories confusing you
- ✅ Professional pride in a clean machine

> *"Go is lightweight, but even lightweight accumulates."* — **Go wisdom**

Keep it clean. Keep it fast.

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
null-e sweep
```

Clean up the Go bloat. Reclaim your disk.

```
     .---.
    |o   o|   "Directive: Clean all the vendor/ directories!"
    |  ^  |
    | === |
    `-----'
     /| |\
```

**[View on GitHub →](https://github.com/us/null-e)**

---

### More Go Cleanup Guides

- [Go vendor/ Directory Cleanup Guide](/go-vendor-directory-cleanup/)
- [Clean Go Module Cache Safely](/clean-go-module-cache/)
- [Go Modules vs Vendor: Disk Space Comparison](/go-modules-vs-vendor/)
- [Optimizing Go CI/CD Disk Usage](/go-ci-cd-disk-optimization/)

**<!-- TODO: INSERT IMAGE - Related posts grid with Go-specific thumbnails -->