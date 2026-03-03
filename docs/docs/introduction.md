# Introduction

null-e is a developer disk cleanup CLI written in Rust. It scans your filesystem for build artifacts, dependency caches, and virtual environments across all major languages and tools — then lets you reclaim the space.

## The Problem

Developer machines accumulate gigabytes of build artifacts and caches across dozens of projects. That side project from 6 months ago still has 800MB of `node_modules`. Your Rust `target/` directories total 15GB. Old Python `.venv` folders, Gradle caches, Docker images — it all adds up.

## Features

- **8 language plugins** — Node.js, Rust, Python, Go, Java (Maven + Gradle), .NET, Swift
- **18 system cleaners** — Xcode, Docker, Android, ML/AI models, IDE caches, Homebrew, Electron apps, cloud CLIs, game engines, and more
- **30+ global cache targets** — npm, pip, cargo, brew, and other developer caches
- **TUI mode** — Interactive terminal UI with 18 scan modes, keyboard navigation, and live progress
- **Git protection** — 4 protection levels to prevent deleting uncommitted work
- **Parallel scanning** — Uses rayon + walkdir for fast multi-threaded filesystem traversal
- **Safe by default** — Trash-based deletion (recoverable), dry-run mode, artifact safety classification
- **Configurable** — TOML config file with scan settings, clean settings, UI preferences, and plugin controls

## Supported Languages & Tools

| Plugin | Detected by | Artifacts |
|--------|------------|-----------|
| Node.js | `package.json` | `node_modules`, `.next`, `.nuxt`, `dist`, `build`, `.cache`, `.turbo`, `coverage`, `.parcel-cache`, `.svelte-kit` |
| Rust | `Cargo.toml` | `target/` |
| Python | `uv.lock`, `poetry.lock`, `pyproject.toml`, `Pipfile`, `requirements.txt` | `.venv`, `__pycache__`, `.pytest_cache`, `.mypy_cache`, `.ruff_cache`, `.tox`, `*.egg-info` |
| Go | `go.mod` | `vendor`, `bin`, `dist` |
| Maven | `pom.xml` | `target/` |
| Gradle | `build.gradle` | `build/`, `.gradle/`, `out/` |
| .NET | `*.csproj`, `*.sln` | `bin/`, `obj/`, `packages/`, `TestResults/` |
| Swift | `Package.swift`, `*.xcodeproj` | `.build/`, `Pods/`, `DerivedData/` |

## System Cleaners

| Category | Targets |
|----------|---------|
| Xcode | DerivedData, Archives, Simulators |
| Docker | Images, containers, volumes, build cache |
| Android | AVD, SDK, Gradle caches |
| ML/AI | HuggingFace, Ollama, PyTorch models |
| IDE | JetBrains, VS Code, Cursor caches |
| Homebrew | Download caches |
| Electron | Slack, Discord, Teams app caches |
| Cloud CLI | AWS, GCP, Azure, kubectl, Terraform |
| Game Dev | Unity, Unreal, Godot |
| Logs | System logs, crash reports |
| iOS Deps | CocoaPods, Carthage, SPM caches |
| Runtimes | nvm, pyenv, rbenv, rustup, sdkman |
| Browsers | Playwright, Cypress, Puppeteer, Selenium |
| System | Trash, Downloads, Temp files |
| macOS | System caches (macOS only) |
