<div align="center">

```
     .---.
    |o   o|    null-e
    |  ^  |    ═══════════════════════════════════
    | === |    The friendly disk cleanup robot
    `-----'    Send your cruft to /dev/null!
     /| |\
```

# null-e

**Disk cleanup CLI for developers — clean node_modules, target, .venv, Docker, Xcode caches and 50+ cache types**

[![CI](https://github.com/us/null-e/actions/workflows/ci.yml/badge.svg)](https://github.com/us/null-e/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/null-e.svg)](https://crates.io/crates/null-e)
[![Downloads](https://img.shields.io/crates/d/null-e.svg)](https://crates.io/crates/null-e)
[![License](https://img.shields.io/badge/license-WTFPL-brightgreen.svg)](LICENSE)

[Quick Start](#quick-start) • [Docs](https://us.github.io/null-e) • [Changelog](CHANGELOG.md)

</div>

---

`/dev/null` + Wall-E = **null-e** — like the adorable trash-compacting robot, null-e tirelessly cleans up developer junk and sends it where it belongs.

**Fast parallel scanning. Git-aware protection. 8 language plugins, 18 system cleaners, 50+ cache targets. Interactive TUI.**

> **Reclaim 100+ GB** — most developer machines accumulate tens of gigabytes of stale `node_modules`, `target/`, `.venv`, Docker images, and IDE caches across dozens of projects.

```bash
cargo install null-e
null-e sweep
```

## What's New

See [CHANGELOG.md](CHANGELOG.md) for the full release history.

## Why null-e?

| Category | Examples | Typical Size |
|----------|----------|-------------|
| **Project Artifacts** — `node_modules`, `target`, `.venv`, `build` | Node.js, Rust, Python, Go, Java, .NET, Swift | 10-100 GB |
| **Global Caches** — npm, pip, cargo, go, maven, gradle | All major package managers | 5-50 GB |
| **Xcode** — DerivedData, Simulators, Archives | iOS/macOS development | 20-100 GB |
| **Docker** — Images, Containers, Volumes, Build Cache | Container workflows | 10-100 GB |
| **Android** — AVD, Gradle, SDK Components | Android development | 5-30 GB |
| **ML/AI** — HuggingFace, Ollama, PyTorch cache | Machine learning | 10-100 GB |
| **IDE Caches** — JetBrains, VS Code, Cursor | All major IDEs | 2-20 GB |
| **More** — Homebrew, Electron, Game Dev, Cloud CLI, macOS System | Everything else | 1-100 GB |

## Features

- **Multi-language Support** — Node.js, Rust, Python, Go, Java, .NET, Swift, Ruby, PHP and more
- **Git Protection** — 4 levels (none/warn/block/paranoid) to prevent deleting uncommitted work
- **Safe Deletion** — Moves to trash by default with recovery option, dry-run mode
- **Parallel Scanning** — Fast multi-threaded directory traversal with rayon + walkdir
- **Interactive TUI** — 18 scan modes, keyboard navigation, live progress with ratatui
- **Analysis Tools** — Find stale projects, duplicate dependencies, optimize git repos
- **System Cleaners** — Xcode, Docker, Android, ML/AI, IDE, Homebrew, Electron, Cloud CLI, and more
- **Configurable** — TOML config file, CLI flags override, JSON output support
- **Cross-Platform** — macOS, Linux, Windows

## Installation

### Using Cargo

```bash
cargo install null-e
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/us/null-e/releases):

| Platform | File |
|----------|------|
| macOS ARM | `null-e-darwin-aarch64.tar.gz` |
| macOS Intel | `null-e-darwin-x86_64.tar.gz` |
| Linux x86_64 | `null-e-linux-x86_64.tar.gz` |
| Linux x86_64 (musl) | `null-e-linux-x86_64-musl.tar.gz` |
| Linux ARM | `null-e-linux-aarch64.tar.gz` |
| Windows | `null-e-windows-x86_64.zip` |

### Package Managers

```bash
# Homebrew (macOS/Linux)
brew install null-e

# AUR (Arch Linux)
yay -S null-e

# Scoop (Windows)
scoop bucket add us https://github.com/us/scoop-bucket
scoop install null-e
```

### Docker

```bash
docker run -v $(pwd):/workspace ghcr.io/us/null-e
```

### From Source

```bash
git clone https://github.com/us/null-e.git
cd null-e
cargo install --path .
```

## Quick Start

```bash
# Scan current directory for cleanable artifacts
null-e

# Deep sweep — find ALL cleanable items across your system
null-e sweep

# Clean global developer caches (npm, pip, cargo, etc.)
null-e caches

# Interactive TUI mode
null-e tui

# Find stale projects not touched in 6 months
null-e stale ~/projects

# Dry run — see what would be deleted
null-e clean ~/projects --dry-run
```

## Commands

### Core

| Command | Description |
|---------|-------------|
| `null-e` | Scan current directory for project artifacts |
| `null-e scan` | Scan with detailed output |
| `null-e clean` | Clean found artifacts (interactive) |
| `null-e sweep` | Deep scan for ALL cleanable items |
| `null-e caches` | Manage global developer caches |
| `null-e tui` | Launch interactive terminal UI |

### System Cleaners

| Command | Description |
|---------|-------------|
| `null-e xcode` | Xcode artifacts (DerivedData, Simulators, Archives) |
| `null-e android` | Android development artifacts |
| `null-e docker` | Docker resources (images, volumes, build cache) |
| `null-e ml` | ML/AI model caches (HuggingFace, Ollama, PyTorch) |
| `null-e ide` | IDE caches (JetBrains, VS Code, Cursor) |
| `null-e homebrew` | Homebrew caches |
| `null-e ios-deps` | iOS dependency caches (CocoaPods, Carthage, SPM) |
| `null-e electron` | Electron app caches (Slack, Discord, Teams) |
| `null-e gamedev` | Game development caches (Unity, Unreal, Godot) |
| `null-e cloud` | Cloud CLI caches (AWS, GCP, Azure, kubectl, Terraform) |
| `null-e macos` | macOS system caches |

### Analysis

| Command | Description |
|---------|-------------|
| `null-e git-analyze` | Find large `.git` repos, suggest `git gc` |
| `null-e stale` | Find projects not touched in months |
| `null-e duplicates` | Find duplicate dependencies |

## Architecture

```
     .---.
    |o   o|
    |  ^  |    ┌──────────────────────────────────────────────────┐
    | === |    │                       CLI                         │
    `-----'    ├──────────────────────────────────────────────────┤
     /| |\     │                    Core Engine                    │
               │  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
               │  │ Scanner  │  │ Cleaner  │  │ Analysis │        │
               │  │          │  │          │  │  Tools   │        │
               │  └──────────┘  └──────────┘  └──────────┘        │
               ├──────────────────────────────────────────────────┤
               │                    Modules                        │
               │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐     │
               │  │Plugins │ │Cleaners│ │ Caches │ │ Docker │     │
               │  │(langs) │ │(system)│ │(global)│ │        │     │
               │  └────────┘ └────────┘ └────────┘ └────────┘     │
               └──────────────────────────────────────────────────┘
```

## Configuration

Create `~/.config/null-e/config.toml`:

```toml
[general]
default_paths = ["~/projects", "~/work"]

[scan]
max_depth = 10
min_size = 1000000  # 1 MB

[clean]
delete_method = "trash"
protection_level = "warn"

[ui]
sort_by = "size"
use_icons = true
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feat/amazing-feature`)
3. Commit your changes using conventional commits
4. Push to the branch (`git push origin feat/amazing-feature`)
5. Open a Pull Request

## License

[WTFPL](LICENSE) — Do What The Fuck You Want To Public License.
