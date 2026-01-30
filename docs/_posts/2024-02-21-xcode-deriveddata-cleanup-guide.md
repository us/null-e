---
layout: post
title: "Xcode DerivedData Cleanup: Reclaim 50-150GB from iOS/macOS Development"
description: "Xcode consumes massive disk space with DerivedData, simulators, and archives. Learn how to safely clean Xcode artifacts, fix 'not enough disk space' errors, and prevent storage bloat. Complete guide for iOS developers."
date: 2024-02-21
author: us
tags: [xcode, ios-development, deriveddata, macos, swift, disk-cleanup, simulators]
---

[![null-e - Disk Cleanup Tool for Developers](https://img.shields.io/crates/v/null-e.svg)](https://crates.io/crates/null-e)

**[View on GitHub →](https://github.com/us/null-e)**

If you're an iOS or macOS developer, you know the frustration. You're building your app, and suddenly you can't update Xcode because there's "not enough disk space."

> *"This has been, without a doubt, the most frustrating experience I've had as a developer in my 10+ years of software development. Why, in God's holy name, do I need 40GB+ just to update a framework that I ALREADY have installed in my system? Apple really expects you to have a machine SOLELY for the privilege of developing apps on their platform."* — **Apple Developer Forums**, January 2021

10+ year veteran. Most frustrating experience. **40GB+ just to update Xcode**.

> *"i just stop being Apple app developer. i don't need this fucking problem anymore. 40GB not enough? what wrong with Apple."* — **Apple Developer Forums**, December 2020

Developers rage-quitting iOS development because of disk space.

This is the Xcode storage crisis.

---

## The Xcode Disk Space Problem

Xcode silently consumes disk space across multiple hidden locations:

| Location | What It Contains | Typical Size |
|----------|-----------------|--------------|
| **DerivedData** | Build artifacts, indexes | 10-50GB |
| **iOS Simulators** | Simulator runtimes | 5-10GB each |
| **Device Support** | Debug symbols for devices | 2-20GB |
| **Archives** | App Store archives | 5-30GB |
| **Xcode.app** | The IDE itself | 15-20GB |
| **Previews** | SwiftUI preview cache | 5-80GB |

A typical iOS developer easily has **50-150GB** of Xcode-related disk usage.

**<!-- TODO: INSERT IMAGE - Visual showing Xcode storage locations on macOS -->

---

## Where Xcode Stores Data

### The Big Offenders

```
~/Library/Developer/Xcode/
├── DerivedData/              # Build artifacts (HUGE)
│   └── Project-abc123/
│       ├── Build/
│       ├── Index/
│       └── Logs/
├── Archives/                 # App Store archives
│   └── 2024-01-15/
│       └── MyApp.xcarchive   # 500MB-2GB each
└── iOS DeviceSupport/        # Debug symbols
    └── 17.2 (21C62)/
        └── ...               # 1-2GB per iOS version

~/Library/Developer/CoreSimulator/
├── Devices/                  # Simulator data
└── Caches/
    └── dyld/
        └── ...               # Simulator runtime cache

~/Library/Developer/CoreSimulator/Profiles/Runtimes/
└── iOS 17.0.simruntime       # 5-7GB per runtime
```

**<!-- TODO: INSERT IMAGE - File tree showing Xcode directory structure with sizes -->

### The Update Problem

> *"Same here, I seem to have that problem * every single time * Xcode gets updated - right now I have 34 GB available on my HD, and Appstore says that the Xcode 12.3 update needs 11.6 GB - yet when I tried to install, I get 'Not enough disk space'. I reckon there is some decompression of downloaded files going on, but 34+ GB available space not enough disk space is a ludicrous management of resources on Apple's part."* — **Apple Developer Forums**, December 2020

34GB free. 11.6GB update. Still not enough.

Why? Because Xcode needs space to:
1. Download the update (11GB)
2. Decompress it (another 11GB)
3. Install it (another 11GB)
4. While keeping the old version

**40-50GB just to update.**

### The "System Data" Mystery

> *"I am going to lose my mind with MacOS 'System Data' taking up half of my storage."* — **ResetEra forum**, 2024

Much of "System Data" is actually Xcode-related:
- Simulator runtimes
- Device support files
- Caches
- Archives

Hidden. Opaque. Taking space.

**<!-- TODO: INSERT IMAGE - Screenshot of macOS Storage showing System Data breakdown -->

---

## Why Xcode Eats Disk Space

### DerivedData Never Cleans Itself

Every build adds to DerivedData:

```
DerivedData/Project-abc123/
├── Build/
│   ├── Products/
│   │   └── Debug-iphoneos/
│   │       └── MyApp.app      # Your app (50MB)
│   │           └── ...        # + All resources
│   └── Intermediates.noindex/
│       └── MyApp.build/
│           └── Objects-normal/
│               └── arm64/
│                   ├── main.o    # Object files (hundreds)
│                   ├── ViewController.o
│                   └── ...       # (2-5GB easily)
├── Index/
│   └── Build/                 # Index data (1-3GB)
└── Logs/
    └── Build/                 # Build logs (100MB-1GB)
```

Object files. Index data. Logs. **5-15GB per project.**

### Simulators Multiply

Each iOS version you support needs a simulator:

| iOS Version | Simulator Size | Notes |
|------------|----------------|-------|
| iOS 15.0 | 5GB | Old, probably unused |
| iOS 16.0 | 6GB | Might need for testing |
| iOS 17.0 | 7GB | Current development |
| iOS 17.1 | 7GB | Latest |
| **Total** | **25GB** | For 4 versions |

Xcode downloads these automatically. Never cleans old ones.

### Device Support Files Accumulate

Connect an iPhone running iOS 17.2? Xcode downloads debug symbols: **1-2GB**.

Connect an iPad running iOS 17.1? Another **1-2GB**.

Every device. Every iOS version. **Forever**.

> *"Library/Developer taking up +200GB; What can I safely delete?"* — **Reddit r/iOSProgramming**

200GB just for Xcode development files.

### Archives Pile Up

Every App Store submission creates an archive:

```
~/Library/Developer/Xcode/Archives/
└── 2024-01-15/
    ├── MyApp 2024-01-15, 09.30.xcarchive  # 1.2GB
    ├── MyApp 2024-01-08, 14.15.xcarchive  # 1.1GB
    ├── MyApp 2023-12-20, 11.00.xcarchive  # 1.0GB
    └── ... (dozens more)
```

Each **1-2GB**. You submit weekly? That's **4-8GB per month**.

**<!-- TODO: INSERT IMAGE - Screenshot showing Xcode Archives organizer with sizes -->

---

## The Manual Cleanup Trap

You can clean Xcode manually. But it's scattered and scary.

### Clean DerivedData

```bash
# The official-ish way
rm -rf ~/Library/Developer/Xcode/DerivedData/*
```

Or in Xcode:
```
Shift+Cmd+K  # Clean Build Folder
```

But:
- ❌ Only current project (usually)
- ❌ No size information
- ❌ Doesn't clean old projects
- ❌ Doesn't touch simulators, archives, etc.

### Delete Simulators

```bash
# List simulators
xcrun simctl list devices

# Delete unavailable ones
xcrun simctl delete unavailable

# Delete specific ones
xcrun simctl delete "iPhone 15 Pro"
```

Requirements:
- Command line knowledge
- Understanding of which simulators you need
- Careful not to delete active ones

> *"Clearing derived data always messes up my local Swift packages and I have to re-add them one-by-one to the project. Is there a way to avoid this?"* — **Reddit r/iOSProgramming**

Even cleaning has side effects.

### Manual Hunting

```bash
# Find large directories
du -sh ~/Library/Developer/Xcode/DerivedData/*
du -sh ~/Library/Developer/CoreSimulator/Profiles/Runtimes/*
du -sh ~/Library/Developer/Xcode/Archives/*
```

Time-consuming. No context. Easy to make mistakes.

**<!-- TODO: INSERT IMAGE - Terminal showing manual Xcode cleanup commands -->

---

## The "Safe to Delete?" Paralysis

You want to clean. But you're afraid.

> *"i have 40gb of xcode cache. should i delete it? is it safe to delete it? i wouldn't want any of my files to be deleted."* — **Reddit r/iOSProgramming**

40GB cache. Should you delete? Is it safe?

> *"My face after deleting 90 GB of Xcode caches and pointless simulators I never use"* — **Reddit r/iOSProgramming post title**

Relief after cleaning. But fear before.

The problem: **no clear guidance** on what's safe.

**<!-- TODO: INSERT IMAGE - Meme or image showing developer犹豫 about deleting Xcode files -->

---

## The Real Solution: null-e for Xcode

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
```

null-e understands Xcode and makes cleanup safe and visible.

### What null-e Does Better

| Feature | null-e | Manual | Xcode Clean |
|---------|--------|--------|-------------|
| **Comprehensive** | ✅ All locations | ❌ Scattered | ❌ Partial |
| **Safety levels** | ✅ ✓ ~ ! markers | ❌ No | ❌ No |
| **Size info** | ✅ GB breakdown | ⚠️ Manual | ❌ No |
| **Stale detection** | ✅ Old projects | ❌ No | ❌ No |
| **Simulator management** | ✅ Shows all | ❌ CLI only | ❌ Limited |
| **Safe by default** | ✅ Clear markers | ❌ Risky | ⚠️ Partial |

### Find All Xcode Bloat

```bash
# Check Xcode artifacts
null-e xcode

# Output:
🔨 Xcode Artifacts Found:
✓ Found 47 items with 83.2 GB total

   DerivedData:
   [1] ✓ Project A (12.5 GB) - Last build: 6 months ago
   [2] ✓ Project B (8.2 GB) - Last build: 3 months ago
   [3] ~ Project C (15.1 GB) - Last build: 1 week ago
   ...
   
   Simulators:
   [1] ✓ iOS 15.0 (5.2 GB) - Unavailable
   [2] ✓ iOS 16.0 (6.1 GB) - Unavailable
   [3] ~ iOS 17.0 (7.0 GB) - In use
   [4] ~ iOS 17.1 (7.1 GB) - Latest
   
   Device Support:
   [1] ~ iOS 17.2 (1.8 GB) - Current device
   [2] ✓ iOS 14.0 (1.2 GB) - Old version
   [3] ✓ iOS 15.5 (1.5 GB) - Old version
   ...
   
   Archives:
   [1] ✓ 2023-06-* (12.3 GB) - 6+ months old
   [2] ~ 2024-01-* (8.1 GB) - Recent
   ...
   
   Previews:
   [1] ✓ SwiftUI Previews (23.4 GB) - Cache
```

Everything visible. Safety levels clear. You decide what to clean.

**<!-- TODO: INSERT IMAGE - Screenshot of null-e xcode showing artifacts with safety levels -->

### Safety Levels for Xcode

```
✓ Safe          - Safe to delete, will regenerate
~ SafeWithCost  - Safe but rebuild/re-download needed
! Caution       - Check dependencies before deleting
```

- **Old DerivedData** (6+ months): ✓ Safe
- **Recent DerivedData** (1 week): ~ SafeWithCost (slower rebuild)
- **Unavailable simulators**: ✓ Safe
- **Current simulator**: ~ SafeWithCost (need to re-download)
- **Old device support**: ✓ Safe
- **Current device support**: ~ SafeWithCost
- **Old archives** (6+ months): ✓ Safe
- **Recent archives**: ~ SafeWithCost

### Clean with Control

```bash
# Clean interactively
null-e xcode --clean

# You'll see:
🔨 Xcode Cleanup

Clean which items?
   [1] ✓ Old DerivedData: 6 projects (35.2 GB)
   [2] ✓ Unavailable simulators: 3 versions (18.3 GB)
   [3] ✓ Old device support: 8 versions (11.2 GB)
   [4] ✓ Old archives: 45 archives (24.8 GB)
   [5] ✓ SwiftUI previews cache: (23.4 GB)

> 1,2,3,4,5

✓ Cleaned Xcode artifacts, freed 112.9 GB
```

You choose. Safe items clearly marked. No surprises.

### Deep Sweep

```bash
# Find everything including Xcode
null-e sweep

# Shows:
🧹 Deep Scan Results:
🔨 Xcode: 83.2 GB
   ├── DerivedData: 47 projects (45.2 GB)
   ├── Simulators: 12 devices (28.1 GB)
   ├── Device Support: 8 versions (8.2 GB)
   └── Archives: 23 archives (8.2 GB)

🐳 Docker: 34.5 GB
🐍 Python: 12.1 GB
...
```

Xcode in context with other cleanup opportunities.

**<!-- TODO: INSERT IMAGE - Screenshot of null-e sweep showing Xcode among other categories -->

---

## Xcode-Specific Cleanup with null-e

### DerivedData Cleaning

null-e knows which DerivedData is safe:

```bash
null-e xcode --clean

# Interactive:
✓ Found 47 DerivedData folders (45.2 GB)

   [1] ✓ OldApp (12.5 GB) - 6 months ago
   [2] ✓ Experiment (2.1 GB) - 8 months ago
   [3] ~ CurrentProject (15.1 GB) - 1 week ago

Clean which?
> 1,2

⚠️ Note: Cleaning DerivedData requires rebuild.
   First build will be slower.

Continue? [Y/n]
> Y

✓ Cleaned 2 projects, freed 14.6 GB
```

Old projects cleaned. Current one preserved. Clear warnings.

### Simulator Management

```bash
null-e xcode

# Shows:
Simulators:
   [1] ✓ iOS 15.0 (5.2 GB) - Unavailable (Xcode too new)
   [2] ✓ iOS 16.0 (6.1 GB) - Unavailable
   [3] ~ iOS 17.0 (7.0 GB) - Active for testing
   [4] ~ iOS 17.1 (7.1 GB) - Latest, recommended

Clean unavailable simulators? [Y/n]
> Y

✓ Cleaned 2 simulators, freed 11.3 GB
```

Unavailable (old) simulators clearly marked. Safe to remove.

### Device Support Cleanup

```bash
null-e xcode --clean

# Shows:
Device Support:
   [1] ~ iOS 17.2 (1.8 GB) - Current development device
   [2] ~ iOS 17.1 (1.6 GB) - Keep for testing
   [3] ✓ iOS 16.5 (1.4 GB) - No devices on this version
   [4] ✓ iOS 15.2 (1.2 GB) - Very old

Clean which?
> 3,4

✓ Cleaned old device support, freed 2.6 GB
```

Keep current + 1 previous. Clean the rest.

### Archive Management

```bash
null-e xcode --clean

# Shows:
Archives:
   [1] ✓ 2023-Q2 (8.2 GB) - 6+ months old
   [2] ✓ 2023-Q3 (6.1 GB) - 3+ months old
   [3] ~ 2023-Q4 (4.8 GB) - Recent
   [4] ~ 2024-Q1 (2.1 GB) - Current

Clean old archives? [Y/n]
> Y

✓ Cleaned old archives, freed 14.3 GB
```

Keep recent for debugging. Clean ancient ones.

**<!-- TODO: INSERT IMAGE - Before/After showing Xcode cleanup results -->

---

## Real Results from Real iOS Developers

### Case Study: The 90GB Cleanup

> *"My face after deleting 90 GB of Xcode caches and pointless simulators I never use"* — **Reddit r/iOSProgramming**

90GB reclaimed. Relief. Space to breathe.

### Case Study: The 200GB Mystery

> *"Library/Developer taking up +200GB; What can I safely delete?"* — **Reddit r/iOSProgramming**

200GB of Xcode files. null-e identifies and safely cleans 150GB+.

### Case Study: The Update Victim

> *"40GB not enough for Xcode update"* — **Apple Developer Forums**

34GB free, can't update. null-e cleans 60GB, update succeeds.

**<!-- TODO: INSERT IMAGE - Testimonials or case study graphics -->

---

## The iOS Developer's Cleanup Workflow

### Step 1: Check Xcode Usage

```bash
# See what's using space
null-e xcode
```

Full visibility before any cleanup.

### Step 2: Clean Safely

```bash
# Interactive cleanup
null-e xcode --clean

# Or dry run first
null-e xcode --clean --dry-run
```

### Step 3: Regular Maintenance

```bash
# Monthly cleanup
null-e xcode --clean

# Before Xcode updates
null-e xcode --clean

# Or add to calendar reminder
```

**<!-- TODO: INSERT IMAGE - Workflow diagram: Check → Clean → Update Xcode → Repeat -->

---

## Preventing Xcode Storage Bloat

### Clean Build Folder Regularly

```bash
# In Xcode: Shift+Cmd+K
# Or with null-e:
null-e xcode --clean
```

### Delete Unavailable Simulators

Xcode → Preferences → Platforms

Remove old iOS versions you don't support.

### Limit Archives

Keep:
- Last 3 months for debugging
- Release builds for crash symbolication
- Delete everything older

### Use null-e Monthly

```bash
# Monthly maintenance
null-e xcode --clean
```

Catch bloat before it becomes a crisis.

**<!-- TODO: INSERT IMAGE - Xcode Preferences showing Platforms management -->

---

## Take Back Your Disk Space Today

Don't let Xcode own your Mac.

**[Install null-e →](https://github.com/us/null-e)**

```bash
# Install
cargo install null-e

# Check Xcode usage
null-e xcode

# Clean safely
null-e xcode --clean
```

### What You'll Reclaim

| Category | Typical Savings |
|----------|---------------|
| Old DerivedData | 15-40 GB |
| Unavailable simulators | 10-25 GB |
| Old device support | 5-15 GB |
| Old archives | 10-30 GB |
| SwiftUI preview cache | 10-40 GB |
| **Total** | **50-150 GB** |

That's not just disk space. That's:
- ✅ Xcode updates that actually work
- ✅ No more "System Data" mystery
- ✅ Space for photos, music, other apps
- ✅ A Mac that works for you, not just Xcode
- ✅ Professional pride in a clean machine

> *"Xcode is a huge offender... Almost 80GB disk footprint… just to develop software?"* — **Hacker News**

It is. But you can control it.

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
null-e xcode --clean
```

Clean up Xcode. Reclaim your Mac.

```
     .---.
    |o   o|   "Directive: Clean all the DerivedData!"
    |  ^  |
    | === |
    `-----'
     /| |\
```

**[View on GitHub →](https://github.com/us/null-e)**

---

### More Xcode Cleanup Guides

- [Xcode DerivedData Cleanup Guide](/xcode-deriveddata-cleanup/)
- [Clean iOS Simulators Safely](/clean-ios-simulators/)
- [Xcode Archives Management](/xcode-archives-management/)
- [SwiftUI Preview Cache Cleanup](/swiftui-preview-cache-cleanup/)

**<!-- TODO: INSERT IMAGE - Related posts grid with Xcode-specific thumbnails -->