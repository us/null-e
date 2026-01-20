# Analysis Tools Guide

null-e includes intelligent analysis tools that go beyond simple cleanup. These tools help you understand your codebase, find optimization opportunities, and maintain a healthy development environment.

## Overview

| Tool | Command | Purpose |
|------|---------|---------|
| [Git Analyzer](#git-analyzer) | `null-e git-analyze` | Find large repos, optimize with git gc |
| [Stale Finder](#stale-project-finder) | `null-e stale` | Find projects not touched in months |
| [Duplicate Finder](#duplicate-finder) | `null-e duplicates` | Find duplicate dependencies |

---

## Git Analyzer

**Command:** `null-e git-analyze`

Analyzes git repositories to find optimization opportunities.

### What It Detects

| Issue | Description | Solution |
|-------|-------------|----------|
| **Large .git directories** | Repos over 100 MB | Review if expected |
| **Loose objects** | Uncompressed git objects | Run `git gc` |
| **Git LFS cache** | Large file storage cache | Run `git lfs prune` |

### How It Works

1. **Scans for .git directories** in the specified path
2. **Measures repository size** and counts loose objects
3. **Identifies optimization opportunities** based on:
   - Total .git size > 100 MB
   - Loose objects > 1000
   - Loose object size > 50 MB
4. **Estimates potential savings** from running `git gc`

### Usage

```bash
# Analyze repositories in a directory
null-e git-analyze ~/projects

# Scan with custom depth
null-e git-analyze -d 5 ~/projects

# Run git gc on repositories that need it
null-e git-analyze ~/projects --fix

# Dry run - see what would happen
null-e git-analyze ~/projects --fix --dry-run

# Verbose output
null-e git-analyze -v ~/projects
```

### Example Output

```
🔍 Git Repository Analysis v0.1.0

✓ Found 7 repositories with potential savings of 1.72 GiB

       Repository                                  Savings   Action
   ─────────────────────────────────────────────────────────────────
   [1] ✓ Large .git: my-project (2.85 GiB)       1.64 GiB   1468 loose objects...
   [2] ✓ Large .git: another-repo (131 MiB)     79.03 MiB   594 loose objects...
   [3] ✓ Large .git: blog (121 MiB)                     -   Already well-packed
   ─────────────────────────────────────────────────────────────────
       Total Potential Savings                    1.72 GiB

💡 Use null-e git-analyze --fix to run git gc on these repositories
```

### Understanding the Output

- **Savings** column shows estimated space that could be reclaimed
- **"-"** means the repository is already well-packed
- **Action** describes what's causing the size

### The --fix Flag

When you run with `--fix`, null-e executes:

```bash
git gc --aggressive --prune=now
```

This:
- Compresses loose objects into pack files
- Removes unreachable objects
- Optimizes repository structure

### Risk Level

| Symbol | Risk | Meaning |
|--------|------|---------|
| `✓` | None | Safe optimization, no data loss |

Git gc is always safe - it only compresses and optimizes, never deletes committed data.

---

## Stale Project Finder

**Command:** `null-e stale`

Finds development projects that haven't been touched in a long time.

### What It Detects

| Metric | Threshold | Description |
|--------|-----------|-------------|
| **Last Activity** | 180 days (default) | Based on git commits or file modification |
| **Minimum Size** | 50 MB | Only reports significant projects |
| **Cleanable Artifacts** | Present | Build artifacts that can be safely removed |

### Project Types Detected

| Type | Icon | Marker Files | Cleanable Directories |
|------|------|--------------|----------------------|
| Node.js | 📦 | `package.json` | `node_modules`, `.next`, `dist`, `build` |
| Rust | 🦀 | `Cargo.toml` | `target` |
| Python | 🐍 | `pyproject.toml`, `requirements.txt` | `venv`, `.venv`, `__pycache__` |
| Go | 🐹 | `go.mod` | - |
| Java | ☕ | `pom.xml`, `build.gradle` | `target`, `build`, `.gradle` |
| Swift | 🍎 | `Package.swift` | `DerivedData`, `.build`, `Pods` |
| Ruby | 💎 | `Gemfile` | `vendor/bundle`, `.bundle` |

### How It Works

1. **Scans for project markers** (package.json, Cargo.toml, etc.)
2. **Checks last activity** via:
   - Git last commit date (preferred)
   - File system modification time (fallback)
3. **Calculates cleanable size** from build artifacts
4. **Reports projects** older than threshold

### Usage

```bash
# Find projects not touched in 180 days (default)
null-e stale ~/projects

# Custom threshold - 90 days
null-e stale --days 90 ~/projects

# Very old projects - 1 year
null-e stale --days 365 ~/projects

# Clean build artifacts from stale projects
null-e stale --days 90 --clean ~/projects

# Verbose output with cleanup commands
null-e stale -v ~/projects

# Dry run
null-e stale --days 90 --clean --dry-run ~/projects
```

### Example Output

```
📦 Stale Project Finder v0.1.0
  Looking for projects not touched in 180 days...

✓ Found 28 stale projects with 5.47 GiB in cleanable artifacts

       Project                                              Cleanable
   ──────────────────────────────────────────────────────────────────
   [1] ! 📦 old-frontend (386 MiB) - 1 years stale          archive?
   [2] ○ 🐍 ml-experiment (280 MiB) - 1 years stale        274.89 MiB
   [3] ○ 📦 test-project (774 MiB) - 7 months stale        701.98 MiB
   ──────────────────────────────────────────────────────────────────
       Total Cleanable Artifacts                            5.47 GiB

💡 Use null-e stale --days 180 --clean to clean build artifacts
```

### Understanding the Output

| Symbol | Risk Level | Meaning |
|--------|------------|---------|
| `○` | Low | Has cleanable artifacts, safe to clean |
| `!` | High | No cleanable artifacts, consider archiving/deleting project |

- **Cleanable** column shows size of build artifacts
- **"archive?"** means the project has no build artifacts - consider archiving or deleting the entire project

### The --clean Flag

When you run with `--clean`, null-e removes build artifacts:

| Project Type | What's Removed |
|--------------|----------------|
| Node.js | `node_modules/` |
| Rust | Runs `cargo clean` |
| Python | `.venv/`, `venv/` |
| Java | `target/`, `build/` |
| Swift | `.build/`, `DerivedData/` |
| Ruby | `vendor/bundle/` |

### Best Practices

1. **Start with a dry run**: `null-e stale --clean --dry-run`
2. **Review "archive?" projects**: These might be better archived or deleted entirely
3. **Back up before cleaning**: Especially for projects you haven't touched in over a year
4. **Consider git status**: null-e doesn't check for uncommitted changes in stale projects

---

## Duplicate Finder

**Command:** `null-e duplicates`

Finds duplicate dependencies across your projects.

### What It Detects

| Type | Description | Solution |
|------|-------------|----------|
| **Node.js duplicates** | Same package in multiple node_modules | Use pnpm or yarn workspaces |
| **Python venvs** | Multiple virtual environments | Use uv or poetry with centralized cache |
| **Rust targets** | Multiple target directories | Use shared `CARGO_TARGET_DIR` |

### How It Works

#### Node.js Detection

1. Scans all `node_modules/*/package.json` files
2. Groups packages by name
3. Identifies packages appearing in multiple locations
4. Calculates size and potential savings

#### Python Detection

1. Finds all virtual environments (`venv/`, `.venv/`, `env/`)
2. Counts total environments
3. Estimates shared package overlap (~40%)

#### Rust Detection

1. Finds all `target/` directories next to `Cargo.toml`
2. Calculates total size
3. Estimates savings from shared target (~35%)

### Usage

```bash
# Find duplicates in a directory
null-e duplicates ~/projects

# Scan with deeper depth
null-e duplicates -d 10 ~/projects

# Verbose output with descriptions
null-e duplicates -v ~/projects
```

### Example Output

```
🔄 Duplicate Dependency Finder v0.1.0

✓ Found 21 duplicate patterns with 3.31 GiB potential savings

       Duplicate                                     Savings   Recommendation
   ──────────────────────────────────────────────────────────────────────────
   [1] 🐍 27 Python venvs (7.55 GiB)               3.02 GiB   Use uv or poetry
   [2] 📦 pdfjs-dist (2 copies, 70 MiB)           35.20 MiB   Use pnpm
   [3] 📦 lodash (10 copies, 13 MiB)              12.12 MiB   Use pnpm
   [4] 🦀 5 Rust targets (2.1 GiB)               735.00 MiB   Use CARGO_TARGET_DIR
   ──────────────────────────────────────────────────────────────────────────
       Total Potential Savings                     3.31 GiB

💡 Recommendations:
   • Use pnpm or yarn workspaces for Node.js deduplication
   • Set CARGO_TARGET_DIR=~/.cargo/target for Rust shared compilation
   • Use uv or poetry for Python
```

### Solutions

#### Node.js: Use pnpm

pnpm uses a content-addressable store, so identical packages are stored once:

```bash
# Install pnpm
npm install -g pnpm

# Use pnpm instead of npm
pnpm install
```

#### Node.js: Use Yarn Workspaces

For monorepos, yarn workspaces hoist shared dependencies:

```json
// package.json
{
  "workspaces": ["packages/*"]
}
```

#### Python: Use uv

uv uses a global cache for packages:

```bash
# Install uv
curl -LsSf https://astral.sh/uv/install.sh | sh

# Use uv instead of pip
uv pip install package
```

#### Python: Use poetry

Poetry with its cache shares packages across projects:

```bash
# Install poetry
curl -sSL https://install.python-poetry.org | python3 -

# Create virtual environment
poetry install
```

#### Rust: Shared Target Directory

Set a shared target directory for all Rust projects:

```bash
# Add to ~/.bashrc or ~/.zshrc
export CARGO_TARGET_DIR=~/.cargo/target
```

Or in `~/.cargo/config.toml`:

```toml
[build]
target-dir = "/Users/you/.cargo/target"
```

### Limitations

- **Node.js**: Different versions of the same package are not considered duplicates
- **Python**: Exact package overlap is estimated, not calculated
- **Rust**: Only detects projects with standard `target/` directory

---

## Risk Levels

All analysis tools use consistent risk levels:

| Symbol | Level | Description |
|--------|-------|-------------|
| `✓` | None | No risk, purely optimization |
| `○` | Low | Low risk, easily reversible |
| `~` | Medium | May require rebuild/reinstall |
| `!` | High | May lose data, verify first |

---

## Combining Analysis Tools

For a complete analysis of your projects:

```bash
# 1. Find git optimization opportunities
null-e git-analyze ~/projects

# 2. Find stale projects
null-e stale ~/projects

# 3. Find duplicate dependencies
null-e duplicates ~/projects

# 4. Deep sweep for cleanable items
null-e sweep
```

### Recommended Workflow

1. **Run analysis tools** to understand your codebase
2. **Review recommendations** before taking action
3. **Start with --dry-run** for any cleaning operations
4. **Fix duplicates** by adopting better tools (pnpm, uv, shared cargo target)
5. **Clean stale projects** to reclaim space
6. **Optimize git repos** with git gc

---

## Technical Details

### Performance

- Analysis tools use **parallel processing** for speed
- Large directories are scanned efficiently using **walkdir**
- Results are cached where possible

### Thresholds

| Tool | Default Threshold | Configurable |
|------|-------------------|--------------|
| Git Analyzer | 100 MB min size | Yes (in code) |
| Stale Finder | 180 days, 50 MB | Yes (`--days`) |
| Duplicate Finder | 10 MB total | Yes (in code) |

### File Access

- Analysis tools are **read-only** by default
- Only `--fix` or `--clean` flags modify files
- Always use `--dry-run` first to preview changes
