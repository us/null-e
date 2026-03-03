# Installation

## From crates.io

```bash
cargo install null-e
```

## From source

```bash
git clone https://github.com/us/null-e.git
cd null-e
cargo install --path .
```

The release profile uses LTO, single codegen unit, and symbol stripping for a small binary.

## Pre-built binaries

Download from [GitHub Releases](https://github.com/us/null-e/releases):

| Platform | File |
|----------|------|
| macOS ARM | `null-e-darwin-aarch64.tar.gz` |
| macOS Intel | `null-e-darwin-x86_64.tar.gz` |
| Linux x86_64 | `null-e-linux-x86_64.tar.gz` |
| Linux ARM | `null-e-linux-aarch64.tar.gz` |
| Windows | `null-e-windows-x86_64.zip` |

## Package managers

```bash
# Homebrew
brew install null-e

# AUR (Arch Linux)
yay -S null-e

# Scoop (Windows)
scoop bucket add us https://github.com/us/scoop-bucket
scoop install null-e
```

## Docker

```bash
docker run -v $(pwd):/workspace ghcr.io/us/null-e
```

## Verify

```bash
null-e --version
null-e --help
```
