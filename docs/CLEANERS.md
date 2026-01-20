# Cleaners Guide

null-e includes specialized cleaners for various development environments. Each cleaner knows exactly what to look for and how to safely clean it.

## Overview

| Cleaner | Command | What It Cleans | Typical Size |
|---------|---------|----------------|--------------|
| [Xcode](#xcode) | `null-e xcode` | DerivedData, Simulators, Archives | 20-100 GB |
| [Android](#android) | `null-e android` | AVD, Gradle, SDK | 5-30 GB |
| [Docker](#docker) | `null-e docker` | Images, Containers, Volumes | 10-100 GB |
| [ML/AI](#mlai) | `null-e ml` | Huggingface, Ollama, PyTorch | 10-100 GB |
| [IDE](#ide) | `null-e ide` | JetBrains, VS Code, Cursor | 2-20 GB |
| [Homebrew](#homebrew) | `null-e homebrew` | Downloads, Old versions | 2-20 GB |
| [iOS Deps](#ios-dependencies) | `null-e ios-deps` | CocoaPods, Carthage, SPM | 1-10 GB |
| [Electron](#electron-apps) | `null-e electron` | Slack, Discord, Spotify | 1-5 GB |
| [Game Dev](#game-development) | `null-e gamedev` | Unity, Unreal, Godot | 10-100 GB |
| [Cloud CLI](#cloud-cli) | `null-e cloud` | AWS, GCP, Azure, kubectl | 1-5 GB |
| [macOS](#macos-system) | `null-e macos` | Orphaned containers, Caches | 1-20 GB |

---

## Xcode

**Command:** `null-e xcode`

Cleans Xcode development artifacts on macOS.

### What It Detects

| Item | Location | Safety | Description |
|------|----------|--------|-------------|
| **DerivedData** | `~/Library/Developer/Xcode/DerivedData` | Safe | Build caches, can be huge |
| **Archives** | `~/Library/Developer/Xcode/Archives` | Caution | App archives for distribution |
| **iOS Simulators** | `~/Library/Developer/CoreSimulator` | SafeWithCost | Simulator runtime data |
| **Device Support** | `~/Library/Developer/Xcode/iOS DeviceSupport` | SafeWithCost | Debug symbols for devices |
| **Previews** | `~/Library/Developer/Xcode/UserData/Previews` | Safe | SwiftUI preview cache |
| **watchOS Support** | `~/Library/Developer/Xcode/watchOS DeviceSupport` | SafeWithCost | Watch debug symbols |

### Usage

```bash
# Show Xcode artifacts
null-e xcode

# Clean interactively
null-e xcode --clean

# With verbose output
null-e xcode -v
```

### Example Output

```
🔨 Xcode Cleanup v0.1.0

✓ Found 39 items with 57.59 GiB total

   By Category:
   📁 DerivedData         25.00 GiB  (3 items)
   📲 Simulators          30.00 GiB  (35 items)
   📦 Device Support       2.59 GiB  (1 items)
```

---

## Android

**Command:** `null-e android`

Cleans Android Studio and SDK artifacts.

### What It Detects

| Item | Location | Safety | Description |
|------|----------|--------|-------------|
| **Gradle Caches** | `~/.gradle/caches` | SafeWithCost | Build dependencies |
| **Gradle Wrapper** | `~/.gradle/wrapper` | SafeWithCost | Gradle distributions |
| **AVD System Images** | `~/.android/avd` | Caution | Emulator images |
| **SDK Components** | `~/Library/Android/sdk` | Caution | Old SDK versions |
| **Build Cache** | `~/.android/build-cache` | Safe | Build cache files |

### Usage

```bash
# Show Android artifacts
null-e android

# Clean with confirmation
null-e android --clean
```

---

## Docker

**Command:** `null-e docker`

Cleans Docker resources (requires Docker to be running).

### What It Detects

| Item | Safety | Description |
|------|--------|-------------|
| **Dangling Images** | Safe | Untagged images |
| **Unused Images** | SafeWithCost | Images not used by containers |
| **Stopped Containers** | Safe | Exited containers |
| **Build Cache** | SafeWithCost | Docker build layer cache |
| **Unused Volumes** | Caution | Orphaned data volumes |
| **Unused Networks** | Safe | Networks not in use |

### Usage

```bash
# Show Docker resources
null-e docker

# Clean (excluding volumes)
null-e docker --clean

# Clean including volumes (careful with data!)
null-e docker --clean --volumes
```

### Notes

- null-e uses the official `docker system prune` commands
- Volumes are excluded by default to protect your data
- Use `--volumes` flag only if you're sure you don't need the data

---

## ML/AI

**Command:** `null-e ml`

Cleans machine learning and AI model caches.

### What It Detects

| Item | Location | Safety | Description |
|------|----------|--------|-------------|
| **Huggingface Hub** | `~/.cache/huggingface` | SafeWithCost | Downloaded models and datasets |
| **Ollama Models** | `~/.ollama/models` | SafeWithCost | Locally pulled LLM models |
| **PyTorch Hub** | `~/.cache/torch` | SafeWithCost | Downloaded model weights |
| **Transformers** | `~/.cache/huggingface/transformers` | SafeWithCost | Transformer model cache |
| **Datasets** | `~/.cache/huggingface/datasets` | Caution | Downloaded datasets |

### Usage

```bash
# Show ML caches
null-e ml

# Clean selected models
null-e ml --clean
```

### Tip

ML model caches can easily exceed 100 GB if you work with large language models. Running `null-e ml` regularly can help reclaim significant disk space.

---

## IDE

**Command:** `null-e ide`

Cleans IDE caches and temporary files.

### What It Detects

| IDE | Items | Safety |
|-----|-------|--------|
| **JetBrains** | Caches, Logs, Local History | SafeWithCost |
| **VS Code** | CachedData, CachedExtensions, Logs | Safe |
| **Cursor** | Same as VS Code | Safe |

### Locations

**JetBrains IDEs** (IntelliJ, WebStorm, PyCharm, etc.):
- `~/Library/Caches/JetBrains`
- `~/Library/Application Support/JetBrains`
- `~/Library/Logs/JetBrains`

**VS Code / Cursor**:
- `~/Library/Application Support/Code/CachedData`
- `~/Library/Application Support/Code/CachedExtensionVSIXs`
- `~/Library/Caches/com.microsoft.VSCode`

### Usage

```bash
# Show IDE caches
null-e ide

# Clean caches
null-e ide --clean
```

---

## Homebrew

**Command:** `null-e homebrew`

Cleans Homebrew package manager caches and old versions.

### What It Detects

| Item | Location | Safety | Description |
|------|----------|--------|-------------|
| **Downloads** | `~/Library/Caches/Homebrew/downloads` | Safe | Downloaded packages |
| **Cask Downloads** | `~/Library/Caches/Homebrew/Cask` | Safe | Downloaded app installers |
| **Old Versions** | `/usr/local/Cellar` or `/opt/homebrew/Cellar` | SafeWithCost | Previous formula versions |

### Usage

```bash
# Show Homebrew caches
null-e homebrew

# Clean using brew cleanup
null-e homebrew --clean

# Scrub (remove even latest downloads)
null-e homebrew --clean --scrub
```

### Notes

null-e uses the official `brew cleanup` command to ensure safe cleanup. The `--scrub` flag corresponds to `brew cleanup --scrub`.

---

## iOS Dependencies

**Command:** `null-e ios-deps`

Cleans iOS dependency manager caches.

### What It Detects

| Manager | Location | Safety | Description |
|---------|----------|--------|-------------|
| **CocoaPods** | `~/Library/Caches/CocoaPods` | SafeWithCost | Pod spec and source cache |
| **Carthage** | `~/Library/Caches/org.carthage.CarthageKit` | SafeWithCost | Built frameworks cache |
| **Swift PM** | `~/Library/Caches/org.swift.swiftpm` | SafeWithCost | Package resolution cache |

### Usage

```bash
# Show iOS dependency caches
null-e ios-deps

# Clean caches
null-e ios-deps --clean
```

### Official Commands

- CocoaPods: `pod cache clean --all`
- Carthage: `rm -rf ~/Library/Caches/org.carthage.CarthageKit`
- Swift PM: Managed by Xcode

---

## Electron Apps

**Command:** `null-e electron`

Cleans caches from Electron-based applications.

### Supported Apps

null-e detects caches from 30+ Electron apps:

| Category | Apps |
|----------|------|
| **Communication** | Slack, Discord, Microsoft Teams, Skype, Zoom |
| **Development** | VS Code, Cursor, GitHub Desktop, Postman |
| **Media** | Spotify, Notion |
| **Utilities** | 1Password, Bitwarden, Figma |

### What It Cleans

For each app, null-e can clean:
- `Cache/` - HTTP and asset cache
- `CachedData/` - Compiled code cache
- `GPUCache/` - GPU shader cache
- `Code Cache/` - V8 code cache
- `Service Worker/` - Service worker data

### Usage

```bash
# Show Electron app caches
null-e electron

# Clean selected apps
null-e electron --clean
```

---

## Game Development

**Command:** `null-e gamedev`

Cleans game engine caches and build artifacts.

### Unity

| Item | Location | Safety | Description |
|------|----------|--------|-------------|
| **Editor Cache** | `~/Library/Caches/com.unity3d.UnityEditor` | Safe | Editor cache files |
| **Asset Store** | `~/Library/Unity/Asset Store-5.x` | SafeWithCost | Downloaded assets |
| **Global Cache** | `~/Library/Unity/cache` | SafeWithCost | Shader and artifact cache |
| **Logs** | `~/Library/Logs/Unity` | Safe | Unity log files |
| **Hub Cache** | `~/Library/Application Support/UnityHub` | Safe | Unity Hub installer cache |

**Per-Project** (detected during scan):
- `Library/` - Project cache
- `Temp/` - Temporary files
- `Logs/` - Project logs
- `Builds/` - Build output

### Unreal Engine

| Item | Location | Safety | Description |
|------|----------|--------|-------------|
| **DerivedDataCache** | Shared DDC location | SafeWithCost | Compiled shaders and assets |
| **Epic Cache** | `~/Library/Application Support/Epic` | SafeWithCost | Engine and marketplace cache |

**Per-Project**:
- `Intermediate/` - Build intermediates
- `Saved/` - Saved data (check before deleting)
- `DerivedDataCache/` - Local DDC
- `Binaries/` - Compiled output

### Godot

| Item | Location | Safety |
|------|----------|--------|
| **Editor Data** | `~/Library/Application Support/Godot` | SafeWithCost |
| **Cache** | `~/Library/Caches/Godot` | Safe |

### Usage

```bash
# Show game dev caches
null-e gamedev

# Clean selected items
null-e gamedev --clean
```

---

## Cloud CLI

**Command:** `null-e cloud`

Cleans cloud provider CLI caches.

### AWS

| Item | Location | Safety | Description |
|------|----------|--------|-------------|
| **CLI Cache** | `~/.aws/cli/cache` | Safe | API response cache |
| **SSO Cache** | `~/.aws/sso/cache` | Safe | SSO token cache |
| **SAM Cache** | `~/.aws-sam/cache` | SafeWithCost | SAM build cache |

### Google Cloud

| Item | Location | Safety | Description |
|------|----------|--------|-------------|
| **Logs** | `~/.config/gcloud/logs` | Safe | Command logs |
| **Cache** | `~/.config/gcloud/cache` | Safe | API cache |
| **ADC Cache** | `~/.config/gcloud/application_default_credentials_cache` | Safe | Credential cache |

### Azure

| Item | Location | Safety | Description |
|------|----------|--------|-------------|
| **Logs** | `~/.azure/logs` | Safe | Command logs |
| **Commands Cache** | `~/.azure/commands` | Safe | Command cache |
| **Extensions** | `~/.azure/cliextensions` | SafeWithCost | CLI extensions |

### Kubernetes

| Item | Location | Safety | Description |
|------|----------|--------|-------------|
| **kubectl Cache** | `~/.kube/cache` | Safe | API discovery cache |
| **HTTP Cache** | `~/.kube/http-cache` | Safe | HTTP response cache |
| **Minikube Cache** | `~/.minikube/cache` | SafeWithCost | ISO and preload images |
| **Kind Cache** | `~/.kind` | SafeWithCost | Kind cluster images |

### Terraform

| Item | Location | Safety | Description |
|------|----------|--------|-------------|
| **Plugin Cache** | `~/.terraform.d/plugin-cache` | SafeWithCost | Provider plugins |

### Pulumi

| Item | Location | Safety | Description |
|------|----------|--------|-------------|
| **Plugins** | `~/.pulumi/plugins` | SafeWithCost | Provider plugins |

### Helm

| Item | Location | Safety | Description |
|------|----------|--------|-------------|
| **Cache** | `~/.cache/helm` | SafeWithCost | Chart cache |

### Usage

```bash
# Show cloud CLI caches
null-e cloud

# Clean selected caches
null-e cloud --clean
```

---

## macOS System

**Command:** `null-e macos` (macOS only)

Cleans macOS-specific developer artifacts.

### What It Detects

| Item | Location | Safety | Description |
|------|----------|--------|-------------|
| **Orphaned Containers** | `~/Library/Containers` | Caution | Sandbox data for uninstalled apps |
| **Large Library Caches** | `~/Library/Caches` | SafeWithCost | App caches over 100 MB |
| **App Support Remnants** | `~/Library/Application Support` | Caution | Data from uninstalled apps |
| **Font Cache** | Font cache locations | Safe | Font rendering cache |

### How Orphan Detection Works

null-e compares:
1. Containers/App Support folders
2. Installed apps in `/Applications` and `~/Applications`
3. Bundle identifiers from app Info.plist files

If a container exists but no matching app is found, it's flagged as potentially orphaned.

### Usage

```bash
# Show macOS system items
null-e macos

# Clean selected items (be careful with Caution items!)
null-e macos --clean

# Verbose output shows more details
null-e macos -v
```

### Caution

Items marked with `!` (Caution) require manual verification. null-e cannot guarantee that an app is truly uninstalled - it might be:
- Installed in a non-standard location
- A helper app for another application
- Required by the system

Always verify before deleting items marked with Caution.

---

## Sweep Command

**Command:** `null-e sweep`

Runs all cleaners at once for a comprehensive system scan.

### Usage

```bash
# Full system sweep
null-e sweep

# Filter by category
null-e sweep --category xcode
null-e sweep --category docker

# Clean interactively
null-e sweep --clean
```

### Available Categories

- `xcode`
- `android`
- `docker`
- `ml`
- `ide`
- `homebrew` / `brew`
- `ios-deps` / `ios`
- `electron`
- `gamedev` / `game`
- `cloud`
- `macos` / `system`

### Example Output

```
🧹 null-e Deep Scan v0.1.0

✓ Found 84 items with 97.92 GiB total

   By Category:
   🔨 Xcode                   57.59 GiB  (39 items)
   🐳 Docker                  32.59 GiB  (22 items)
   🗄️ macOS System             3.43 GiB  (9 items)
   🥥 iOS Dependencies         1.64 GiB  (2 items)
   💻 IDE                      1.06 GiB  (4 items)
   🐘 Android                945.78 MiB  (2 items)
   🍺 Package Manager        511.08 MiB  (5 items)
   🎮 Electron Apps          191.72 MiB  (1 items)
```
