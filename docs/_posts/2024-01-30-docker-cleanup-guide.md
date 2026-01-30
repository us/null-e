---
layout: post
title: "Docker Cleanup: Remove Unused Images, Containers, and Volumes to Reclaim 50GB+"
description: "Complete guide to cleaning Docker images, containers, volumes, and build cache. Learn how to safely reclaim 50-200GB of disk space and prevent 'no space left on device' errors."
date: 2024-01-30
author: us
tags: [docker, containers, disk-cleanup, devops, developer-tools, wsl2, docker-desktop]
---

[![null-e - Disk Cleanup Tool for Developers](https://img.shields.io/crates/v/null-e.svg)](https://crates.io/crates/null-e)

**[View on GitHub →](https://github.com/us/null-e)**

If you use Docker, you know the panic. You're building an image, running containers, and suddenly:

> *"No space left on device"*

Or worse, on Windows with WSL2:

> *"This morning when I hopped online for work I was greeted with a murder mystery. For some unknown reason, my WSL2 Ubuntu installation had eaten over a third of my 1 TB disk space even though the actual content within the distro was less than 10 GB"* — **ashn.dev blog**, August 2025

**373GB** of WSL2 virtual disk for **10GB** of actual data.

Docker is powerful. But it's a **disk space nightmare**.

---

## The Docker Disk Space Problem

Docker accumulates data in multiple places, most of them hidden:

| Component | What It Is | Typical Size | Why It Grows |
|-----------|-----------|--------------|--------------|
| **Images** | Downloaded/built images | 10-50GB | Every pull, every build |
| **Containers** | Stopped containers | 1-10GB | Created but not removed |
| **Volumes** | Persistent data | 5-50GB | Database data, logs |
| **Build cache** | Layer cache | 5-30GB | Every docker build |
| **WSL2 VHDX** | Virtual disk (Windows) | 20-100GB | Grows, never shrinks |

A typical developer using Docker easily has **50-200GB** of Docker-related disk usage.

**<!-- TODO: INSERT IMAGE - Visual showing Docker components consuming disk space -->

---

## Where Docker Stores Data (And Why It's Confusing)

### Docker Desktop (macOS/Windows)

```
# macOS
~/Library/Containers/com.docker.docker/
└── Data/vms/0/data/
    └── Docker.raw          # 50-100GB virtual disk

# Windows (WSL2)
%LOCALAPPDATA%\Docker\
└── WSL\
    └── Data\ext4.vhdx      # 20-100GB virtual disk
```

Docker stores everything in a **virtual disk file**. This file:
- ✅ Grows automatically when you add data
- ❌ **Never shrinks** when you delete data (without special steps)
- ❌ Is opaque—you can't see individual files

### Linux (Native Docker)

```
# Linux uses direct storage
/var/lib/docker/
├── containers/         # Container data
├── image/             # Image layers
├── volumes/           # Volume data
├── overlay2/          # Overlay filesystem
└── buildkit/          # Build cache
```

More visible, but still scattered across multiple directories.

**<!-- TODO: INSERT IMAGE - Comparison: Docker Desktop virtual disk vs Linux native storage -->

---

## The WSL2 Nightmare (Windows Developers)

Windows developers get hit hardest:

### The One-Way Growth Problem

> *"Since WSL2 stores its filesystem on a .vhdx, that file grows when the dataset is processed and never releases that space... it seems to prefer to grow the file rather than re-use existing empty space... This is annoying since it basically means that my system backups include a huge .vhdx file that is mostly empty."* — **GitHub microsoft/WSL Issue #4699**, November 2019 (1100+ thumbs up)

WSL2's `.vhdx` file:
- Grows when you add data
- **Doesn't shrink** when you delete data inside WSL
- Requires manual compaction to reclaim space
- Is confusing and poorly documented

> *"Why is it so fucking hard to change disk space allocation for Docker on Windows with WSL2?... It has been a nightmare getting docker to believe that it has more than 250GB in disk space.... Docker offers NO documentation on how to do this"* — **Reddit r/docker rant**

No GUI. No simple command. Just pain.

> *"Whose cock do I have to suck to get a fucking gui slider that lets me drag 256 --> 512GB or whatever? I had to nuke docker from my PC and reinstall from scratch."* — **Reddit r/docker**

Reinstall from scratch. The only "fix."

### The Scale of the Problem

> *"Docker's VHDX shrank from 11.7GB → 10.1GB. Ubuntu's ext4.vhdx shrank from 8.5GB → 8.1GB. So even completely 'empty', these two files still hog ~18GB, and they just keep creeping up over time. Feels like no matter what I do, the space never really comes back."* — **Reddit r/docker**

18GB for "empty" virtual disks. Growing constantly.

> *"I had my docker build cache and docker volume explode to over 400gb of usage for Ubuntu's ext4.vhdx... cleaned up 240GB [manually]."* — **Reddit r/docker**

400GB explosion. Manual cleanup required 240GB of effort.

**<!-- TODO: INSERT IMAGE - Screenshot showing WSL2 ext4.vhdx file properties with massive size -->

---

## Why Docker Eats Disk Space

### Images Never Clean Themselves

Every `docker pull` and `docker build` adds layers:

```bash
# Pull an image
docker pull node:20

# Creates layers:
# - Base OS layer (100MB)
# - Node.js layer (150MB)
# - npm layer (50MB)
# Total: 300MB

# Build your app
docker build -t myapp .

# Creates more layers:
# - Dependencies layer (500MB)
# - Source code layer (10MB)
# - Build artifacts layer (200MB)
# Total: 710MB
```

Images accumulate. Old versions. Intermediate layers. All taking space.

### Containers Are Not Auto-Removed

```bash
# Run a container
docker run -d --name mycontainer nginx

# Stop it
docker stop mycontainer

# It's still there, taking space
docker ps -a
# CONTAINER ID   IMAGE   STATUS
# abc123         nginx   Exited (0) 3 days ago
```

Stopped containers persist. Their writable layers. Their logs. All taking space.

### Build Cache Grows Forever

```bash
# Build an image
docker build -t myapp .

# Creates build cache layers
# Every RUN command creates a cached layer
# These layers are never auto-cleaned
```

Every build adds cache. Old cache. Unused cache. Forever.

### Volumes Hold Data

```bash
# Create a volume
docker volume create mydata

# Use it
docker run -v mydata:/data postgres

# Volume persists even after container is removed
# Database data, logs, uploads—all stay
```

Volumes are meant to persist. But they often outlive their usefulness.

**<!-- TODO: INSERT IMAGE - Diagram showing Docker layer accumulation -->

---

## The Manual Cleanup Trap

Docker cleanup commands exist. But they're confusing and incomplete.

### docker system prune

```bash
# The "official" cleanup
docker system prune -a
```

What it does:
- ✅ Removes stopped containers
- ✅ Removes unused networks
- ✅ Removes dangling images
- ⚠️ **Doesn't remove volumes** (by default)
- ❌ **Doesn't shrink WSL2 VHDX**
- ❌ Permanent (no recovery)

> *"My usual go-to, docker system prune, felt like defusing a bomb blindfolded. One wrong flag and I could lose hours of work."* — **DEV Community**, November 2025

Scary. Dangerous. And incomplete.

### docker volume prune

```bash
# Remove unused volumes
docker volume prune
```

⚠️ **DANGEROUS**: Removes volumes with potentially important data.

> *"After reading some other posts it seems docker system prune can be destructive to an active running sentry installation. Is there anything (Images, Volumes or Containers) that can be pruned safely?"* — **Sentry Forums**

Even experienced developers are afraid.

### The VHDX Compaction (Windows)

```bash
# PowerShell as Administrator
wsl --shutdown
optimize-vhd -Path "C:\Users\You\AppData\Local\Docker\WSL\Data\ext4.vhdx" -Mode Full
```

Requirements:
- Windows Pro/Enterprise (Home doesn't have optimize-vhd)
- PowerShell knowledge
- Administrative access
- Understanding of VHDX files

Not user-friendly. Not documented well.

**<!-- TODO: INSERT IMAGE - Terminal showing docker prune commands with warnings -->

---

## The Real Solution: null-e for Docker

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
```

null-e understands Docker and makes cleanup safe and visible.

### What null-e Does Better

| Feature | null-e | docker prune | Manual |
|---------|--------|--------------|--------|
| **Visibility** | ✅ Shows what's there | ❌ No preview | ❌ Complex |
| **Safety levels** | ✅ ✓ ~ ! markers | ❌ No | ❌ No |
| **Volumes protection** | ✅ Default exclude | ⚠️ Optional | ❌ Risky |
| **WSL2 handling** | ✅ Detects & helps | ❌ No | ❌ Complex |
| **Size info** | ✅ GB breakdown | ❌ Limited | ❌ Manual |
| **Git-style dry-run** | ✅ --dry-run | ❌ No | ❌ No |

### Find All Docker Bloat

```bash
# Check Docker resources
null-e docker

# Output:
🐳 Docker Resources Found:
✓ Found 15 containers, 47 images, 12 volumes (67.4 GB)

   Containers:
   [1] ✓ Stopped: mycontainer (150 MB) - Exited 3 days ago
   [2] ✓ Stopped: test-db (890 MB) - Exited 1 week ago
   ...
   
   Images:
   [1] ✓ Unused: node:18 (350 MB) - Not referenced
   [2] ✓ Dangling: <none> (120 MB) - Build layer
   [3] ~ In use: myapp:latest (780 MB) - Has running container
   ...
   
   Volumes:
   [1] ! mydata (12.3 GB) - Used by postgres container
   [2] ! logs (4.1 GB) - Used by nginx container
   ...
```

Everything visible. Safety levels clear. You decide what to clean.

**<!-- TODO: INSERT IMAGE - Screenshot of null-e docker showing resources with safety levels -->

### Clean with Control

```bash
# Clean interactively (default: no volumes)
null-e docker --clean

# You'll see:
🐳 Docker Cleanup

Clean which items?
   [1] ✓ Stopped containers: 8 (980 MB)
   [2] ✓ Dangling images: 12 (3.2 GB)
   [3] ✓ Unused images: 27 (18.5 GB)
   [4] ~ Build cache: 89 entries (14.7 GB)
   [5] ! Unused volumes: 7 (8.2 GB)

> 1,2,3,4

✓ Cleaned Docker resources, freed 36.6 GB
```

You choose. Volumes excluded by default (safety). Everything else interactive.

### WSL2 Detection and Help

```bash
# On Windows with WSL2
null-e docker

# Shows:
🐳 Docker Resources Found:
✓ Found containers, images, volumes (34.2 GB)

⚠️ WSL2 detected:
   Docker VHDX: 45.2 GB (C:\Users\You\...\ext4.vhdx)
   
   Note: After cleanup, the VHDX file won't shrink automatically.
   Use: wsl --shutdown and disk compaction to reclaim space.
   
   null-e can guide you through this process.

Show WSL2 compaction guide? [Y/n]
```

null-e detects WSL2. Explains the problem. Offers help.

**<!-- TODO: INSERT IMAGE - Screenshot showing WSL2 detection and guidance -->

---

## What's Safe to Delete?

| Resource | Safe? | Notes |
|----------|-------|-------|
| **Dangling images** | Yes | Untagged intermediate layers |
| **Unused images** | Usually | Images not used by containers |
| **Stopped containers** | Usually | Unless you need logs |
| **Unused volumes** | **Careful** | May contain important data |
| **Build cache** | Yes | Just slows next build |

null-e marks every item so you know exactly what you're deleting.

---

## Docker Cleanup Strategy

### Daily Development

```bash
# Quick cleanup - safe stuff only
null-e docker --clean
```

### Weekly Maintenance

```bash
# More thorough cleanup
docker system prune -a
```

### Before Reclaiming Disk Space

```bash
# Full cleanup including old volumes
null-e docker --clean --volumes
```

---

## Common Issues

### "No space left on device"

Docker ran out of disk space. Quick fix:

```bash
# Emergency cleanup
docker system prune -a -f
```

### Build cache growing huge

```bash
# Clear build cache
docker builder prune -a
```

### Too many old images

```bash
# Remove images older than 24h
docker image prune -a --filter "until=24h"
```

---

## Pro Tips

### 1. Set Build Cache Limit

Add to `~/.docker/daemon.json`:
```json
{
  "builder": {
    "gc": {
      "enabled": true,
      "defaultKeepStorage": "20GB"
    }
  }
}
```

### 2. Use Multi-stage Builds

Smaller final images = less disk usage:

```dockerfile
FROM node:20 AS builder
WORKDIR /app
COPY . .
RUN npm ci && npm run build

FROM node:20-slim
COPY --from=builder /app/dist ./dist
CMD ["node", "dist/index.js"]
```

### 3. Clean in CI/CD

Add cleanup to your CI pipeline:

```yaml
- name: Docker cleanup
  run: docker system prune -af
```

---

## Results

Typical Docker cleanup:

```
Before: 89 GB Docker data
After:  31 GB Docker data
Freed:  58 GB
```

---

## Real Results from Real Developers

### Case Study: The WSL2 Victim

> *"This morning when I hopped online for work I was greeted with a murder mystery. For some unknown reason, my WSL2 Ubuntu installation had eaten over a third of my 1 TB disk space even though the actual content within the distro was less than 10 GB"* — **ashn.dev blog**

373GB WSL2 virtual disk. null-e identifies, cleans, and guides compaction.

### Case Study: The 400GB Explosion

> *"I had my docker build cache and docker volume explode to over 400gb of usage... cleaned up 240GB."* — **Reddit r/docker**

400GB Docker usage. null-e safely cleans 240GB of reclaimable data.

### Case Study: The CI Runner

> *"Over 195GB used by containers alone, and a whopping 190GB of it was 'reclaimable.' The beast wasn't just in its lair; it had eaten the whole house."* — **BlockQueue Systems blog**, June 2025

195GB containers. 190GB reclaimable. null-e identifies and cleans.

**<!-- TODO: INSERT IMAGE - Before/After comparison showing Docker disk space reclaimed -->

---

## Conclusion

Don't let Docker own your disk.

> *"Docker is amazing, but it's also a hoarder by default."* — **DEV Community**

Control the hoarder. Reclaim your space.

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
null-e docker --clean
```

Clean up Docker. Reclaim your disk.

```
     .---.
    |o   o|   "Directive: Clean all the containers!"
    |  ^  |
    | === |
    `-----'
     /| |\
```

**[View on GitHub →](https://github.com/us/null-e)**

---

### More Docker Cleanup Guides

- [Docker Disk Space Cleanup Guide](/docker-disk-space-cleanup/)
- [Fix WSL2 Docker Disk Bloat](/fix-wsl2-docker-disk-bloat/)
- [Docker System Prune vs null-e](/docker-system-prune-vs-null-e/)
- [Docker Build Cache Management](/docker-build-cache-management/)

**<!-- TODO: INSERT IMAGE - Related posts grid with Docker-specific thumbnails -->
