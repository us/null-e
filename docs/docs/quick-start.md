# Quick Start

## Scan

See what's taking up space:

```bash
# Scan current directory
null-e

# Scan specific directories
null-e scan ~/projects ~/code

# Show only large artifacts (> 500MB)
null-e scan ~/projects --min-size 500MB

# Show top 10 largest projects
null-e scan ~/projects --top 10

# JSON output
null-e scan ~/projects --output json
```

## Clean

```bash
# Dry run — see what would be deleted
null-e clean ~/projects --dry-run

# Clean with trash (default, recoverable)
null-e clean ~/projects

# Clean specific types only
null-e clean ~/projects --only node,rust

# Skip specific types
null-e clean ~/projects --exclude python

# Permanent delete (not recoverable)
null-e clean ~/projects --method permanent

# Skip confirmation prompt
null-e clean ~/projects --force
```

## TUI mode

Launch the interactive terminal UI:

```bash
null-e tui
```

Use arrow keys to navigate, Enter to select, `d` to delete selected items, `p` to toggle permanent delete.

## Global caches

Manage developer caches (npm, pip, cargo, etc.):

```bash
# List all caches with sizes
null-e caches

# Clean specific caches
null-e caches --clean

# Clean all caches
null-e caches --clean-all

# Use official package manager cleanup
null-e caches --official
```

## System cleaners

Clean up system-level artifacts:

```bash
# Deep scan: Xcode + Docker + ML + IDE + logs
null-e sweep

# Clean a specific category
null-e sweep --category docker --clean
null-e sweep --category xcode --clean
null-e sweep --category ml --clean

# Individual commands
null-e docker          # Show Docker usage
null-e docker --clean  # Clean Docker resources
null-e xcode           # Show Xcode artifacts
null-e xcode --clean   # Clean Xcode artifacts
```
