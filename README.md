<div align="center">

<img src="assets/logo-200.png" alt="null-e mascot" width="120" />

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

### v0.3.0

### ⚠ BREAKING CHANGES

* config directory changed from ~/.config/devsweep to ~/.config/null-e

### Features

* add null-e text next to robot mascot ([1846f3a](https://github.com/us/null-e/commit/1846f3a110755a6a01f09d7e543342c0207a05a7))
* added multi-platform distribution support ([dfeb712](https://github.com/us/null-e/commit/dfeb7121b009cfc06600f6345f4b4531055ddecc))
* added tui ([7feb994](https://github.com/us/null-e/commit/7feb9942ec6c05bfcd1ab9f8130d940200140341))
* apply Minimal Jekyll theme ([dbc0ee3](https://github.com/us/null-e/commit/dbc0ee3741624022b8dc79898e4ba5271e384c0c))
* apply project template (release-please, Makefile, pre-commit, README format) ([9e15641](https://github.com/us/null-e/commit/9e1564165962cb321cd1b5769daeed1c3059940b))
* complete project review, fix 15 bugs, improve detection coverage ([6ef1bf6](https://github.com/us/null-e/commit/6ef1bf6c3cde9660969f41fb64c0fbff1cdea67f))
* major TUI improvements and new cleaners ([1754edc](https://github.com/us/null-e/commit/1754edc2aaf83b3ffe9756e74a3753df34c13556))
* modernize Jekyll site with professional design ([02933ce](https://github.com/us/null-e/commit/02933ce92ba772c2981c3450912daba775b51d0e))


### Bug Fixes

* add Jekyll layout templates for proper rendering ([bbb4bbb](https://github.com/us/null-e/commit/bbb4bbbca6d86f47719732fe3ff13800ab11cd08))
* gate platform-specific imports with cfg attributes ([d05bee6](https://github.com/us/null-e/commit/d05bee6c65ef918180cb73a0f26b99c7278a2c1e))
* hardcode footer bg, increase text opacity, add article padding ([8abeb6c](https://github.com/us/null-e/commit/8abeb6cd575cbec36650c8cb4e4a12605ac0436b))
* keep CleanableItem and Result imports available for non-macOS stub ([1c5ec43](https://github.com/us/null-e/commit/1c5ec4329f56e77bf1618217041094743ce3f870))
* limit ASCII art width to prevent overflow ([60e02e7](https://github.com/us/null-e/commit/60e02e7720a209ca1c1379d06cb36b56b925f379))
* move ASCII art outside center div to prevent alignment issues ([6c4db7d](https://github.com/us/null-e/commit/6c4db7db9bc1006cae28cb651fc52e8213d9bdc4))
* prevent markdown headings inside code blocks ([69a1d91](https://github.com/us/null-e/commit/69a1d9174efe018911c3766fbab91a86a09a53be))
* reduce keywords to 5 for crates.io ([664ebbd](https://github.com/us/null-e/commit/664ebbd0d413fd116eece77bdb98e6e1433fb54f))
* replace ASCII art with text to prevent overflow ([9be3dd2](https://github.com/us/null-e/commit/9be3dd28dee2636f7abf58bcbc37b80f532ed5db))
* restore ASCII art with ultra-small font size (7px) to prevent overflow ([16166df](https://github.com/us/null-e/commit/16166df9c3c10199b161555d8c77565ab5530b78))
* sidebar overflow and blog section improvements ([909cee9](https://github.com/us/null-e/commit/909cee9555d59828e6c5e8c3eca4d423d5006c05))
* update footer to TMLS design with brand + meta layout ([974445c](https://github.com/us/null-e/commit/974445c42ce3ab413cdbd34d9e335cdaa170e501))

## Changelog

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

## Disclaimer

**null-e** is provided "as-is" without warranty of any kind. The authors are
not responsible for any data loss resulting from the use of this software.

- **Always keep backups** of important work before running cleanup operations
- **Use `--dry-run`** to preview what will be deleted before committing
- **Trash mode** (default) allows recovery, but is not guaranteed on all filesystems
- **Permanent delete** is irreversible — use with extreme caution
- This tool is designed for developers who understand the artifacts being cleaned
