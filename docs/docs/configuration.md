# Configuration

null-e uses a TOML configuration file for persistent settings.

## Setup

```bash
# Generate a sample config file
null-e config --init

# Show the config file path
null-e config --path
```

Config location: `~/.config/devsweep/config.toml`

## Full Reference

```toml
[general]
default_paths = ["~/projects", "~/code"]   # Default scan directories
exclude_paths = []                          # Always exclude these paths
log_level = "info"                          # error, warn, info, debug, trace
verbose = false

[scan]
max_depth = null                  # Scan depth (null = unlimited)
skip_hidden = true                # Skip hidden directories
respect_gitignore = true          # Honor .gitignore files
min_size = null                   # Minimum artifact size in bytes
ignore_patterns = []              # Glob patterns to ignore
parallelism = null                # Thread count (null = CPU count)
check_git_status = true           # Check git status before deletion

[clean]
delete_method = "trash"           # trash, permanent, dry-run
protection_level = "warn"         # none, warn, block, paranoid
continue_on_error = true          # Continue if a single deletion fails
auto_confirm = false              # Skip confirmation prompts
dry_run = false                   # Global dry-run mode

[ui]
theme = "auto"                    # dark, light, auto
show_file_counts = true           # Show number of files in artifacts
show_dates = true                 # Show last modified dates
sort_by = "size"                  # size, name, date, kind
sort_reverse = false              # Reverse sort order
use_icons = true                  # Show icons in output

[plugins]
enabled = []                      # Empty = all plugins enabled
disabled = []                     # Plugin IDs to disable
```

## Config Sections

### [general]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `default_paths` | string[] | `[]` | Directories to scan when no path is given |
| `exclude_paths` | string[] | `[]` | Paths to always exclude from scanning |
| `log_level` | string | `"info"` | Log verbosity |
| `verbose` | bool | `false` | Enable verbose output |

### [scan]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `max_depth` | int/null | `null` | Maximum directory traversal depth |
| `skip_hidden` | bool | `true` | Skip directories starting with `.` |
| `respect_gitignore` | bool | `true` | Honor `.gitignore` rules |
| `min_size` | int/null | `null` | Minimum artifact size in bytes |
| `ignore_patterns` | string[] | `[]` | Glob patterns to skip |
| `parallelism` | int/null | `null` | Worker threads (null = CPU count) |
| `check_git_status` | bool | `true` | Check git before deleting |

### [clean]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `delete_method` | string | `"trash"` | `trash`, `permanent`, or `dry-run` |
| `protection_level` | string | `"warn"` | `none`, `warn`, `block`, `paranoid` |
| `continue_on_error` | bool | `true` | Don't stop on individual failures |
| `auto_confirm` | bool | `false` | Skip all confirmation prompts |
| `dry_run` | bool | `false` | Never actually delete |

### [ui]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `theme` | string | `"auto"` | TUI theme: `dark`, `light`, `auto` |
| `show_file_counts` | bool | `true` | Display file counts in results |
| `show_dates` | bool | `true` | Display last modified dates |
| `sort_by` | string | `"size"` | Sort results by: `size`, `name`, `date`, `kind` |
| `sort_reverse` | bool | `false` | Reverse sort order |
| `use_icons` | bool | `true` | Use icons in terminal output |

### [plugins]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | string[] | `[]` | Plugin IDs to enable (empty = all) |
| `disabled` | string[] | `[]` | Plugin IDs to disable |

Available plugin IDs: `node`, `rust`, `python`, `go`, `maven`, `gradle`, `dotnet`, `swift`

## Environment Variables

Standard Rust logging is supported:

```bash
RUST_LOG=debug null-e scan ~/projects
```

## CLI vs Config

CLI flags override config file values. For example:

```bash
# Config says "warn", but this overrides to "block"
null-e clean ~/projects --protection block
```
