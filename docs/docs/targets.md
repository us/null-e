# Targets & Plugins

null-e uses a plugin system to detect and clean project-specific artifacts. Each plugin identifies a project type and its cleanable directories.

## Project Plugins

### Node.js

**ID:** `node` | **Priority:** 50 | **Detected by:** `package.json`

Automatically identifies the package manager from lockfiles: npm (`package-lock.json`), yarn (`yarn.lock`), pnpm (`pnpm-lock.yaml`), bun (`bun.lockb`).

| Artifact | Description |
|----------|-------------|
| `node_modules` | Dependencies |
| `.next` | Next.js build output |
| `.nuxt` | Nuxt build output |
| `dist` | Build output |
| `build` | Build output |
| `.cache` | Various caches |
| `.parcel-cache` | Parcel bundler cache |
| `.turbo` | Turborepo cache |
| `coverage` | Test coverage |
| `.nyc_output` | NYC coverage |
| `storybook-static` | Storybook build |
| `.svelte-kit` | SvelteKit build |
| `out` | Static export output |

### Rust

**ID:** `rust` | **Priority:** 60 | **Detected by:** `Cargo.toml`

| Artifact | Description |
|----------|-------------|
| `target/` | Build output, debug + release artifacts |

Restore command: `cargo build`

### Python

**ID:** `python` | **Priority:** 50 | **Detected by** (in priority order): `uv.lock`, `poetry.lock`, `pyproject.toml`, `Pipfile`, `environment.yml`, `requirements.txt`

Identifies package manager: pip, poetry, pipenv, conda, uv.

| Artifact | Description |
|----------|-------------|
| `.venv` / `venv` / `env` / `.env` | Virtual environments (verified via `pyvenv.cfg`) |
| `__pycache__` | Bytecode cache |
| `.pytest_cache` | Pytest cache |
| `.mypy_cache` | Mypy type checker cache |
| `.ruff_cache` | Ruff linter cache |
| `.tox` | Tox test environments |
| `.nox` | Nox test environments |
| `*.egg-info` | Package metadata |
| `dist` | Built packages |
| `build` | Build artifacts |
| `htmlcov` | Coverage HTML reports |
| `.coverage` | Coverage data |

### Go

**ID:** `go` | **Priority:** 55 | **Detected by:** `go.mod`

| Artifact | Description |
|----------|-------------|
| `vendor` | Vendored dependencies |
| `bin` | Compiled binaries |
| `dist` | Distribution output |

### Java — Maven

**ID:** `maven` | **Priority:** 60 | **Detected by:** `pom.xml`

| Artifact | Description |
|----------|-------------|
| `target/` | Build output |

### Java — Gradle

**ID:** `gradle` | **Priority:** 60 | **Detected by:** `build.gradle`, `build.gradle.kts`, `settings.gradle`

| Artifact | Description |
|----------|-------------|
| `build/` | Build output |
| `.gradle/` | Gradle cache |
| `out/` | IntelliJ build output |

### .NET

**ID:** `dotnet` | **Priority:** 55 | **Detected by:** `*.csproj`, `*.fsproj`, `*.vbproj`, `*.sln`

| Artifact | Description |
|----------|-------------|
| `bin/` | Compiled output |
| `obj/` | Intermediate build files |
| `packages/` | NuGet packages |
| `TestResults/` | Test results |

### Swift

**ID:** `swift` | **Priority:** 55 | **Detected by:** `Package.swift` (SPM) or `*.xcodeproj` / `*.xcworkspace`

| Artifact | Description |
|----------|-------------|
| `.build/` | SPM build directory |
| `.swiftpm/` | SPM metadata |
| `Pods/` | CocoaPods dependencies |
| `DerivedData/` | Xcode build data |
| `build/` | Build output |

## Artifact Safety Classification

Each artifact type has a safety level that determines how it's handled:

| Level | Types | Behavior |
|-------|-------|----------|
| **AlwaysSafe** | Cache, Logs, Temporary, Bytecode | Always safe to delete |
| **SafeIfGitClean** | BuildOutput, TestOutput, DocsBuild | Safe if git has no uncommitted changes |
| **SafeWithLockfile** | Dependencies, PackageManagerCache | Safe if a lockfile exists (can be restored) |
| **RequiresConfirmation** | VirtualEnv, IdeArtifacts, Docker, Custom | Always asks for confirmation |
| **NeverAuto** | LockFile | Never automatically deleted |

## Scanner Algorithm

The parallel scanner works as follows:

1. Each root path is scanned in parallel using rayon
2. `walkdir` traverses the directory tree, skipping known artifact dirs (node_modules, target, .venv, etc.) to avoid descending into them
3. At each directory, all plugins are checked via `detect_project()` — the highest-priority match wins
4. Detected artifacts are sized in parallel via `rayon::par_iter_mut()`
5. Results are filtered by `--min-size` and sorted by reclaimable size (descending)
6. The scanner uses `DashMap` for concurrent insert and atomic counters for live progress

Hidden directories (starting with `.`) are skipped, except for `.git`, `.github`, `.vscode`, and `.idea`.
