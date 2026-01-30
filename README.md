# null-e - Disk Cleanup Tool for Developers 🤖

```
     .---.
    |o   o|    null-e
    |  ^  |    ═══════════════════════════════════
    | === |    The friendly disk cleanup robot
    `-----'    Send your cruft to /dev/null!
     /| |\
```

> **Clean node_modules, target, .venv, Docker images, Xcode DerivedData, and 50+ cache types. Reclaim 100+ GB of disk space.**

**The Friendly Disk Cleanup Robot** - Inspired by Wall-E, powered by Rust.

null-e is a fast, cross-platform disk cleanup CLI tool that helps developers reclaim disk space by finding and cleaning development artifacts, build caches, and unused files. Works on **macOS**, **Linux**, and **Windows**.

[![Crates.io](https://img.shields.io/crates/v/null-e.svg)](https://crates.io/crates/null-e)
[![Downloads](https://img.shields.io/crates/d/null-e.svg)](https://crates.io/crates/null-e)
![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)
![License](https://img.shields.io/badge/license-WTFPL-brightgreen.svg)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)

## Why "null-e"?

> `/dev/null` + Wall-E = **null-e** 🤖
>
> Like the adorable trash-compacting robot from the movie, null-e tirelessly cleans up your developer junk and sends it where it belongs - to `/dev/null`!

## What Can null-e Clean?

| Category              | Examples                                          | Typical Size |
| --------------------- | ------------------------------------------------- | ------------ |
| **Project Artifacts** | `node_modules`, `target`, `.venv`, `build`        | 10-100 GB    |
| **Global Caches**     | npm, pip, cargo, go, maven, gradle                | 5-50 GB      |
| **Xcode**             | DerivedData, Simulators, Archives, Device Support | 20-100 GB    |
| **Docker**            | Images, Containers, Volumes, Build Cache          | 10-100 GB    |
| **Android**           | AVD, Gradle, SDK Components                       | 5-30 GB      |
| **ML/AI**             | Huggingface models, Ollama, PyTorch cache         | 10-100 GB    |
| **IDE Caches**        | JetBrains, VS Code, Cursor                        | 2-20 GB      |
| **Homebrew**          | Downloads, Old versions                           | 2-20 GB      |
| **iOS Dependencies**  | CocoaPods, Carthage, SPM                          | 1-10 GB      |
| **Electron Apps**     | Slack, Discord, Spotify, Teams caches             | 1-5 GB       |
| **Game Dev**          | Unity, Unreal, Godot caches                       | 10-100 GB    |
| **Cloud CLI**         | AWS, GCP, Azure, kubectl, Terraform               | 1-5 GB       |
| **macOS System**      | Orphaned containers, App remnants                 | 1-20 GB      |

## Features

- **Multi-language Support**: Node.js, Rust, Python, Go, Java, .NET, Swift, Ruby, PHP, and more
- **Git Protection**: Never accidentally delete uncommitted changes
- **Safe Deletion**: Moves to trash by default with recovery option
- **Parallel Scanning**: Fast directory traversal using multiple threads
- **Analysis Tools**: Find stale projects, duplicate dependencies, optimize git repos
- **Beautiful CLI**: Colored output with progress indicators
- **Cross-Platform**: macOS, Linux, Windows support

## Installation

### Using Cargo (Recommended)

```bash
cargo install null-e
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/us/null-e/releases):

- **macOS**: `null-e-darwin-aarch64.tar.gz` (Apple Silicon) / `null-e-darwin-x86_64.tar.gz` (Intel)
- **Linux**: `null-e-linux-x86_64.tar.gz` / `null-e-linux-aarch64.tar.gz`
- **Windows**: `null-e-windows-x86_64.zip`

### Package Managers

```bash
# Homebrew (macOS/Linux) - requires 75+ stars for homebrew-core
brew install null-e          # coming soon

# Scoop (Windows)
scoop bucket add us https://github.com/us/scoop-bucket
scoop install null-e

# AUR (Arch Linux)
yay -S null-e

# Nix
nix-env -iA nixpkgs.null-e   # coming soon
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

# Deep sweep - find ALL cleanable items across your system
null-e sweep

# Clean global developer caches (npm, pip, cargo, etc.)
null-e caches

# Analyze git repositories for optimization
null-e git-analyze ~/projects

# Find stale projects not touched in 6 months
null-e stale ~/projects

# Find duplicate dependencies
null-e duplicates ~/projects
```

## Commands Overview

### Core Commands

| Command         | Description                                  |
| --------------- | -------------------------------------------- |
| `null-e`        | Scan current directory for project artifacts |
| `null-e scan`   | Scan with detailed output                    |
| `null-e clean`  | Clean found artifacts (interactive)          |
| `null-e sweep`  | Deep scan for ALL cleanable items            |
| `null-e caches` | Manage global developer caches               |

### Specialized Cleaners

| Command           | Description                         |
| ----------------- | ----------------------------------- |
| `null-e xcode`    | Clean Xcode artifacts               |
| `null-e android`  | Clean Android development artifacts |
| `null-e docker`   | Clean Docker resources              |
| `null-e ml`       | Clean ML/AI model caches            |
| `null-e ide`      | Clean IDE caches                    |
| `null-e homebrew` | Clean Homebrew caches               |
| `null-e ios-deps` | Clean iOS dependency caches         |
| `null-e electron` | Clean Electron app caches           |
| `null-e gamedev`  | Clean game development caches       |
| `null-e cloud`    | Clean cloud CLI caches              |
| `null-e macos`    | Clean macOS system caches           |

### Analysis Tools

| Command              | Description                           |
| -------------------- | ------------------------------------- |
| `null-e git-analyze` | Find large .git repos, suggest git gc |
| `null-e stale`       | Find projects not touched in months   |
| `null-e duplicates`  | Find duplicate dependencies           |

## Usage Examples

### Basic Scanning

```bash
# Scan current directory
null-e

# Scan specific directories
null-e ~/projects ~/work

# Scan with depth limit
null-e -d 5 ~/projects

# Filter by minimum size
null-e -s 100MB ~/projects

# Show all results (no limit)
null-e -a ~/projects

# Verbose output
null-e -v ~/projects
```

### Deep Sweep

```bash
# Find everything cleanable
null-e sweep

# Filter by category
null-e sweep --category xcode
null-e sweep --category docker

# Clean interactively
null-e sweep --clean
```

### Global Caches

```bash
# Show all global caches
null-e caches

# Clean selected caches interactively
null-e caches --clean

# Clean all caches
null-e caches --clean-all
```

### Xcode Cleanup

```bash
# Show Xcode artifacts
null-e xcode

# Clean DerivedData, old simulators, etc.
null-e xcode --clean
```

### Docker Cleanup

```bash
# Show Docker resources
null-e docker

# Clean (excluding volumes)
null-e docker --clean

# Clean including volumes (careful!)
null-e docker --clean --volumes
```

### Git Analysis

```bash
# Analyze git repos in a directory
null-e git-analyze ~/projects

# Run git gc on repos that need it
null-e git-analyze ~/projects --fix

# Dry run
null-e git-analyze ~/projects --fix --dry-run
```

### Stale Project Detection

```bash
# Find projects not touched in 180 days (default)
null-e stale ~/projects

# Custom threshold (90 days)
null-e stale --days 90 ~/projects

# Clean build artifacts from stale projects
null-e stale --days 90 --clean ~/projects
```

### Duplicate Detection

```bash
# Find duplicate dependencies
null-e duplicates ~/projects

# Verbose output with details
null-e duplicates -v ~/projects
```

## Protection Levels

null-e protects your uncommitted work:

```bash
# Warn about uncommitted changes (default)
null-e clean -p warn

# Block cleaning repos with uncommitted changes
null-e clean -p block

# No protection (dangerous!)
null-e clean -p none

# Paranoid mode - require confirmation for everything
null-e clean -p paranoid
```

## Deletion Methods

```bash
# Move to trash (default, safe)
null-e clean -m trash

# Permanent delete (careful!)
null-e clean -m permanent

# Dry run (no deletion)
null-e clean -m dry-run
# or
null-e clean -n
```

## Configuration

Create `~/.config/null-e/config.toml`:

```toml
[general]
default_paths = ["~/projects", "~/work"]
verbose = false

[scan]
max_depth = 10
skip_hidden = true
min_size = 1000000  # 1 MB

[clean]
delete_method = "trash"
protection_level = "warn"

[ui]
use_icons = true
sort_by = "size"
```

### Config Commands

```bash
# Show current config
null-e config

# Initialize config file
null-e config --init

# Show config path
null-e config --path
```

## Project Types Supported

| Language/Framework | Marker Files                         | Cleanable Artifacts                                |
| ------------------ | ------------------------------------ | -------------------------------------------------- |
| **Node.js**        | `package.json`                       | `node_modules`, `.next`, `.nuxt`, `dist`, `.cache` |
| **Rust**           | `Cargo.toml`                         | `target/`                                          |
| **Python**         | `requirements.txt`, `pyproject.toml` | `.venv`, `__pycache__`, `.pytest_cache`            |
| **Go**             | `go.mod`                             | `vendor/`                                          |
| **Java/Kotlin**    | `pom.xml`, `build.gradle`            | `target/`, `build/`, `.gradle/`                    |
| **.NET**           | `*.csproj`                           | `bin/`, `obj/`                                     |
| **Swift**          | `Package.swift`                      | `.build/`, `.swiftpm/`                             |
| **Ruby**           | `Gemfile`                            | `vendor/bundle`, `.bundle`                         |
| **PHP**            | `composer.json`                      | `vendor/`                                          |

## Safety Levels

Each cleanable item has a safety level:

| Level            | Symbol | Meaning                                    |
| ---------------- | ------ | ------------------------------------------ |
| **Safe**         | `✓`    | Safe to delete, will be regenerated        |
| **SafeWithCost** | `~`    | Safe but may slow down next operation      |
| **Caution**      | `!`    | May lose some data, verify before deleting |
| **Dangerous**    | `⚠`    | High risk, may break things                |

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

## Documentation

Detailed documentation for each module:

- [Cleaners Guide](docs/CLEANERS.md) - System cleaners (Xcode, Docker, etc.)
- [Caches Guide](docs/CACHES.md) - Global cache management
- [Analysis Guide](docs/ANALYSIS.md) - Analysis tools (git, stale, duplicates)

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Project Structure

```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Library exports
├── analysis/         # Analysis tools
│   ├── git.rs        # Git repository analysis
│   ├── stale.rs      # Stale project detection
│   └── duplicates.rs # Duplicate dependency detection
├── caches/           # Global cache management
├── cleaners/         # System cleaners
│   ├── xcode.rs      # Xcode cleaner
│   ├── android.rs    # Android cleaner
│   ├── docker.rs     # Docker cleaner
│   ├── ml.rs         # ML/AI cleaner
│   ├── ide.rs        # IDE cleaner
│   ├── homebrew.rs   # Homebrew cleaner
│   ├── ios_deps.rs   # iOS dependencies cleaner
│   ├── electron.rs   # Electron apps cleaner
│   ├── gamedev.rs    # Game development cleaner
│   ├── cloud.rs      # Cloud CLI cleaner
│   ├── macos.rs      # macOS system cleaner
│   └── logs.rs       # Log cleaner
├── plugins/          # Language plugins
├── core/             # Core scanning/cleaning logic
├── docker/           # Docker integration
├── git/              # Git integration
├── trash/            # Trash/delete operations
└── config/           # Configuration management
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Adding a New Cleaner

1. Create a new file in `src/cleaners/`
2. Implement the cleaner with `detect()` method
3. Add to `src/cleaners/mod.rs`
4. Add CLI command in `src/main.rs`
5. Add tests

### Adding a New Plugin

1. Create a new file in `src/plugins/`
2. Implement the `Plugin` trait
3. Register in `src/plugins/mod.rs`
4. Add tests

## License

This project is licensed under the WTFPL - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [Wall-E](https://www.pixar.com/feature-films/wall-e) (the adorable trash robot!)
- Also inspired by [npkill](https://github.com/voidcosmos/npkill) and [kondo](https://github.com/tbillington/kondo)
- Built with [Rust](https://www.rust-lang.org/)

---

```
     .---.
    |o   o|   "Directive: Clean all the things!"
    |  ^  |
    | === |   Made with 💚 for developers who need their disk space back.
    `-----'
     /| |\
```
