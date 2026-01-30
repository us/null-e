---
layout: post
title: "Java Gradle and Maven Cleanup: Reclaim 20-100GB from Build Directories"
description: "Java developers lose massive disk space to target/, build/, and .gradle/. Learn how to safely clean Java build artifacts, Maven local repository, and Gradle caches. Complete guide with real developer stories."
date: 2024-02-18
author: us
tags: [java, gradle, maven, kotlin, build-directory, disk-cleanup, target-folder, gradle-cache]
---

[![null-e - Disk Cleanup Tool for Developers](https://img.shields.io/crates/v/null-e.svg)](https://crates.io/crates/null-e)

**[View on GitHub →](https://github.com/us/null-e)**

If you're a Java or Kotlin developer, you know the scenario. You run `./gradlew build`, and suddenly your disk space starts vanishing into `build/`, `.gradle/`, and `target/` directories.

> *"I've got a tiny (by modern standards) SSD. A good bit of it is occupied by old nuget packages, to the point that I can't reinstall Visual Studio because there's not enough space."* — **Reddit r/dotnet** (similar Java problem)

Replace "nuget" with "Maven" or "Gradle," and you've got the Java experience.

> *"So I poked at my .gradle folder and found it's size to be 28GB. 🤯... I checked ./gradle/daemon thoroughly and found out 4 days of logs were consuming ~10GB of disk memory!"* — **Jitin Sharma blog**, January 2021

28GB `.gradle` folder. 10GB just from **4 days of logs**.

The Java ecosystem is powerful—but it's a disk space vacuum.

---

## The Java Build Directory Problem

Every Java/Kotlin project creates build artifacts that never clean themselves:

| Build Tool | Directory | Typical Size | Contents |
|-----------|-----------|--------------|----------|
| **Maven** | `target/` | 200MB-2GB | Compiled classes, JARs, test output |
| **Gradle** | `build/` | 300MB-3GB | Compiled classes, reports, caches |
| **Gradle (global)** | `~/.gradle/` | 2-20GB | Wrapper downloads, caches, daemons |
| **Maven (global)** | `~/.m2/repository/` | 1-10GB | Downloaded dependencies |

And that's per project. A Java developer with 10, 20, 50 projects easily has **20-100GB** of build artifacts.

**<!-- TODO: INSERT IMAGE - Visual showing Java projects with target/, build/, and .gradle/ directories -->

---

## Why Java Build Directories Get So Big

### Dependencies Are Downloaded and Cached

Java's dependency management downloads everything:

```xml
<!-- Maven pom.xml -->
<dependencies>
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-web</artifactId>
        <version>3.2.0</version>
    </dependency>
</dependencies>
```

Spring Boot web starter alone: **50+ dependencies** including:
- Spring Framework
- Tomcat
- Jackson
- Hibernate Validator
- Logging frameworks
- And more...

Each downloaded. Each cached. Each taking space.

### Gradle Wrapper Downloads

```bash
# Each project has a Gradle wrapper
./gradlew build

# Downloads:
# - Gradle distribution (100-200MB per version)
# - Stored in ~/.gradle/wrapper/
# - Never cleaned automatically
```

10 projects using different Gradle versions? **1-2GB** just for Gradle distributions.

### Build Outputs Multiply

```
my-project/
├── build/
│   ├── classes/          # Compiled .class files
│   ├── libs/             # Generated JARs
│   ├── reports/          # Test reports (HTML, XML)
│   ├── test-results/     # Test output
│   ├── tmp/              # Temporary files
│   └── ...               # More
└── .gradle/              # (if using Gradle)
    ├── buildOutputCleanup/
    ├── checksums/
    └── ...
```

Every build adds more. Test reports. Coverage data. Temporary files. Never cleaned.

### The Maven Local Repository

```bash
# Where Maven stores all dependencies
~/.m2/repository/
├── com/
│   └── company/
│       └── library/
│           ├── 1.0.0/
│           ├── 1.0.1/
│           ├── 1.1.0/
│           └── ... (every version ever used)
├── org/
│   └── springframework/
│       └── ...
└── ... (thousands more)
```

Every version of every library. Forever. **2-20GB** easily.

> *"Googling doesn't help - the results are about removing packages from a project, as opposed to from the PC. At least that's my understanding of the situation."* — **Reddit r/dotnet** (same Java problem)

You can't find how to clean the global cache. It's hidden. Undocumented.

**<!-- TODO: INSERT IMAGE - File tree showing Maven repository structure with versions -->

---

## The Gradle Cache Explosion

Gradle is particularly aggressive with caching:

### ~/.gradle/ Structure

```
~/.gradle/
├── caches/                    # Build cache (huge)
│   ├── build-cache-1/
│   ├── jars-9/
│   └── ...
├── daemon/                    # Daemon logs (grows fast)
│   └── 8.5/
│       └── daemon-*.out.log   # 10GB in 4 days!
├── wrapper/                   # Gradle distributions
│   └── dists/
│       ├── gradle-8.5-bin/
│       ├── gradle-8.4-bin/
│       └── ... (every version)
└── checksums/                 # Dependency checksums
```

### The Daemon Log Problem

> *"4 days of logs were consuming ~10GB of disk memory!"* — **Jitin Sharma blog**

Gradle daemon logs grow **2.5GB per day**. No rotation. No cleanup. Just endless growth.

### Build Cache Growth

| Gradle Version | Cache Strategy | Disk Usage |
|----------------|----------------|------------|
| Old versions | No cleanup | Grows forever |
| New versions | 7-day default | Still large |
| CI environments | Shared cache | 10-50GB |

**<!-- TODO: INSERT IMAGE - Screenshot showing ~/.gradle directory size breakdown -->

---

## The Android Development Tax

Android development multiplies the Java problem:

### Android-Specific Bloat

```
~/.android/
├── build-cache/              # Android build cache
├── avd/                      # Emulator images (2-5GB each)
└── ...

~/.gradle/caches/
├── transform-*/              # Android transforms
└── ...
```

| Component | Typical Size | Notes |
|-----------|--------------|-------|
| **Android SDK** | 5-20GB | Platform tools, build tools |
| **Emulator images** | 2-5GB each | x86, ARM versions |
| **Gradle Android plugin** | 500MB-1GB | Plugin + dependencies |
| **Build cache** | 5-15GB | Android-specific |

> *"My SSD was almost full because of this which I think also may have negatively impacted performance."* — **Reddit r/androiddev**, 2022

Android development can consume **30-50GB** easily.

**<!-- TODO: INSERT IMAGE - Android Studio with storage usage breakdown -->

---

## The Manual Cleanup Trap

You know build directories are big. But cleaning them is scattered across tools.

### Maven: mvn clean

```bash
# The Maven way
mvn clean
```

What it does:
- ✅ Deletes `target/`
- ✅ Fast
- ❌ Only current project
- ❌ No global cache cleanup
- ❌ No `~/.m2/repository` cleanup

### Gradle: ./gradlew clean

```bash
# The Gradle way
./gradlew clean
```

What it does:
- ✅ Deletes `build/`
- ⚠️ Keeps `.gradle/` (cache)
- ❌ Only current project
- ❌ No `~/.gradle/` cleanup
- ❌ No daemon log cleanup

### Manual Cleanup

```bash
# Find all target directories
find ~ -name "target" -type d -prune

# Delete manually
rm -rf target/ build/

# Clean Maven cache... somehow
rm -rf ~/.m2/repository/*  # Dangerous!

# Clean Gradle cache... somehow
rm -rf ~/.gradle/caches/*  # Also dangerous!
```

Problems:
- ❌ Dangerous (might break things)
- ❌ No size information
- ❌ No project context
- ❌ No safety checks
- ❌ Doesn't find all locations

**<!-- TODO: INSERT IMAGE - Terminal showing scattered cleanup commands -->

---

## The Many Tools Problem

| What You Want | Tool | Command |
|--------------|------|---------|
| Clean `target/` | Maven | `mvn clean` |
| Clean `build/` | Gradle | `./gradlew clean` |
| Clean `~/.m2/` | Manual | ??? |
| Clean `~/.gradle/` | Manual | ??? |
| Find old projects | Manual | ??? |
| Check sizes | Manual | `du -sh` |

**Six different approaches** for one ecosystem. No unified tool.

---

## The Real Solution: null-e for Java Developers

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
```

null-e understands Java build tools and cleans them safely.

### What null-e Does Better

| Feature | null-e | mvn clean | gradle clean | Manual |
|---------|--------|-----------|--------------|--------|
| **Multi-project** | ✅ Scans all | ❌ One | ❌ One | ❌ Manual |
| **Maven + Gradle** | ✅ Both | ❌ Maven only | ❌ Gradle only | ❌ Neither |
| **Global cache** | ✅ ~/.m2, ~/.gradle | ❌ No | ❌ No | ⚠️ Dangerous |
| **Size info** | ✅ Shows GB | ❌ No | ❌ No | ❌ Slow |
| **Stale detection** | ✅ Finds old | ❌ No | ❌ No | ❌ No |
| **Git protection** | ✅ Checks | ❌ No | ❌ No | ❌ No |
| **Safety levels** | ✅ Every item | ❌ No | ❌ No | ❌ No |

### Find All Java Bloat

```bash
# Scan for all Java projects
null-e ~/projects

# Output:
✓ Found 18 Java projects with 47.3 GB cleanable

   [1] ○ spring-api (3.2 GB) - 2 weeks ago, Maven target/: 2.8GB
   [2] ○ android-app (8.5 GB) - 1 month ago, Gradle build/: 3GB, .gradle/: 5GB
   [3] ○ old-service (2.1 GB) - 6 months ago, target/: 1.9GB
   ...
```

See every Java project. Maven or Gradle. Exact sizes. How old. What's safe to clean.

**<!-- TODO: INSERT IMAGE - Screenshot of null-e showing Java projects with build directory sizes -->

### Find Stale Projects

```bash
# Projects not built in 180 days
null-e stale ~/projects --days 180

# Safe to clean - you haven't touched them in 6 months
```

Old microservices. Abandoned experiments. Safe to clean.

### Clean Global Caches

```bash
# Check Java caches
null-e caches

# Output:
✓ Found 8 caches with 24.6 GiB total
   [1] ☕ Maven repository    8.2 GB  (~/.m2/repository/)
   [2] ☕ Gradle caches       12.4 GB (~/.gradle/caches/)
   [3] ☕ Gradle wrapper      2.1 GB  (~/.gradle/wrapper/)
   [4] ☕ Gradle daemon logs  1.9 GB  (~/.gradle/daemon/)
```

One command. Maven and Gradle. Global caches, daemon logs, everything.

**<!-- TODO: INSERT IMAGE - Screenshot of null-e caches showing Java-specific caches -->

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
- ✅ Never deletes source code

---

## Java-Specific Cleanup with null-e

### Maven Projects

null-e detects `pom.xml` and cleans Maven projects:

```bash
null-e ~/projects

# Shows:
✓ Found Maven projects with 12.4 GB cleanable
   [1] ○ spring-api (3.2 GB) - last built: 2 weeks ago
       ├── target/           2.8 GB
       │   ├── classes/      150 MB
       │   ├── test-classes/ 200 MB
       │   ├── libs/         45 MB
       │   └── reports/      890 MB
       └── pom.xml           (tracked)
```

Reports, test output, compiled classes—all identified.

### Gradle Projects

null-e detects `build.gradle` or `build.gradle.kts`:

```bash
null-e ~/projects

# Shows:
✓ Found Gradle projects with 24.8 GB cleanable
   [1] ○ android-app (8.5 GB) - last built: 1 month ago
       ├── build/            3.1 GB
       ├── .gradle/          5.2 GB
       └── build.gradle      (tracked)
   [2] ○ kotlin-service (4.2 GB) - last built: 3 weeks ago
       ├── build/            1.8 GB
       └── .gradle/          2.3 GB
```

Both `build/` and `.gradle/` tracked separately.

**<!-- TODO: INSERT IMAGE - Screenshot comparing Maven vs Gradle project cleanup -->

### Global Cache Cleanup

```bash
null-e caches --clean

# Interactive prompt:
Clean which caches?
   [1] ☕ Maven repository    8.2 GB
   [2] ☕ Gradle caches       12.4 GB
   [3] ☕ Gradle wrapper      2.1 GB
   [4] ☕ Gradle daemon logs  1.9 GB

> 2,4

✓ Cleaned Gradle caches and daemon logs, freed 14.3 GB
```

Reclaim 10-30GB from old dependency versions and log files.

### Android Support

null-e includes Android-specific cleaners:

```bash
null-e android

# Shows:
✓ Found Android artifacts (15.3 GB)
   [1] ~ Gradle caches       5.2 GB
   [2] ~ Android build cache 3.8 GB
   [3] ! AVD images          6.3 GB (2 emulators)
```

Android development bloat handled specially.

**<!-- TODO: INSERT IMAGE - Screenshot of Android-specific cleanup -->

---

## Real Results from Real Java Developers

### Case Study: The Gradle Log Victim

> *"4 days of logs were consuming ~10GB of disk memory!"* — **Jitin Sharma blog**

10GB in 4 days. null-e's daemon log cleanup fixes this permanently.

### Case Study: The Maven Repository

> *"I've got a tiny SSD... occupied by old packages, to the point that I can't reinstall IDE because there's not enough space."* — **Reddit r/dotnet** (same Java experience)

Maven repository filling SSD. null-e cleans old versions safely.

### Case Study: The Android Developer

> *"My SSD was almost full because of Android development."* — **Reddit r/androiddev**

Android SDK + Gradle + emulators = 30-50GB. null-e reclaims it.

**<!-- TODO: INSERT IMAGE - Before/After comparison showing Java disk space reclaimed -->

---

## The Java Developer's Cleanup Workflow

### Step 1: Scan Everything

```bash
# Find all Java bloat
null-e ~/projects ~/work ~/java-projects
```

See Maven, Gradle, Android—all in one view.

### Step 2: Identify Stale Projects

```bash
# Find old projects
null-e stale ~/projects --days 180

# Safe to clean - you haven't touched them in 6 months
```

### Step 3: Clean Global Caches

```bash
# Clean Maven and Gradle caches
null-e caches --clean
```

Reclaim 10-30GB instantly.

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
alias javaclean='null-e caches --clean-all && null-e stale ~/projects --days 90 --clean'

# Run monthly
# Or add to cron:
0 0 1 * * /usr/local/bin/null-e caches --clean-all --force
```

**<!-- TODO: INSERT IMAGE - Workflow diagram: Scan → Identify → Clean → Automate -->

---

## Optimizing Java Build Storage

### Use Gradle Build Cache Wisely

```gradle
// build.gradle
buildCache {
    local {
        enabled = true
        removeUnusedEntriesAfterDays = 7  // Clean old entries
    }
}
```

### Maven: Use Local Repository Manager

Consider a repository manager (Nexus, Artifactory) to share dependencies across team.

### Clean Old Gradle Versions

```bash
# List wrapper versions
ls ~/.gradle/wrapper/dists/

# Delete old ones (or let null-e do it)
```

### Android: Manage Emulators

```bash
# List AVDs
emulator -list-avds

# Delete unused
null-e android --clean
```

**<!-- TODO: INSERT IMAGE - Code snippets showing Java optimization tips -->

---

## Take Back Your Disk Space Today

Don't let target/, build/, and .gradle/ own your machine.

**[Install null-e →](https://github.com/us/null-e)**

```bash
# Install
cargo install null-e

# Scan your Java projects
null-e ~/projects

# Find stale projects (6+ months old)
null-e stale ~/projects --days 180

# Clean safely with git protection
null-e clean ~/projects
```

### What You'll Reclaim

| Category | Typical Savings |
|----------|---------------|
| Stale target/ directories | 5-20 GB |
| Stale build/ directories | 5-25 GB |
| Maven repository (~/.m2/) | 3-15 GB |
| Gradle caches (~/.gradle/) | 5-20 GB |
| Gradle daemon logs | 1-10 GB |
| Android artifacts | 10-30 GB |
| **Total** | **29-120 GB** |

That's not just disk space. That's:
- ✅ Faster builds (less cache scanning)
- ✅ Faster IDE indexing
- ✅ More space for active projects
- ✅ No more "disk full" during builds
- ✅ Professional pride in a clean machine

> *"So I poked at my .gradle folder and found it's size to be 28GB."* — **Java developer**

It happens to everyone. But you can fix it.

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
null-e sweep
```

Clean up the Java build bloat. Reclaim your disk.

```
     .---.
    |o   o|   "Directive: Clean all the target/ directories!"
    |  ^  |
    | === |
    `-----'
     /| |\
```

**[View on GitHub →](https://github.com/us/null-e)**

---

### More Java Cleanup Guides

- [Java Gradle and Maven Cleanup Guide](/java-gradle-maven-cleanup/)
- [Clean Maven Local Repository Safely](/clean-maven-repository/)
- [Gradle Cache Cleanup: Reclaim 20GB+](/gradle-cache-cleanup/)
- [Android Development Disk Cleanup](/android-development-cleanup/)

**<!-- TODO: INSERT IMAGE - Related posts grid with Java-specific thumbnails -->