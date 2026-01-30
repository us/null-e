---
layout: default
title: null-e - Disk Cleanup Tool for Developers
description: Clean node_modules, target, .venv, Docker images, Xcode caches and 50+ cache types. Reclaim 100+ GB of disk space.
---

## The Problem

As developers, our disks fill up fast:

- **node_modules** folders everywhere (500MB-2GB each)
- **Rust target/** directories eating gigabytes
- **Python .venv** scattered across projects
- **Docker images** you forgot about
- **Xcode DerivedData** growing endlessly
- **Global caches** from npm, pip, cargo, homebrew...

**The result?** "Your disk is almost full" notifications.

## The Solution

```bash
cargo install null-e
null-e sweep
```

null-e scans your system and finds everything that can be safely cleaned:

| Category | What it finds | Typical savings |
|----------|--------------|-----------------|
| Project Artifacts | node_modules, target, .venv, build | 10-100 GB |
| Global Caches | npm, pip, cargo, go, maven | 5-50 GB |
| Xcode | DerivedData, Simulators, Archives | 20-100 GB |
| Docker | Images, Containers, Build Cache | 10-100 GB |
| ML/AI | Huggingface, Ollama, PyTorch | 10-100 GB |
| IDE Caches | JetBrains, VS Code, Cursor | 2-20 GB |

## Features

<div class="features">
  <div class="feature">
    <h4>⚡ Fast</h4>
    <p>Parallel scanning with Rust. Scans thousands of files in seconds.</p>
  </div>
  <div class="feature">
    <h4>🛡️ Safe</h4>
    <p>Git protection, moves to trash by default. Never lose work.</p>
  </div>
  <div class="feature">
    <h4>🎯 Smart</h4>
    <p>Detects 15+ languages and frameworks. Knows what's safe to delete.</p>
  </div>
  <div class="feature">
    <h4>💻 Cross-platform</h4>
    <p>Works on macOS, Linux, and Windows. Same commands everywhere.</p>
  </div>
</div>

## Quick Start

### Install
```bash
cargo install null-e
```

### Basic Usage
```bash
# Scan current directory
null-e

# Deep sweep - find EVERYTHING
null-e sweep

# Clean global caches
null-e caches

# Xcode cleanup (macOS)
null-e xcode

# Docker cleanup
null-e docker
```

## Why "null-e"?

> `/dev/null` + Wall-E = **null-e**
>
> Like the adorable trash-compacting robot from the movie, null-e tirelessly cleans up your developer junk and sends it where it belongs!

## Documentation

{% for post in site.posts limit:10 %}
<div class="blog-item">
  <a href="{{ post.url | relative_url }}">{{ post.title }}</a>
  <div class="blog-meta">
    {{ post.date | date: "%B %d, %Y" }}
    {% if post.tags %}
    {% for tag in post.tags limit:3 %}
    <span class="tag">{{ tag }}</span>
    {% endfor %}
    {% endif %}
  </div>
</div>
{% endfor %}

<p style="margin-top: 20px;">
  <a href="{{ site.baseurl }}/blog/" style="color: #267CB9; font-weight: 600;">View all {{ site.posts | size }} guides →</a>
</p>
