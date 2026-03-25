# Detection Reference

Complete list of everything null-e can detect and clean. All paths are relative to `~/` unless noted otherwise.

> **Want to add something?** Open an issue or PR on [GitHub](https://github.com/nicholasgasior/null-e) — the detection lists are in `src/caches/mod.rs`, `src/cleaners/`, and `src/plugins/`.

---

## Project Artifacts (Dynamic Scan)

These are found by walking the directory tree and matching marker files. This is the only dynamic detection — everything else below is static path lists.

### Node.js
**Markers:** `package.json`, `yarn.lock`, `pnpm-lock.yaml`, `bun.lockb`

| Artifact | Type |
|----------|------|
| `node_modules` | Dependencies |
| `.next` | Next.js build |
| `.nuxt` | Nuxt build |
| `dist`, `build`, `out` | Build output |
| `.cache`, `.parcel-cache` | Bundler cache |
| `.turbo` | Turborepo cache |
| `coverage`, `.nyc_output` | Test coverage |
| `storybook-static` | Storybook build |
| `.svelte-kit` | SvelteKit build |

### Rust
**Marker:** `Cargo.toml`

| Artifact | Type |
|----------|------|
| `target/` | Build output (debug + release) |

### Python
**Markers:** `pyproject.toml`, `Pipfile`, `requirements.txt`, `setup.py`, `environment.yml`, `uv.lock`

| Artifact | Type |
|----------|------|
| `.venv`, `venv`, `env` | Virtual environments |
| `__pycache__` | Bytecode cache |
| `.pytest_cache`, `.mypy_cache`, `.ruff_cache` | Tool caches |
| `.tox`, `.nox` | Test environments |
| `dist`, `build`, `*.egg-info` | Build output |
| `htmlcov`, `.coverage` | Coverage data |

### Go
**Marker:** `go.mod`

| Artifact | Type |
|----------|------|
| `vendor` | Vendored deps |
| `bin`, `dist` | Build output |

### Java — Maven
**Marker:** `pom.xml`

| Artifact | Type |
|----------|------|
| `target/` | Build output |

### Java — Gradle
**Markers:** `build.gradle`, `build.gradle.kts`, `settings.gradle`

| Artifact | Type |
|----------|------|
| `build/` | Build output |
| `.gradle/` | Gradle cache |
| `out/` | IntelliJ build output |

### .NET
**Markers:** `*.csproj`, `*.fsproj`, `*.vbproj`, `*.sln`

| Artifact | Type |
|----------|------|
| `bin/`, `obj/` | Build output |
| `packages/` | NuGet packages |
| `TestResults/` | Test results |

### Swift / Xcode
**Markers:** `Package.swift`, `*.xcodeproj`, `*.xcworkspace`

| Artifact | Type |
|----------|------|
| `.build/` | SPM build dir |
| `.swiftpm/` | SPM metadata |
| `Pods/` | CocoaPods deps |
| `DerivedData/` | Xcode build data |
| `build/` | Build output |

---

## Global Caches (Static Paths)

Detected via `null-e caches`. All paths relative to `~/`.

### JavaScript / Node.js

| Cache | Paths | Clean Command |
|-------|-------|---------------|
| npm | `.npm/_cacache` | `npm cache clean --force` |
| Yarn | `.yarn/cache`, `.cache/yarn` | `yarn cache clean` |
| pnpm | `.pnpm-store`, `.local/share/pnpm/store`, `Library/pnpm/store` | `pnpm store prune` |
| Bun | `.bun/install/cache` | — |
| Deno | `.cache/deno`, `.deno` | `deno cache --reload` |
| node-gyp | `Library/Caches/node-gyp`, `.cache/node-gyp`, `.node-gyp` | — |

### Python

| Cache | Paths | Clean Command |
|-------|-------|---------------|
| pip | `.cache/pip`, `Library/Caches/pip` | `pip cache purge` |
| uv | `.cache/uv`, `Library/Caches/uv` | `uv cache clean` |
| Poetry | `.cache/pypoetry`, `Library/Caches/pypoetry` | `poetry cache clear --all .` |
| Pipenv | `.cache/pipenv` | — |
| Conda | `.conda/pkgs`, `anaconda3/pkgs`, `miniconda3/pkgs` | `conda clean --all` |

### Rust

| Cache | Paths | Clean Command |
|-------|-------|---------------|
| Cargo registry | `.cargo/registry` | — (auto GC in 1.75+) |
| Cargo git | `.cargo/git` | — |

### Go

| Cache | Paths | Clean Command |
|-------|-------|---------------|
| Go modules | `go/pkg/mod` | `go clean -modcache` |
| Go build | `.cache/go-build`, `Library/Caches/go-build` | `go clean -cache` |

### JVM

| Cache | Paths | Clean Command |
|-------|-------|---------------|
| Gradle | `.gradle/caches` | — |
| Maven | `.m2/repository` | — |
| SBT | `.sbt`, `.ivy2/cache` | — |

### Other

| Cache | Paths | Clean Command |
|-------|-------|---------------|
| NuGet | `.nuget/packages` | `dotnet nuget locals all --clear` |
| Gem | `.gem`, `.local/share/gem` | `gem cleanup` |
| Bundler | `.bundle/cache` | `bundle clean --force` |
| Composer | `.composer/cache`, `.cache/composer` | `composer clear-cache` |
| CocoaPods | `Library/Caches/CocoaPods` | `pod cache clean --all` |
| Dart/Flutter | `.pub-cache` | — |
| Android | `.android/cache`, `.android/build-cache` | — |
| Hugging Face | `.cache/huggingface` | — |
| PyTorch | `.cache/torch` | — |
| Homebrew | `Library/Caches/Homebrew` | `brew cleanup --prune=all` |
| Cypress | `.cache/Cypress`, `Library/Caches/Cypress` | `cypress cache clear` |
| Playwright | `.cache/ms-playwright`, `Library/Caches/ms-playwright` | — |
| Electron | `.cache/electron`, `Library/Caches/electron` | — |

---

## Electron / Chromium Apps

Detected via `null-e sweep` and `null-e electron`. Scans `~/Library/Application Support/{app}/` for cache subdirectories (`Cache`, `CachedData`, `GPUCache`, `Code Cache`, `Service Worker`, `blob_storage`, `DesktopProfile`, `PersistentCache`) and `~/Library/Caches/{bundle_id}`.

### Detected Apps (41)

| App | Folder | Icon |
|-----|--------|------|
| Slack | `Slack` | 💬 |
| Discord | `discord` | 🎮 |
| Spotify | `Spotify` | 🎵 |
| Microsoft Teams | `Microsoft Teams` | 👥 |
| Notion | `Notion` | 📝 |
| Figma | `Figma` | 🎨 |
| Obsidian | `obsidian` | 💎 |
| Postman | `Postman` | 📮 |
| Insomnia | `Insomnia` | 🌙 |
| Hyper | `Hyper` | ⚡ |
| GitKraken | `GitKraken` | 🐙 |
| Atom | `Atom` | ⚛️ |
| Signal | `Signal` | 🔒 |
| WhatsApp | `WhatsApp` | 📱 |
| Telegram Desktop | `Telegram Desktop` | ✈️ |
| Linear | `Linear` | 📊 |
| Loom | `Loom` | 🎥 |
| Cron | `Cron` | 📅 |
| Raycast | `com.raycast.macos` | 🔍 |
| 1Password | `1Password` | 🔐 |
| Bitwarden | `Bitwarden` | 🔐 |
| Franz | `Franz` | 📬 |
| Station | `Station` | 🚉 |
| Skype | `Skype` | 📞 |
| Zoom | `zoom.us` | 📹 |
| Webex | `Cisco Webex Meetings` | 🌐 |
| Miro | `Miro` | 🖼️ |
| ClickUp | `ClickUp` | ✅ |
| Todoist | `Todoist` | ☑️ |
| Trello | `Trello` | 📋 |
| Stremio | `com.stremio.stremio-shell-macos` | 📺 |
| Antigravity | `Antigravity` | 🚀 |
| Anytype | `anytype` | 📐 |
| Chrome | `Google/Chrome` | 🌐 |
| Brave | `BraveSoftware/Brave-Browser` | 🦁 |
| Arc | `Arc` | 🌈 |
| Edge | `Microsoft Edge` | 🔵 |
| Opera | `com.operasoftware.Opera` | 🔴 |
| Vivaldi | `Vivaldi` | 🎹 |

### Custom Cache Bundle IDs

Some apps don't follow the `com.{name}.desktop` pattern:

| App | Bundle ID |
|-----|-----------|
| Spotify | `com.spotify.client` |
| Discord | `com.hnc.Discord` |
| Figma | `com.figma.Desktop` |
| Chrome | `com.google.Chrome` |
| Brave | `com.brave.Browser` |
| Arc | `company.thebrowser.Browser` |
| Edge | `com.microsoft.edgemac` |
| Stremio | `com.westbridge.stremio5-mac` |

---

## IDE Caches

Detected via `null-e ide`.

### JetBrains

**Products:** IntelliJ IDEA, PyCharm, WebStorm, PhpStorm, CLion, GoLand, Rider, RubyMine, DataGrip, Android Studio, Fleet

| Platform | Cache Path | Data Path | Log Path |
|----------|-----------|-----------|----------|
| macOS | `Library/Caches/JetBrains/` | `Library/Application Support/JetBrains/` | `Library/Logs/JetBrains/` |
| Linux | `.cache/JetBrains/` | `.config/JetBrains/` | — |
| Windows | `AppData/Local/JetBrains/` | `AppData/Roaming/JetBrains/` | — |

### VS Code

| Platform | Paths |
|----------|-------|
| macOS | `Library/Application Support/Code/{CachedData,CachedExtensions,CachedExtensionVSIXs,Cache,User/workspaceStorage}`, `Library/Caches/com.microsoft.VSCode` |
| Linux | `.config/Code/{CachedData,CachedExtensions,Cache,User/workspaceStorage}` |
| Windows | `AppData/Roaming/Code/{CachedData,CachedExtensions,Cache,User/workspaceStorage}` |

### Cursor

| Platform | Paths |
|----------|-------|
| macOS | `Library/Application Support/Cursor/{CachedData,Cache,User/workspaceStorage}`, `Library/Caches/com.todesktop.230313mzl4w4u92` |

### Zed

| Platform | Paths |
|----------|-------|
| macOS | `Library/Caches/dev.zed.Zed`, `Library/Application Support/Zed/{languages,extensions,copilot,node}` |

### Sublime Text

| Platform | Paths |
|----------|-------|
| macOS | `Library/Application Support/Sublime Text/{Cache,Index}`, `Library/Caches/com.sublimetext.4` |
| Linux | `.config/sublime-text/{Cache,Index}` |
| Windows | `AppData/Roaming/Sublime Text/{Cache,Index}` |

---

## Xcode (macOS)

Detected via `null-e xcode`.

| Path | Description |
|------|-------------|
| `Library/Developer/Xcode/DerivedData/` | Per-project build data |
| `Library/Developer/Xcode/Archives/` | App archives (.xcarchive) |
| `Library/Developer/Xcode/iOS DeviceSupport/` | Debug symbols per iOS version |
| `Library/Developer/Xcode/watchOS DeviceSupport/` | watchOS debug symbols |
| `Library/Developer/Xcode/tvOS DeviceSupport/` | tvOS debug symbols |
| `Library/Developer/CoreSimulator/Devices/` | Simulator runtimes |
| `Library/Caches/com.apple.dt.Xcode` | Xcode cache |
| `Library/Caches/com.apple.dt.instruments` | Instruments cache |
| `Library/Caches/org.swift.swiftpm` | SPM cache |

---

## iOS Dependencies

Detected via `null-e ios-deps`.

| Tool | Paths | Clean Command |
|------|-------|---------------|
| CocoaPods | `Library/Caches/CocoaPods/`, `.cocoapods/repos/` | `pod cache clean --all` |
| Carthage | `Library/Caches/org.carthage.CarthageKit/` | — |
| SPM | `Library/Caches/org.swift.swiftpm/`, `Library/org.swift.swiftpm/` | `swift package purge-cache` |

---

## Docker

Detected via `null-e docker`. Uses `docker` CLI commands (requires Docker to be running).

| What | Docker Command |
|------|----------------|
| Disk usage | `docker system df` |
| Dangling images | `docker images -f dangling=true` |
| Stopped containers | `docker ps -a -f status=exited` |
| Dangling volumes | `docker volume ls -f dangling=true` |
| Build cache | `docker builder du` |

---

## ML / AI Models

Detected via `null-e ml`.

| Tool | Paths | Min Size |
|------|-------|----------|
| Hugging Face Hub | `.cache/huggingface/hub/` | 10 MB/model |
| Hugging Face Datasets | `.cache/huggingface/datasets/` | — |
| Hugging Face Transformers | `.cache/huggingface/transformers/` | — |
| Ollama | `.ollama/models/blobs/`, `.ollama/models/manifests/` | 100 MB/model |
| PyTorch | `.cache/torch/`, `.cache/torch/hub/` | 10 MB |
| Keras | `.keras/models/` | 10 MB |
| TensorFlow | `.tensorflow/`, `.cache/tensorflow/` | 10 MB |
| Jupyter | `.cache/jupyter/`, `.jupyter/`, `.local/share/jupyter/` | 10 MB |
| LM Studio | `.lmstudio/models/`, `.cache/lm-studio/` | — |
| GPT4All | `.cache/gpt4all/`, `Library/Application Support/nomic.ai/GPT4All/` | 100 MB |

---

## Cloud CLI Tools

Detected via `null-e cloud`.

| Tool | Paths | Clean Command |
|------|-------|---------------|
| AWS CLI | `.aws/cli/cache/`, `.aws/sso/cache/`, `.aws/boto/cache/` | — |
| AWS SAM | `.aws-sam/cache/` | — |
| Google Cloud | `.config/gcloud/{logs,cache}/` | `gcloud components cleanup --unused` |
| Azure | `.azure/{logs,cliextensions,commands}/` | `az cache purge` |
| kubectl | `.kube/cache/`, `.kube/http-cache/` | — |
| Minikube | `.minikube/cache/` | `minikube delete --purge` |
| Kind | `.kind/` | — |
| Terraform | `.terraform.d/plugin-cache/` | — |
| Pulumi | `.pulumi/plugins/` | `pulumi plugin rm --all` |
| Helm | `.cache/helm/`, `Library/Caches/helm/` | — |

---

## Language Runtime Version Managers

Detected via `null-e sweep`. Identifies inactive versions that can be removed.

| Manager | Versions Path | Active Detection |
|---------|--------------|-----------------|
| nvm | `~/.nvm/versions/node/` | `~/.nvm/alias/default` |
| fnm | `Library/Application Support/fnm/node-versions/` | — |
| volta | `.volta/tools/image/node/` | `.volta/tools/user/platform.json` |
| n | `/usr/local/n/versions/node/` | — |
| pyenv | `~/.pyenv/versions/` | `~/.pyenv/version` |
| conda | `~/anaconda3/envs/`, `~/miniconda3/envs/`, `~/miniforge3/envs/` | — |
| rbenv | `~/.rbenv/versions/` | `~/.rbenv/version` |
| rvm | `~/.rvm/rubies/` | `~/.rvm/config/default` |
| sdkman | `~/.sdkman/candidates/java/` | `current` symlink |
| rustup | `~/.rustup/toolchains/` | `rustup default` |
| gvm | `~/.gvm/gos/` | `~/.gvm/environments/default` |
| Go SDK | `~/sdk/go*/` | `go version` |

---

## Game Development

Detected via `null-e gamedev`.

### Unity
| Path (macOS) | Description |
|------|-------------|
| `Library/Application Support/Unity/` | Global data |
| `Library/Caches/com.unity3d.UnityEditor` | Editor cache |
| `Library/Unity/Asset Store-5.x` | Downloaded assets |
| `Library/Unity/cache` | Cache |
| `Library/Application Support/UnityHub/` | Unity Hub |
| Project: `Library/`, `Temp/`, `Logs/`, `Builds/`, `obj/` | Per-project |

### Unreal Engine
| Path (macOS) | Description |
|------|-------------|
| `Library/Application Support/Epic/` | Epic Games |
| `Library/Application Support/Unreal Engine/Common/DerivedDataCache/` | DDC |
| Project: `Intermediate/`, `Saved/`, `DerivedDataCache/`, `Binaries/` | Per-project |

### Godot
| Path (macOS) | Description |
|------|-------------|
| `Library/Application Support/Godot/` | Editor data |
| `Library/Caches/Godot/` | Cache |

---

## Android Development

Detected via `null-e android`.

| What | Path |
|------|------|
| AVD Emulators | `.android/avd/*.avd/` |
| System Images | `Library/Android/sdk/system-images/` |
| Gradle Caches | `.gradle/{caches,wrapper,daemon,native}/` |
| Build Cache | `.android/{cache,build-cache}/` |
| Android Studio Cache | `Library/Caches/Google.AndroidStudio*` |

---

## System Cleanup

Detected via `null-e sweep` (system category).

| What | Path (macOS) | Min Size |
|------|-------------|----------|
| Trash | `~/.Trash/` | 1 MB |
| Old Downloads | `~/Downloads/*.{zip,dmg,pkg,...}` | 100 MB, 30+ days old |
| Temp Files | `Library/Caches/TemporaryItems/` | 500 MB |
| System Caches | `Library/Caches/` | 1 GB |
| Font Cache | `Library/Caches/com.apple.FontRegistry` | 50 MB |

---

## Logs

Detected via `null-e sweep`.

| What | Path | Min Size |
|------|------|----------|
| User Logs | `Library/Logs/` | 10 MB |
| Homebrew Logs | `Library/Logs/Homebrew/` | 5 MB |
| Crash Reports | `Library/Logs/DiagnosticReports/` | 5 MB |
| npm Logs | `.npm/_logs/` | 1 MB |
| Yarn Logs | `.yarn/logs/` | — |
| Gradle Daemon | `.gradle/daemon/` | — |

---

## Homebrew

Detected via `null-e homebrew`.

| What | Path | Clean Command |
|------|------|---------------|
| Downloads | `Library/Caches/Homebrew/downloads/` | `brew cleanup` |
| Cask Cache | `Library/Caches/Homebrew/Cask/` | `brew cleanup --cask` |
| Old Formula Versions | `/opt/homebrew/Cellar/` | `brew cleanup -s --prune=all` |

---

## Testing Browsers

Detected via `null-e sweep`.

| Tool | Paths | Clean Command |
|------|-------|---------------|
| Playwright | `.cache/ms-playwright`, `Library/Caches/ms-playwright` | — |
| Cypress | `.cache/Cypress`, `Library/Caches/Cypress` | `cypress cache clear` |
| Puppeteer | `.cache/puppeteer` | — |
| Selenium | `.cache/selenium` | — |

---

## Adding New Detection Targets

Want null-e to detect something it currently misses? Here's where to add it:

| What to add | File to edit |
|-------------|-------------|
| New global cache (e.g., `~/.cache/newcache`) | `src/caches/mod.rs` → add `CacheDefinition` |
| New Electron/Chromium app | `src/cleaners/electron.rs` → add to `ELECTRON_APPS` |
| Non-standard cache bundle ID | `src/cleaners/electron.rs` → add to `EXTRA_CACHE_BUNDLE_IDS` |
| New IDE | `src/cleaners/ide.rs` → add `detect_newide()` method |
| New language/project type | `src/plugins/` → implement `Plugin` trait |
| New system cleaner category | `src/cleaners/` → create new module |

Each `CacheDefinition` in `src/caches/mod.rs` looks like:

```rust
CacheDefinition {
    id: "mycache",
    name: "My Cache",
    icon: "📦",
    paths: &["relative/to/home", "Library/Caches/mycache"],
    clean_command: Some("mycache clean"),
    description: "Description of this cache",
},
```
