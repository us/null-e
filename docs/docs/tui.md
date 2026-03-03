# TUI Mode

null-e includes an interactive terminal UI built with ratatui and crossterm.

## Launch

```bash
null-e tui
null-e ui          # alias
null-e interactive # alias
```

## Scan Modes

The TUI offers 18 scan modes, selectable from the main screen:

| Mode | What it scans |
|------|--------------|
| SCAN ALL | Everything below |
| Dev Projects | node_modules, target, venv, .gradle, vendor |
| Global Caches | npm, pip, cargo, brew caches |
| Xcode | DerivedData, Archives, Simulators |
| Docker | Images, Containers, Volumes |
| IDE Caches | JetBrains, VS Code, Cursor |
| ML/AI Models | HuggingFace, Ollama, PyTorch |
| Android | AVD, SDK, Gradle caches |
| Electron Apps | Slack, Discord, Teams |
| Cloud CLI | AWS, GCP, Azure, kubectl, Terraform |
| Package Managers | Homebrew, apt, chocolatey |
| Game Dev | Unity, Unreal, Godot |
| Misc Tools | Vagrant, Go, Ruby, Git LFS, Maven |
| Test Browsers | Playwright, Cypress, Puppeteer |
| System | Trash, Downloads, Temp files |
| Logs | System logs, crash reports |
| Language Runtimes | nvm, pyenv, rbenv, rustup, sdkman, gvm |
| Binary Analysis | Duplicate, conflict, unused managers |

## Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `g` | Go to top |
| `G` | Go to bottom |
| `Ctrl+U` | Page up |
| `Ctrl+D` | Page down |
| `Tab` / `Shift+Tab` | Switch tab |
| `/` | Search |
| `b` / `Backspace` | Go back |

### Selection & Deletion

| Key | Action |
|-----|--------|
| `Enter` | Toggle selection |
| `Space` | Expand/collapse |
| `a` | Select all |
| `A` / `u` | Deselect all |
| `d` / `Delete` | Delete selected items |
| `p` | Toggle permanent delete mode |
| `y` | Confirm deletion |
| `n` | Cancel deletion |

### Other

| Key | Action |
|-----|--------|
| `r` / `F5` | Refresh / rescan |
| `?` | Show help |
| `q` / `Ctrl+C` | Quit |
| Mouse scroll | Scroll 3 lines |

## App Flow

```
Ready → (select scan mode) → Scanning → Results
                                          ↓
                                   (press d) → Confirming → (press y) → Cleaning
                                                  ↓
                                           (press n) → Results
```

During scanning, a live progress indicator shows directories scanned, artifacts found, and estimated reclaimable space. You can cancel a scan with `q` or `Ctrl+C`.
