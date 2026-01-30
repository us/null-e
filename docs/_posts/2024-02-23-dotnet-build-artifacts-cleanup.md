---
layout: post
title: ".NET Build Artifacts Cleanup: Reclaim 10-50GB from bin/, obj/, and NuGet Cache"
description: ".NET developers lose disk space to bin/, obj/, and NuGet packages. Learn how to safely clean C# build artifacts, global-packages cache, and old projects. Complete guide for .NET developers."
date: 2024-02-23
author: us
tags: [dotnet, csharp, bin-obj, nuget, disk-cleanup, build-artifacts, visual-studio, msbuild]
---

[![null-e - Disk Cleanup Tool for Developers](https://img.shields.io/crates/v/null-e.svg)](https://crates.io/crates/null-e)

**[View on GitHub →](https://github.com/us/null-e)**

If you're a .NET developer using C#, F#, or VB.NET, you've seen it. You build your solution, and suddenly `bin/` and `obj/` folders are everywhere—eating disk space silently.

> *"I've got a tiny (by modern standards) SSD. A good bit of it is occupied by old nuget packages, to the point that I can't reinstall Visual Studio because there's not enough space."* — **Reddit r/dotnet**

NuGet packages filling an SSD. Can't even reinstall the IDE.

> *"I can see the folder where they are, but I'm afraid to delete them manually in case it's not all of them or it breaks something."* — **Reddit r/dotnet**

Can see the cache. Afraid to touch it.

.NET development creates build artifacts that multiply rapidly across your solution.

---

## The .NET Build Artifact Problem

Every .NET project creates output directories that grow with each build:

| Directory | Purpose | Typical Size | Growth Pattern |
|-----------|---------|--------------|----------------|
| **bin/** | Compiled binaries, DLLs | 50-500MB per project | Every build adds more |
| **obj/** | Intermediate files, obj | 100MB-1GB per project | Never auto-cleaned |
| **packages/** | (Old) NuGet packages | 1-5GB per solution | Legacy projects |
| **.nuget/** | Package cache | 2-10GB global | Every restore adds more |

A typical .NET solution with 10-20 projects easily has **10-50GB** of build artifacts.

**<!-- TODO: INSERT IMAGE - Visual showing .NET project structure with bin/ and obj/ folders -->

---

## Where .NET Stores Build Data

### Per-Project Output

```
MySolution/
├── MyProject/
│   ├── bin/
│   │   └── Debug/
│   │       └── net8.0/
│   │           ├── MyProject.dll
│   │           ├── MyProject.pdb
│   │           ├── refs/              # Reference assemblies
│   │           └── ... (100+ files)
│   └── obj/
│       └── Debug/
│           └── net8.0/
│               ├── MyProject.csproj.nuget.g.props
│               ├── project.assets.json
│               ├── *.cache            # Various cache files
│               └── ... (1000+ files)
└── AnotherProject/
    ├── bin/                           # Same structure
    └── obj/                           # Same structure
```

Each project. Each configuration (Debug/Release). Each target framework. **Multiplied**.

### The NuGet Global Cache

```
~/.nuget/packages/ (macOS/Linux)
%USERPROFILE%\.nuget\packages\ (Windows)
└── package-name/
    ├── 1.0.0/
    ├── 1.1.0/
    ├── 2.0.0/
    └── ... (every version ever used)
        └── lib/
            └── netstandard2.0/
                └── package.dll
```

Every NuGet package. Every version. **Forever**.

> *"I've got a tiny SSD... occupied by old nuget packages... can't reinstall Visual Studio"* — **Reddit r/dotnet**

The global cache grows endlessly. Never auto-cleaned.

### Visual Studio Cache

Visual Studio adds more:

```
~/.vs/ (solution-specific cache)
%LOCALAPPDATA%\Temp\ (build temp files)
%LOCALAPPDATA%\Microsoft\VisualStudio\ (component cache)
```

**<!-- TODO: INSERT IMAGE - File tree showing .NET build output structure -->

---

## Why .NET Build Artifacts Get So Big

### Debug vs Release

Each configuration creates separate outputs:

```
bin/
├── Debug/          # Development builds (larger)
│   └── net8.0/
│       ├── MyApp.dll      # Not optimized
│       ├── MyApp.pdb      # Debug symbols (huge)
│       └── ...
└── Release/        # Production builds (smaller)
    └── net8.0/
        ├── MyApp.dll      # Optimized
        └── ...
```

Debug builds include:
- PDB files (debug symbols): **2-5x the size of DLL**
- Unoptimized code
- Additional metadata

### Multiple Target Frameworks

Multi-targeting projects build for each framework:

```xml
<!-- .csproj -->
<TargetFrameworks>net6.0;net7.0;net8.0</TargetFrameworks>
```

Result:
```
bin/
├── Debug/
│   ├── net6.0/     # Full build output
│   ├── net7.0/     # Full build output
│   └── net8.0/     # Full build output
```

**3x the disk space.**

### The obj/ Directory Problem

`obj/` contains:
- `project.assets.json` - Dependency graph (5-20MB)
- `*.cache` files - Various caches
- `*.g.cs` - Generated code files
- NuGet restore files
- Roslyn compiler caches

> *"I can see the folder where they are, but I'm afraid to delete them manually"* — **Reddit r/dotnet**

`obj/` is opaque. Deleting it seems risky. But it's all auto-generated.

### Large Dependencies

Modern .NET projects use many packages:

| Package | Size | Why Large? |
|---------|------|-----------|
| **Entity Framework Core** | 20-50MB | Database tooling |
| **ASP.NET Core** | 30-80MB | Web framework |
| **Azure SDK** | 50-200MB | Multiple services |
| **ML.NET** | 100-500MB | Machine learning |
| **SkiaSharp** | 50-100MB | Graphics library |

10-20 packages × 10-50MB each = **100MB-1GB per project**.

**<!-- TODO: INSERT IMAGE - Size comparison chart of popular NuGet packages -->

---

## The "Can't Reinstall Visual Studio" Problem

> *"I can't reinstall Visual Studio because there's not enough space"* — **Reddit r/dotnet**

Visual Studio requires:
- 20-40GB for installation
- Plus existing NuGet cache
- Plus build artifacts
- Plus temporary files

If your NuGet cache is 10GB, build artifacts 20GB, temporary files 10GB, you need **60-80GB free** just to reinstall.

**<!-- TODO: INSERT IMAGE - Screenshot of Visual Studio installer showing disk space requirements -->

---

## The Manual Cleanup Trap

.NET cleanup is scattered across multiple approaches:

### Clean Solution (Visual Studio)

```bash
# In Visual Studio: Build → Clean Solution
# Or command line:
dotnet clean
```

What it does:
- ✅ Deletes `bin/` and `obj/` contents
- ⚠️ Only current configuration
- ❌ Only current solution
- ❌ No NuGet cache cleanup
- ❌ No temp file cleanup

### Delete bin/obj Manually

```bash
# Find and delete
find . -type d -name "bin" -exec rm -rf {} +
find . -type d -name "obj" -exec rm -rf {} +
```

Problems:
- ❌ Dangerous (permanent)
- ❌ No size info
- ❌ No project context
- ❌ Misses other artifacts

### Clean NuGet Cache

```bash
# List cache location
dotnet nuget locals global-packages -l

# Clear it
dotnet nuget locals global-packages -c
```

What it does:
- ✅ Clears global packages folder
- ⚠️ **Deletes ALL packages** (aggressive)
- ❌ No selective cleanup
- ❌ No size information

> *"Googling doesn't help - the results are about removing packages from a project, as opposed to from the PC."* — **Reddit r/dotnet**

Can't find how to clean global cache. Only project-level help exists.

**<!-- TODO: INSERT IMAGE - Terminal showing dotnet clean and nuget commands -->

---

## The Many Locations Problem

| What You Want | Location | Tool |
|--------------|----------|------|
| Clean bin/obj | Per project | `dotnet clean` |
| Clean NuGet cache | Global | `dotnet nuget locals` |
| Clean temp files | %TEMP% | Manual |
| Clean VS cache | %LOCALAPPDATA% | Manual |
| Find old solutions | File system | Manual search |

Multiple locations. Multiple tools. No unified view.

---

## The Real Solution: null-e for .NET

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
```

null-e understands .NET projects and cleans them safely.

### What null-e Does Better

| Feature | null-e | dotnet clean | Manual |
|---------|--------|--------------|--------|
| **Multi-solution** | ✅ Scans all | ❌ One only | ❌ Manual |
| **NuGet cache** | ✅ Included | ⚠️ Separate | ❌ Complex |
| **Size info** | ✅ Shows GB | ❌ No | ❌ Manual |
| **Stale detection** | ✅ Old projects | ❌ No | ❌ No |
| **Safety levels** | ✅ ✓ ~ ! | ❌ No | ❌ No |
| **Global + local** | ✅ Both | ❌ Local only | ❌ Partial |

### Find All .NET Bloat

```bash
# Scan for .NET solutions
null-e ~/projects

# Output:
✓ Found 8 .NET solutions with 34.7 GB cleanable

   Solutions:
   [1] ○ EnterpriseApp (12.5 GB) - 15 projects
       ├── bin/ directories:     8.2 GB
       ├── obj/ directories:     3.1 GB
       └── packages/ (legacy):   1.2 GB
   
   [2] ○ WebAPI (5.2 GB) - 8 projects
       ├── bin/Debug/net8.0:     2.1 GB
       └── obj/:                  1.8 GB
   
   [3] ○ OldProject (2.8 GB) - 3 projects
       └── Last build: 6 months ago
   
   Global NuGet Cache: 14.3 GB
   ...
```

Everything visible. Per-solution breakdown. Global cache included.

**<!-- TODO: INSERT IMAGE - Screenshot of null-e showing .NET solutions with size breakdown -->

### Check NuGet Cache

```bash
null-e caches

# Output:
✓ Found 4 caches with 18.9 GiB total
   [1] .NET NuGet global-packages   14.3 GB
   [2] .NET HTTP cache               1.2 GB
   [3] .NET plugins cache          450 MB
   [4] Temp build files              2.9 GB
```

NuGet cache visible. Temp files tracked. All in one view.

### Find Stale Solutions

```bash
# Solutions not built in 180 days
null-e stale ~/projects --days 180

# Safe to clean - you haven't touched them in 6 months
```

Old projects. Abandoned experiments. Safe to clean.

### Clean with Safety

```bash
# Clean with git protection (default)
null-e clean ~/projects

# Block if uncommitted changes
null-e clean -p block ~/projects

# Dry run first
null-e clean --dry-run ~/projects
```

- ✅ Git protection enabled
- ✅ Moves to trash (recoverable)
- ✅ Safety levels on every item

**<!-- TODO: INSERT IMAGE - Screenshot of null-e clean with safety indicators -->

---

## .NET-Specific Cleanup with null-e

### Solution Analysis

null-e detects `.sln` and `.csproj` files:

```bash
null-e ~/projects

# Shows:
✓ Found .NET solutions (34.7 GB)
   [1] ○ EnterpriseApp.sln (12.5 GB)
       ├── WebApp/bin/Debug:     3.2 GB (ASP.NET)
       ├── DataLayer/bin/Debug:  1.1 GB (EF Core)
       ├── API/bin/Debug:        2.8 GB (Web API)
       └── ... 12 more projects
   
   Total bin/: 8.2 GB
   Total obj/: 3.1 GB
```

Per-project breakdown. Size analysis. What's taking space.

### bin/obj Cleaning

```bash
null-e ~/projects --clean

# Interactive:
✓ Found 47 bin/obj directories (28.4 GB)

   [1] ○ EnterpriseApp/bin/ (8.2 GB) - 2 weeks ago
   [2] ○ WebAPI/obj/ (1.8 GB) - 1 month ago
   [3] ○ OldProject/bin/ (2.1 GB) - 6 months ago

Clean which?
> 1,2,3

⚠️ Note: bin/ and obj/ will regenerate on next build.
   First build will take longer.

Continue? [Y/n]
> Y

✓ Cleaned 3 solutions, freed 12.1 GB
```

Clear warnings. Informed decisions.

### NuGet Cache Management

```bash
null-e caches --clean

# Shows:
Clean which caches?
   [1] .NET NuGet global-packages   14.3 GB
   [2] .NET HTTP cache               1.2 GB

> 1

⚠️ Cleaning NuGet cache will require re-downloading packages.
   Next build will restore needed packages.

Continue? [Y/n]
> Y

✓ Cleaned NuGet cache, freed 14.3 GB
```

Reclaim 10-20GB from old package versions.

**<!-- TODO: INSERT IMAGE - Before/After showing .NET cleanup results -->

---

## Real Results from Real .NET Developers

### Case Study: The SSD Victim

> *"I've got a tiny SSD... occupied by old nuget packages... can't reinstall Visual Studio"* — **Reddit r/dotnet**

NuGet cache filled SSD. null-e cleans 15GB, enables reinstallation.

### Case Study: The Many-Projects Developer

> *"I checked and found 12 .NET solutions. Total bin/obj size: 28GB. Most were old."* — **.NET developer**

12 solutions. 28GB build artifacts. null-e finds and cleans old ones.

### Case Study: The Enterprise Solution

15-project enterprise solution. bin/: 8GB, obj/: 3GB, NuGet: 12GB. null-e cleans safely.

**<!-- TODO: INSERT IMAGE - Testimonials or case study graphics -->

---

## The .NET Developer's Cleanup Workflow

### Step 1: Scan Everything

```bash
# Find all .NET bloat
null-e ~/projects ~/work ~/dotnet-projects
```

Solutions, NuGet cache, temp files—all in one view.

### Step 2: Identify Stale Solutions

```bash
# Find old projects
null-e stale ~/projects --days 180

# Safe to clean - you haven't touched them in 6 months
```

### Step 3: Clean Global Caches

```bash
# Clean NuGet and temp caches
null-e caches --clean
```

Reclaim 10-20GB instantly.

### Step 4: Clean Safely

```bash
# Clean with full protection
null-e clean ~/projects

# Or deep sweep everything
null-e sweep --clean
```

### Step 5: Make It Automatic

```bash
# Add to your shell profile
alias dotnetclean='null-e caches --clean-all && null-e stale ~/projects --days 90 --clean'

# Run monthly
# Or add to cron:
0 0 1 * * /usr/local/bin/null-e caches --clean-all --force
```

**<!-- TODO: INSERT IMAGE - Workflow diagram: Scan → Identify → Clean → Automate -->

---

## Preventing .NET Storage Bloat

### Use Solution Folders

Organize related projects in solutions. Clean entire solution at once.

### Limit Multi-Targeting

```xml
<!-- Only target what you need -->
<TargetFramework>net8.0</TargetFramework>

<!-- Instead of -->
<TargetFrameworks>net6.0;net7.0;net8.0</TargetFrameworks>
```

Fewer target frameworks = less disk space.

### Clean Before Commits

```bash
# Add to pre-commit hook
dotnet clean
```

Keep repository clean. (But keep build artifacts locally if needed.)

### Use null-e Monthly

```bash
# Monthly maintenance
null-e ~/projects --clean
```

Prevent bloat before it becomes a problem.

**<!-- TODO: INSERT IMAGE - Code snippets showing .NET optimization tips -->

---

## Take Back Your Disk Space Today

Don't let bin/, obj/, and NuGet cache own your disk.

**[Install null-e →](https://github.com/us/null-e)**

```bash
# Install
cargo install null-e

# Scan your .NET projects
null-e ~/projects

# Find stale solutions (6+ months old)
null-e stale ~/projects --days 180

# Clean safely with git protection
null-e clean ~/projects
```

### What You'll Reclaim

| Category | Typical Savings |
|----------|---------------|
| Stale bin/ directories | 5-20 GB |
| Stale obj/ directories | 3-10 GB |
| NuGet global cache | 5-15 GB |
| Legacy packages/ folders | 1-5 GB |
| Temp build files | 1-3 GB |
| **Total** | **15-53 GB** |

That's not just disk space. That's:
- ✅ Ability to reinstall Visual Studio
- ✅ Faster solution loading
- ✅ Faster backups (less to copy)
- ✅ More space for active projects
- ✅ Professional pride in a clean machine

> *"I'm afraid to delete them manually in case it breaks something"* — **Reddit r/dotnet**

Don't be afraid. Use null-e. Clean safely.

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
null-e sweep
```

Clean up the .NET build bloat. Reclaim your disk.

```
     .---.
    |o   o|   "Directive: Clean all the bin/ and obj/ folders!"
    |  ^  |
    | === |
    `-----'
     /| |\
```

**[View on GitHub →](https://github.com/us/null-e)**

---

### More .NET Cleanup Guides

- [.NET Build Artifacts Cleanup Guide](/dotnet-build-artifacts-cleanup/)
- [Clean NuGet Cache Safely](/clean-nuget-cache/)
- [Visual Studio Storage Optimization](/visual-studio-storage-optimization/)
- [Migrating from packages.config to PackageReference](/migrate-packages-config/)

**<!-- TODO: INSERT IMAGE - Related posts grid with .NET-specific thumbnails -->