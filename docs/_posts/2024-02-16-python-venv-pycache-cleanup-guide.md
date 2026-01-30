---
layout: post
title: "How .venv and __pycache__ Are Eating Your Disk Space (And How to Clean Python Projects Safely)"
description: "Python developers lose 10-50GB to virtual environments and cache files. Learn how to safely clean .venv, __pycache__, and pip cache without breaking your projects. Complete guide with real developer stories."
date: 2024-02-16
author: us
tags: [python, virtualenv, venv, pycache, pip, disk-cleanup, pyenv, conda]
---

[![null-e - Disk Cleanup Tool for Developers](https://img.shields.io/crates/v/null-e.svg)](https://crates.io/crates/null-e)

**[View on GitHub →](https://github.com/us/null-e)**

If you're a Python developer, you've seen it. You run `pip install` in a new project, create a virtual environment, and suddenly your disk space starts vanishing.

> *"I just saw that my conda environment is 13GB in size. I looked up which packages are taking more space and they are mostly pytorch / cuda modules. I wonder if that's normal or if there's any way to avoid using so much disk space."* — **PyTorch Forums**, February 2023

13GB. For a single conda environment. And PyTorch with CUDA is the culprit.

> *"When I run $ du -hs * in ~/.virtualenvs, it shows that the size of most of them is over 400MB. The total space consumed by virtualenvs is over 6GB. Also, there are a lot of packages which are common across all the environments."* — **GitHub packaging-problems issue**, February 2020

6GB of virtual environments. With massive duplication across projects.

This is Python development: powerful, convenient, and quietly consuming your disk.

---

## The Python Virtual Environment Problem

Every Python project you work on likely has its own virtual environment. And each one is a complete Python installation plus all dependencies:

| Environment Type | Typical Size | Contents |
|-----------------|--------------|----------|
| Simple Flask/Django | 200-500 MB | Python + web framework |
| Data Science (pandas, numpy) | 500MB-1.5GB | Plus MKL, BLAS libraries |
| Machine Learning (PyTorch) | 2-5GB | CUDA, cuDNN, torch |
| Full ML Stack | 5-10GB | PyTorch + TensorFlow + JAX |
| Conda Environment | 1-8GB | Complete isolated Python |

Multiply by 10, 20, 50 projects. You're looking at **10-50GB** of Python environments.

> *"I have just cleaned up multiple machines with disk-space issues due to the fact that conda's cache had become huge and I think something more proactive should be done on conda's side."* — **GitHub conda/anaconda-issues**, May 2015

Multiple machines. Same problem. Conda cache explosion.

**<!-- TODO: INSERT IMAGE - Visual showing multiple Python projects each with .venv folders consuming disk space -->

---

## Why Python Environments Get So Big

### The "Fat Package" Problem

Python packages bundle everything:

```python
# You install:
pip install torch

# You get:
# - PyTorch libraries (500MB)
# - CUDA runtime (1-2GB)
# - cuDNN libraries (500MB)
# - MKL libraries (300MB)
# - ... and more

# Total: 2-4GB for one package
```

> *"I just saw that my conda environment is 13GB in size... they are mostly pytorch / cuda modules."* — **PyTorch Forums**

Machine learning packages are the worst offenders. But they're not alone:

| Package | Size | Why So Big? |
|---------|------|-------------|
| **PyTorch** | 2-4GB | CUDA, cuDNN, MKL |
| **TensorFlow** | 1-3GB | GPU libraries |
| **Pandas** | 100-300MB | MKL, BLAS |
| **NumPy** | 50-150MB | Optimized math libs |
| **Jupyter** | 100-300MB | Web stack, widgets |
| **Conda base** | 1-3GB | Complete Python distro |

### The Duplication Problem

> *"When I run $ du -hs * in ~/.virtualenvs... there are a lot of packages which are common across all the environments and there are few packages which are different. Is there a better way to organize the dependencies so that disk space can be reduced?"* — **GitHub packaging-problems**

Every `.venv` has its own copy of:
- Python standard library
- pip/setuptools/wheel
- Common packages (requests, click, etc.)

10 projects with `requests`? 10 copies of `requests`. **10x duplication**.

### The Conda Cache Explosion

> *"The default configuration for miniconda3 is in the user's $HOME directory where we have a 10G storage limit. Users run into 'Disk quota exceeded' often when using miniconda3 because of package storage and the package cache."* — **GitHub conda/conda issue**, 2024

Conda users in HPC environments hit **10GB quotas** just from package cache.

> *"I have just cleaned up multiple machines with disk-space issues due to the fact that conda's cache had become huge"* — **GitHub conda/anaconda-issues**

The conda package cache (`pkgs/`) keeps every version of every package you've ever downloaded. Forever.

**<!-- TODO: INSERT IMAGE - Diagram showing package duplication across multiple virtualenvs -->

---

## The Hidden Caches: __pycache__ and Beyond

It's not just virtual environments. Python creates caches everywhere:

### __pycache__ Directories

```python
# Your project
my_project/
├── __pycache__/          # Compiled bytecode
├── module_a/
│   └── __pycache__/      # More bytecode
├── module_b/
│   └── __pycache__/      # Even more
└── tests/
    └── __pycache__/      # Test bytecode too
```

Every Python file creates a `__pycache__` directory with `.pyc` files. Small per file, but it adds up:

- 100 Python files → 100 `__pycache__` directories
- Large project → 50-200MB of `__pycache__`
- Multiple projects → 1-5GB of bytecode cache

### .pytest_cache

```bash
# Pytest creates cache
.pytest_cache/
└── v/
    └── cache/
        └── nodeids  # Test node IDs
        └── stepwise  # Stepwise state
        └── ...
```

Not huge, but unnecessary after tests run.

### .mypy_cache

Type checking with mypy? More cache:

```bash
.mypy_cache/
└── 3.11/
    └── ...  # Type cache for every module
```

### pip Cache

Every package pip downloads stays in cache:

```bash
# Location depends on OS
~/Library/Caches/pip/        # macOS
~/.cache/pip/                # Linux
%LocalAppData%\pip\Cache\   # Windows

# Size: 1-10GB easily
```

> *"I use the map to process data, then 300GB dataset becomes 3TB cache, and run out of my device storage."* — **Hugging Face Forums**

ML processing creates cache explosions. But even normal pip usage accumulates.

**<!-- TODO: INSERT IMAGE - File tree showing __pycache__ proliferation in a Python project -->

---

## The Manual Cleanup Trap

You know you should clean up. But Python cleanup is scattered across tools and locations.

### Delete .venv?

```bash
# The naive approach
rm -rf .venv
```

But:
- ❌ You lose the environment context
- ❌ You might delete the wrong one
- ❌ You'll need to recreate it later
- ❌ What about __pycache__, .pytest_cache, etc.?

### Clean __pycache__?

```bash
# Find and delete
find . -type d -name "__pycache__" -exec rm -rf {} +
```

Problems:
- ❌ Slow on large projects
- ❌ No size information
- ❌ Doesn't find .venv
- ❌ Doesn't find pip cache
- ❌ No git protection

### Clean pip Cache?

```bash
pip cache purge
```

But:
- ❌ Only works for pip
- ❌ Doesn't touch conda
- ❌ Doesn't touch poetry
- ❌ Doesn't touch virtualenvs
- ❌ No project cleanup

### The Many Tools Problem

| Cache Type | Tool | Command |
|------------|------|---------|
| pip cache | pip | `pip cache purge` |
| conda cache | conda | `conda clean --all` |
| poetry cache | poetry | `poetry cache clear --all .` |
| uv cache | uv | `uv cache clean` |
| pycache | manual | `find . -name "__pycache__" -delete` |
| .venv | manual | `rm -rf .venv` |

**Six different tools** for one language's cleanup. Insanity.

**<!-- TODO: INSERT IMAGE - Screenshot showing multiple terminal windows with different Python cleanup commands -->

---

## The Real Solution: null-e for Python Developers

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
```

null-e was built to solve exactly this: scattered Python cleanup in one tool.

### What null-e Does Better

| Feature | null-e | Manual Tools |
|---------|--------|--------------|
| **Parallel scanning** | ✅ Multi-threaded | ❌ Single-threaded |
| **All package managers** | ✅ pip, conda, poetry, uv | ❌ Separate tools |
| **All cache types** | ✅ __pycache__, .pytest_cache, etc. | ❌ Miss most |
| **Virtualenv detection** | ✅ .venv, venv, env | ❌ Manual hunting |
| **Git protection** | ✅ Checks git status | ❌ No checks |
| **Size information** | ✅ Shows MB/GB | ❌ No info |

### Find All Python Bloat

```bash
# Scan for all Python projects
null-e ~/projects

# Output:
✓ Found 23 Python projects with 18.3 GB cleanable

   [1] ○ ml-project (2.8 GB) - 4 months stale, 2.5 GB .venv (torch)
   [2] ○ data-analysis (1.2 GB) - 6 months stale, 1.1 GB .venv (pandas)
   [3] ○ web-app (450 MB) - 2 months stale, 400 MB venv (Django)
   ...
```

See exactly what's there. Which packages are consuming space. How old the project is.

**<!-- TODO: INSERT IMAGE - Screenshot of null-e showing Python projects with venv sizes -->

### Find Stale Projects

```bash
# Projects not touched in 180 days
null-e stale ~/projects --days 180

# Safe to clean - you haven't touched them in 6 months
```

Old experiments. Finished projects. Safe to clean.

### Clean Global Python Caches

```bash
# Check all Python caches
null-e caches

# Output:
✓ Found 8 caches with 6.45 GiB total
   [1] 🐍 pip cache          2.34 GiB   pip cache purge
   [2] 🐍 conda packages     3.12 GiB   conda clean --all
   [3] 🐍 poetry cache       890 MiB    poetry cache clear --all .
   [4] 🐍 uv cache           125 MiB    uv cache clean
```

One command. All Python package managers. pip, conda, poetry, uv—all covered.

**<!-- TODO: INSERT IMAGE - Screenshot of null-e caches showing Python-specific caches -->

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

## Python-Specific Cleanup with null-e

### What null-e Finds in Python Projects

| Item | Pattern | Typical Size | Safety |
|------|---------|--------------|--------|
| **Virtualenv** | `.venv/`, `venv/`, `env/` | 200MB-5GB | ○ Low |
| **__pycache__** | `*/__pycache__/` | 10-100MB | ✓ Safe |
| **pytest cache** | `.pytest_cache/` | 1-10MB | ✓ Safe |
| **mypy cache** | `.mypy_cache/` | 5-50MB | ✓ Safe |
| **Build dirs** | `build/`, `dist/`, `*.egg-info/` | 10-100MB | ○ Low |
| **Tox** | `.tox/` | 100MB-1GB | ○ Low |
| **Jupyter** | `.ipynb_checkpoints/` | Varies | ~ Medium |

```bash
null-e ~/projects

# Shows:
✓ Found 23 Python projects with 18.3 GB cleanable
   [1] ○ ml-project (2.8 GB)
       ├── .venv/          2.5 GB
       ├── __pycache__/    45 MB
       ├── .pytest_cache/  12 MB
       └── build/          180 MB
   ...
```

**<!-- TODO: INSERT IMAGE - Terminal output showing detailed Python project breakdown -->

### Cleaning Virtual Environments Safely

null-e knows which virtualenvs are safe to delete:

```bash
null-e ~/projects --clean

# Interactive prompt:
✓ Found 18 virtualenvs with 14.2 GB total

   [1] ○ ml-project/.venv (2.5 GB) - 4 months stale
   [2] ○ data-analysis/venv (1.1 GB) - 6 months stale
   [3] ○ current-project/.venv (400 MB) - 2 days stale ⚠️

Clean which environments? (e.g., 1,2 or all)
> 1,2

✓ Cleaned 2 environments, freed 3.6 GB
```

Stale environments cleaned. Active ones preserved.

### Cleaning __pycache__ in Bulk

```bash
null-e ~/projects --clean

# Or deep sweep for everything
null-e sweep --clean
```

All `__pycache__` directories found and cleaned across all projects.

### Global Cache Cleanup

```bash
# One command, all Python caches
null-e caches --clean

# Output:
✓ Cleaned Python caches
   [1] 🐍 pip cache          2.34 GiB ✓
   [2] 🐍 conda packages     3.12 GiB ✓
   [3] 🐍 poetry cache       890 MiB  ✓

Total freed: 6.35 GiB
```

**<!-- TODO: INSERT IMAGE - Screenshot of cache cleanup results showing space freed -->

---

## Real Results from Real Python Developers

### Case Study: The ML Researcher

> *"I just saw that my conda environment is 13GB in size. I looked up which packages are taking more space and they are mostly pytorch / cuda modules."* — **PyTorch Forums**

13GB single environment. null-e identifies and cleans old ML experiments safely.

### Case Study: The Many-Projects Developer

> *"When I run $ du -hs * in ~/.virtualenvs, it shows that the size of most of them is over 400MB. The total space consumed by virtualenvs is over 6GB."* — **GitHub packaging-problems**

6GB of virtualenvs. null-e finds stale ones, cleans safely.

### Case Study: The HPC User

> *"The default configuration for miniconda3 is in the user's $HOME directory where we have a 10G storage limit. Users run into 'Disk quota exceeded' often."* — **GitHub conda/conda**

10GB quota exceeded. null-e's conda cache cleanup fixes this.

**<!-- TODO: INSERT IMAGE - Before/After comparison showing Python disk space reclaimed -->

---

## The Python Developer's Cleanup Workflow

### Step 1: Scan Everything

```bash
# Find all Python bloat
null-e ~/projects ~/work ~/experiments
```

See the full picture. No surprises.

### Step 2: Identify Stale Projects

```bash
# Find old experiments
null-e stale ~/projects --days 180

# These are safe - you haven't touched them in 6 months
```

### Step 3: Clean Global Caches

```bash
# Clean pip, conda, poetry, uv caches
null-e caches --clean
```

Reclaim 2-10GB instantly.

### Step 4: Clean Safely

```bash
# Clean with full protection
null-e clean ~/projects

# Or clean everything at once
null-e sweep --clean
```

### Step 5: Make It Automatic

```bash
# Add to your shell profile
alias pyclean='null-e caches --clean-all && null-e stale ~/projects --days 90 --clean'

# Run monthly
# Or add to cron:
0 0 1 * * /usr/local/bin/null-e caches --clean-all --force
```

**<!-- TODO: INSERT IMAGE - Workflow diagram: Scan → Identify → Clean → Automate -->

---

## Package Manager Specifics

### pip

```bash
# What null-e finds
~/.cache/pip/ or ~/Library/Caches/pip/

# Clean command
pip cache purge
```

### conda

```bash
# What null-e finds
~/miniconda3/pkgs/ or ~/anaconda3/pkgs/

# Clean command
conda clean --all
```

> *"I have just cleaned up multiple machines with disk-space issues due to the fact that conda's cache had become huge"* — **GitHub conda/anaconda-issues**

### poetry

```bash
# What null-e finds
~/Library/Caches/pypoetry/ or ~/.cache/pypoetry/

# Clean command
poetry cache clear --all .
```

### uv

```bash
# What null-e finds
~/.cache/uv/

# Clean command
uv cache clean
```

### pyenv

```bash
# What null-e finds
~/.pyenv/versions/

# Multiple Python versions installed
# Each 100-200MB
```

null-e handles them all. One tool. All package managers.

---

## Take Back Your Disk Space Today

Don't let .venv and __pycache__ own your machine.

**[Install null-e →](https://github.com/us/null-e)**

```bash
# Install
cargo install null-e

# Scan your Python projects
null-e ~/projects

# Find stale projects (6+ months old)
null-e stale ~/projects --days 180

# Clean safely with git protection
null-e clean ~/projects
```

### What You'll Reclaim

| Category | Typical Savings |
|----------|---------------|
| Stale .venv directories | 5-20 GB |
| __pycache__ files | 1-3 GB |
| pip cache | 2-10 GB |
| conda cache | 3-15 GB |
| poetry/uv cache | 500MB-2GB |
| Build directories | 500MB-2GB |
| **Total** | **12-52 GB** |

That's not just disk space. That's:
- ✅ No more "Disk quota exceeded" errors
- ✅ Faster file operations (fewer small files)
- ✅ Cleaner project directories
- ✅ No more hunting for which .venv to delete
- ✅ Professional pride in a clean machine

> *"Is there a better way to organize the dependencies so that disk space can be reduced?"* — **GitHub packaging-problems**

Yes. It's called null-e.

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
null-e sweep
```

Clean up the Python mess. Reclaim your disk.

```
     .---.
    |o   o|   "Directive: Clean all the .venvs!"
    |  ^  |
    | === |
    `-----'
     /| |\
```

**[View on GitHub →](https://github.com/us/null-e)**

---

### More Python Cleanup Guides

- [How .venv and __pycache__ Are Eating Your Disk Space](/how-venv-pycache-eating-disk-space/)
- [Clean Conda Cache and Reclaim 10GB+](/clean-conda-cache-reclaim-space/)
- [pip vs conda vs poetry: Disk Space Comparison](/pip-conda-poetry-disk-space/)
- [How to Clean PyTorch/TensorFlow Environments](/clean-ml-python-environments/)

**<!-- TODO: INSERT IMAGE - Related posts grid with Python-specific thumbnails -->