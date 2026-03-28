# Installation

## Desktop App (GUI)

Download the latest version from [GitHub Releases](https://github.com/us/null-e/releases/latest).

| Platform | File | Notes |
|----------|------|-------|
| **macOS (Apple Silicon)** | `null-e_x.x.x_aarch64.dmg` | M1, M2, M3, M4 |
| **macOS (Intel)** | `null-e_x.x.x_x64.dmg` | Intel Macs |
| **Windows** | `null-e_x.x.x_x64-setup.exe` | 64-bit Windows 10/11 |
| **Linux (deb)** | `null-e_x.x.x_amd64.deb` | Ubuntu, Debian |
| **Linux (AppImage)** | `null-e_x.x.x_amd64.AppImage` | Any Linux distro |

### macOS Setup

The app is currently unsigned. macOS will block it by default.

1. Download the `.dmg` for your chip
2. Open the DMG and drag `null-e.app` to `/Applications`
3. Open Terminal and run:

```bash
xattr -rd com.apple.quarantine /Applications/null-e.app
```

4. Open null-e from Applications or Spotlight

> **Why?** macOS Gatekeeper blocks unsigned apps. The `xattr` command removes the quarantine flag. This is safe — verify the source on [GitHub](https://github.com/us/null-e).

### Windows Setup

1. Download and run the `.exe` installer
2. If SmartScreen warns you, click **"More info" → "Run anyway"**
3. Launch from Start Menu

### Linux Setup

```bash
# Ubuntu/Debian
sudo dpkg -i null-e_x.x.x_amd64.deb

# AppImage (any distro)
chmod +x null-e_x.x.x_amd64.AppImage
./null-e_x.x.x_amd64.AppImage
```

## CLI (command-line)

```bash
# From crates.io
cargo install null-e

# From Homebrew
brew tap us/tap
brew install null-e

# From source
git clone https://github.com/us/null-e.git
cd null-e
cargo install --path crates/null-e-cli
```

## Auto-Updates

The desktop app checks for updates on launch. A notification bar shows when a new version is available — click **"Update & Restart"** to install.

Manual check: **Settings → About → Check for updates**

## Verify

```bash
null-e --version
null-e --help
```
