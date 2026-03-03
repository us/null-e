# Commands

## Global Flags

These flags work with all subcommands:

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `[PATHS]` | — | CWD | Directories to scan |
| `--max-depth` | `-d` | unlimited | Maximum scan depth |
| `--min-size` | `-s` | 1MB | Minimum artifact size (e.g., `100KB`, `1MB`, `1GB`) |
| `--top` | `-t` | 25 | Show top N largest projects (0 = all) |
| `--protection` | `-p` | `warn` | Git protection level: `none`, `warn`, `block`, `paranoid` |
| `--method` | `-m` | `trash` | Delete method: `trash`, `permanent`, `dry-run` |
| `--force` | `-f` | false | Skip confirmation prompts |
| `--dry-run` | `-n` | false | Show what would be deleted without deleting |
| `--verbose` | `-v` | false | Verbose output |
| `--output` | — | `pretty` | Output format: `pretty`, `json`, `compact` |
| `--all` | `-a` | false | Show all projects (no limit) |
| `--no-cache` | — | false | Skip scan cache, force full rescan |

## Subcommands

### null-e (no subcommand)

Scans the current directory. Equivalent to `null-e scan .`.

### null-e scan

Scan for build artifacts and dependencies.

```bash
null-e scan ~/projects
null-e scan ~/projects --detailed
null-e scan ~/projects --min-size 500MB --top 10
```

### null-e clean

Delete discovered artifacts.

```bash
null-e clean ~/projects
null-e clean ~/projects --only node,rust,python
null-e clean ~/projects --exclude go
null-e clean ~/projects --method permanent --force
```

### null-e tui

Launch interactive TUI mode. See [TUI Mode](#tui) for details.

```bash
null-e tui
null-e ui          # alias
null-e interactive # alias
```

### null-e caches

Manage global developer caches (npm, pip, cargo, brew, etc.).

```bash
null-e caches              # List caches with sizes
null-e caches --clean      # Interactive cache cleanup
null-e caches --clean-all  # Clean all caches
null-e caches --official   # Use official package manager cleanup commands
```

### null-e sweep

Deep system scan across multiple categories.

```bash
null-e sweep                         # Scan all categories
null-e sweep --clean                 # Clean all categories
null-e sweep --category docker       # Scan specific category
null-e sweep --category xcode --clean
```

### null-e config

Manage configuration.

```bash
null-e config --init   # Generate sample config file
null-e config --path   # Show config file path
```

### null-e list

List all supported project types and their detected artifacts.

### Category-specific commands

Each category has its own subcommand:

| Command | Category |
|---------|----------|
| `null-e xcode` | Xcode DerivedData, Archives, Simulators |
| `null-e android` | Android AVD, SDK, Gradle |
| `null-e docker` | Docker images, volumes, build cache |
| `null-e ml` | ML/AI caches (HuggingFace, Ollama, PyTorch) |
| `null-e ide` | IDE caches (JetBrains, VS Code, Cursor) |
| `null-e homebrew` | Homebrew caches |
| `null-e ios-deps` | CocoaPods, Carthage, SPM |
| `null-e electron` | Electron app caches (Slack, Discord, Teams) |
| `null-e gamedev` | Unity, Unreal, Godot |
| `null-e cloud` | AWS, GCP, Azure, kubectl, Terraform |
| `null-e macos` | macOS system caches (macOS only) |

All category commands support `--clean` to perform cleanup.

### null-e docker

Docker has additional flags:

```bash
null-e docker --clean           # Clean dangling images + stopped containers
null-e docker --clean --volumes # Also clean unused volumes
```

### null-e homebrew

```bash
null-e homebrew --clean   # Clean download caches
null-e homebrew --scrub   # Deep clean
```

### Analysis commands

```bash
null-e git-analyze              # Analyze git repositories
null-e git-analyze --fix        # Run git gc on repos

null-e stale                    # Find stale projects
null-e stale --days 180         # Projects not modified in 180 days
null-e stale --clean            # Clean stale projects

null-e duplicates               # Find duplicate dependencies
```
