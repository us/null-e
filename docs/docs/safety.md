# Git Protection

null-e includes a multi-level git protection system to prevent accidentally deleting uncommitted work.

## Protection Levels

Set via `--protection` / `-p` flag or config file:

| Level | Flag | Behavior |
|-------|------|----------|
| **none** | `-p none` | No protection. Deletes everything without checking git status. Use with caution. |
| **warn** | `-p warn` | Default. Warns about uncommitted changes but proceeds with deletion. |
| **block** | `-p block` | Blocks deletion if the project has uncommitted changes (staged or modified files). |
| **paranoid** | `-p paranoid` | Blocks if no git repo exists. Also blocks if files were modified within the last 7 days. |

## What Gets Checked

For each project, null-e inspects the git status using `gix` (a pure-Rust git implementation):

| Check | `none` | `warn` | `block` | `paranoid` |
|-------|--------|--------|---------|-----------|
| Staged/modified files | — | warn | block | block |
| Untracked files | — | warn | warn | block |
| No git repo | — | warn | warn | block |
| Modified in last 7 days | — | warn | warn | block |

## Delete Methods

| Method | Flag | Behavior |
|--------|------|----------|
| **Trash** | `-m trash` | Default. Moves to system trash via the `trash` crate. Recoverable. |
| **Permanent** | `-m permanent` | `rm -rf` equivalent. Not recoverable. |
| **Dry Run** | `-m dry-run` / `-n` | Calculates sizes, prints what would be deleted, but deletes nothing. |

## Recommended Workflow

```bash
# 1. Always start with a dry run
null-e clean ~/projects --dry-run

# 2. Review the output, then clean with trash (default)
null-e clean ~/projects

# 3. For maximum safety on important codebases
null-e clean ~/projects --protection block

# 4. For aggressive cleanup of old/unused projects
null-e clean ~/projects --protection none --method permanent
```

## TUI Safety

In TUI mode:
- Items are selected individually before deletion
- Press `d` to start deletion, `y` to confirm
- Press `p` to toggle between trash and permanent delete
- Permanent mode is clearly indicated in the UI
- `std::panic::catch_unwind` wraps the delete operation to prevent crashes
