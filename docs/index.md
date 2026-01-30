---
layout: default
title: null-e - Disk Cleanup Tool for Developers
description: Clean node_modules, target, .venv, Docker images, Xcode caches and 50+ cache types. Reclaim 100+ GB of disk space.
---

<div align="center">

```
     .---.
    |o   o|    null-e
    |  ^  |    ═══════════════════════════════════
    | === |    The friendly disk cleanup robot
    `-----'    Send your cruft to /dev/null!
     /| |\
```

# Reclaim Your Disk Space

**Clean node_modules, target, .venv, Docker, Xcode, and 50+ cache types.**

[![Crates.io](https://img.shields.io/crates/v/null-e.svg)](https://crates.io/crates/null-e)
[![Downloads](https://img.shields.io/crates/d/null-e.svg)](https://crates.io/crates/null-e)

</div>

---

## The Problem

As developers, our disks fill up fast:

- **node_modules** folders everywhere (500MB-2GB each)
- **Rust target/** directories eating gigabytes
- **Python .venv** scattered across projects
- **Docker images** you forgot about
- **Xcode DerivedData** growing endlessly
- **Global caches** from npm, pip, cargo, homebrew...

**The result?** "Your disk is almost full" notifications.

## The Solution

```bash
cargo install null-e
null-e sweep
```

null-e scans your system and finds everything that can be safely cleaned:

| Category | What it finds | Typical savings |
|----------|--------------|-----------------|
| Project Artifacts | node_modules, target, .venv, build | 10-100 GB |
| Global Caches | npm, pip, cargo, go, maven | 5-50 GB |
| Xcode | DerivedData, Simulators, Archives | 20-100 GB |
| Docker | Images, Containers, Build Cache | 10-100 GB |
| ML/AI | Huggingface, Ollama, PyTorch | 10-100 GB |
| IDE Caches | JetBrains, VS Code, Cursor | 2-20 GB |

## Features

- **Fast** - Parallel scanning with Rust
- **Safe** - Git protection, moves to trash by default
- **Smart** - Detects 15+ languages and frameworks
- **Cross-platform** - macOS, Linux, Windows

## Quick Start

```bash
# Install
cargo install null-e

# Scan current directory
null-e

# Deep sweep - find EVERYTHING
null-e sweep

# Clean global caches
null-e caches

# Xcode cleanup (macOS)
null-e xcode

# Docker cleanup
null-e docker
```

## Why "null-e"?

> `/dev/null` + Wall-E = **null-e**
>
> Like the adorable trash-compacting robot from the movie, null-e tirelessly cleans up your developer junk and sends it where it belongs!

## Installation

### Cargo (Recommended)
```bash
cargo install null-e
```

### Pre-built Binaries
Download from [GitHub Releases](https://github.com/us/null-e/releases)

### Package Managers
```bash
# Homebrew (coming soon)
brew install null-e

# AUR (Arch Linux)
yay -S null-e

# Scoop (Windows)
scoop install null-e
```

---

## Blog

{% for post in site.posts limit:5 %}
- [{{ post.title }}]({{ post.url | relative_url }}) - {{ post.date | date: "%B %d, %Y" }}
{% endfor %}

---

<div align="center">

```
     .---.
    |o   o|   "Directive: Clean all the things!"
    |  ^  |
    | === |
    `-----'
     /| |\
```

[GitHub](https://github.com/us/null-e) | [Crates.io](https://crates.io/crates/null-e)

</div>
