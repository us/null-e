---
layout: post
title: "PHP vendor/ Directory Cleanup: Reclaim 5-30GB from Composer Dependencies"
description: "PHP developers lose disk space to vendor/ directories and Composer cache. Learn how to safely clean PHP dependencies, Composer global packages, and old projects. Complete guide for Laravel and Symfony developers."
date: 2024-02-25
author: us
tags: [php, composer, vendor, laravel, symfony, disk-cleanup, dependencies, autoload]
---

[![null-e - Disk Cleanup Tool for Developers](https://img.shields.io/crates/v/null-e.svg)](https://crates.io/crates/null-e)

**[View on GitHub →](https://github.com/us/null-e)**

If you're a PHP developer using Laravel, Symfony, or any modern PHP framework, you know the drill. You run `composer install`, and your project suddenly has a `vendor/` directory that's hundreds of megabytes.

Modern PHP development relies heavily on Composer packages, and those packages add up quickly—especially in full-stack frameworks like Laravel.

A typical Laravel application can easily have **200MB-1GB** of vendor dependencies. Multiple projects? **5-30GB** of PHP vendor directories.

Let's clean it up safely.

---

## The PHP vendor/ Problem

Composer installs all dependencies locally in each project:

| Project Type | vendor/ Size | Package Count | Notes |
|-------------|--------------|---------------|-------|
| Simple PHP script | 10-30MB | 5-10 packages | Few dependencies |
| Slim/Lumen app | 30-100MB | 15-30 packages | Micro framework |
| Laravel app | 150-500MB | 50-100 packages | Full-stack framework |
| Large Laravel app | 500MB-1GB | 100-200 packages | Many integrations |
| Symfony app | 200-600MB | 80-150 packages | Enterprise framework |

20 PHP projects across your machine? **3-20GB** of vendor directories.

**<!-- TODO: INSERT IMAGE - Visual showing PHP projects with vendor/ directories -->

---

## Where PHP Stores Dependencies

### Local vendor/ Directory

Every Composer project creates a vendor directory:

```
my-laravel-app/
├── app/
├── bootstrap/
├── config/
├── vendor/                    # Composer dependencies
│   ├── autoload.php          # Composer autoloader
│   ├── bin/                  # CLI executables
│   ├── composer/             # Composer's own classes
│   ├── laravel/
│   │   └── framework/        # Laravel framework
│   ├── symfony/
│   │   └── ...               # Symfony components
│   ├── doctrine/
│   │   └── ...               # Doctrine ORM
│   └── ... (50-200 more packages)
│       └── vendor-name/
│           └── package-name/
│               ├── src/      # Package source
│               ├── tests/    # Package tests
│               └── composer.json
```

Every package. Every file. Tests, docs, examples—all included.

### Composer Global Packages

```bash
# Global Composer packages
~/.composer/
└── vendor/
    ├── phpunit/
    ├── phpcs/
    ├── laravel/installer/
    └── ...
```

Global tools installed via `composer global require`.

### Composer Cache

```
~/.composer/cache/
└── repo/
    └── https---repo.packagist.org/
        └── provider-*.json   # Package metadata cache
```

Cached repository metadata. Can grow to **1-5GB**.

**<!-- TODO: INSERT IMAGE - File tree showing PHP vendor/ directory structure -->

---

## Why PHP vendor/ Gets So Big

### Laravel's Dependency Tree

A typical Laravel composer.json:

```json
{
    "require": {
        "php": "^8.1",
        "laravel/framework": "^10.0",
        "laravel/sanctum": "^3.0",
        "laravel/socialite": "^5.0",
        "doctrine/dbal": "^3.0",
        "guzzlehttp/guzzle": "^7.0",
        "predis/predis": "^2.0",
        "spatie/laravel-permission": "^5.0",
        "maatwebsite/excel": "^3.1",
        "intervention/image": "^2.7"
    }
}
```

Results in **80-150 packages** including:
- Laravel framework (50+ packages)
- Symfony components (30+ packages)
- Doctrine DBAL
- Guzzle HTTP client
- Laravel-specific packages
- Their dependencies

### Development vs Production

```json
{
    "require-dev": {
        "phpunit/phpunit": "^10.0",
        "nunomaduro/collision": "^7.0",
        "spatie/laravel-ignition": "^2.0",
        "fakerphp/faker": "^1.0",
        "mockery/mockery": "^1.0"
    }
}
```

Development dependencies add **50-100MB more**:
- PHPUnit (with all dependencies)
- Faker
- Debug tools
- Mocking libraries

### Package Files Included

Every Composer package includes everything:

```
package-name/
├── src/              # Source code (needed)
├── tests/            # Test files (rarely needed)
├── docs/             # Documentation (rarely needed)
├── examples/         # Examples (rarely needed)
├── README.md         # (needed?)
├── LICENSE           # (needed?)
└── composer.json     # (needed)
```

Tests, docs, examples—**rarely used but always installed**.

**<!-- TODO: INSERT IMAGE - Size breakdown of a Laravel vendor/ directory -->

---

## The Manual Cleanup Trap

Cleaning PHP vendor directories is manual:

### composer install vs update

```bash
# Clean install (removes old, installs new)
rm -rf vendor/
composer install

# But you lose:
# - Time (re-download everything)
# - Specific versions if lock file changes
```

### composer clear-cache

```bash
# Clear Composer's cache
composer clear-cache
```

What it does:
- ✅ Clears metadata cache
- ❌ Doesn't touch vendor/ directories
- ❌ Doesn't remove packages
- ❌ Minimal space savings

### Manual Deletion

```bash
# Find large vendor directories
find ~ -type d -name "vendor" -exec du -sh {} \;

# Delete manually
rm -rf my-project/vendor

# Then reinstall
composer install
```

Problems:
- ❌ Time-consuming
- ❌ Re-download takes time
- ❌ No size tracking
- ❌ Risk of version changes

**<!-- TODO: INSERT IMAGE - Terminal showing Composer commands -->

---

## The Multiple Projects Problem

| Project | Framework | vendor/ Size | Last Updated |
|---------|-----------|--------------|--------------|
| Client A | Laravel | 450MB | 3 months ago |
| Client B | Laravel | 520MB | 6 months ago |
| Client C | Symfony | 380MB | 2 months ago |
| Side Project | Laravel | 290MB | 1 year ago |
| Old App | CodeIgniter | 120MB | 2 years ago |

**Total: 1.76GB** just for 5 projects.

20 clients? **8-12GB** of vendor directories.

---

## The Real Solution: null-e for PHP

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
```

null-e understands PHP projects and cleans them safely.

### What null-e Does Better

| Feature | null-e | rm -rf | composer |
|---------|--------|--------|----------|
| **Multi-project** | ✅ Scans all | ❌ Manual | ❌ One only |
| **Size info** | ✅ Shows MB/GB | ❌ Manual | ❌ No |
| **Stale detection** | ✅ Old projects | ❌ No | ❌ No |
| **Safety levels** | ✅ ✓ ~ ! | ❌ No | ❌ No |
| **Global cache** | ✅ Included | ❌ No | ⚠️ Partial |
| **Git protection** | ✅ Enabled | ❌ No | ❌ No |

### Find All PHP Bloat

```bash
# Scan for PHP projects
null-e ~/projects

# Output:
✓ Found 18 PHP projects with 12.4 GB cleanable

   Laravel Applications:
   [1] ○ client-portal (520 MB) - vendor/
       ├── Last deploy: 3 months ago
       └── Packages: 89
   
   [2] ○ e-commerce (780 MB) - vendor/
       ├── Last commit: 6 months ago
       └── Packages: 134
   
   [3] ○ api-service (340 MB) - vendor/
       ├── Active development
       └── Packages: 67
   
   Symfony Applications:
   [1] ○ enterprise-app (640 MB) - vendor/
       └── Packages: 112
   
   Composer Global:
   ├── Global vendor/: 180 MB
   └── Cache: 2.1 GB
```

Everything visible. Laravel, Symfony, global packages, cache.

**<!-- TODO: INSERT IMAGE - Screenshot of null-e showing PHP projects -->

### Check Composer Cache

```bash
null-e caches

# Output:
✓ Found 2 caches with 2.28 GB total
   [1] 📦 Composer cache           2.1 GB
   [2] 📦 Composer global vendor   180 MB
```

Composer cache visible. Global packages tracked.

### Find Stale Projects

```bash
# Projects not touched in 180 days
null-e stale ~/projects --days 180

# Safe to clean - you haven't touched them in 6 months
```

Old client projects. Abandoned side projects. Safe to clean.

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

**<!-- TODO: INSERT IMAGE - Screenshot of null-e clean with PHP projects -->

---

## PHP-Specific Cleanup with null-e

### vendor/ Directory Cleaning

```bash
null-e ~/projects --clean

# Interactive:
✓ Found 18 vendor/ directories (12.4 GB)

   [1] ○ client-portal/vendor (520 MB)
       ├── 89 packages
       └── Last composer install: 3 months ago
   
   [2] ○ e-commerce/vendor (780 MB)
       ├── 134 packages
       └── Last composer install: 6 months ago
   
   [3] ○ api-service/vendor (340 MB)
       ├── 67 packages
       └── Active development

Clean which?
> 1,2

⚠️ Note: vendor/ can be recreated with:
   composer install

First recreate will take time to re-download.

Continue? [Y/n]
> Y

✓ Cleaned 2 projects, freed 1.3 GB
```

Clear warnings. Recreation instructions.

### Composer Cache Cleanup

```bash
null-e caches --clean

# Shows:
Clean which caches?
   [1] 📦 Composer cache    2.1 GB
   [2] 📦 Global vendor     180 MB

> 1

⚠️ Cleaning Composer cache will require re-downloading package metadata.
   Next composer install/update will be slower.

Continue? [Y/n]
> Y

✓ Cleaned Composer cache, freed 2.1 GB
```

Reclaim 2-5GB from old package metadata.

### Global Vendor Cleanup

```bash
null-e caches --clean

# Shows:
Composer Global:
   Global vendor: 180 MB
   Packages: phpunit, phpcs, laravel-installer, ...

Clean old global packages?
> Y

⚠️ Note: These are global CLI tools.
   You may need to reinstall tools you use.

✓ Reviewing packages...
   [✓] phpunit (kept - recently used)
   [✓] phpcs (kept - recently used)
   [✓] laravel-installer (kept - recently used)

✓ No unused packages found
```

Smart analysis of global packages.

**<!-- TODO: INSERT IMAGE - Before/After showing PHP cleanup results -->

---

## Real Results from Real PHP Developers

### Case Study: The Laravel Agency

15 Laravel client projects. Average 400MB each. Total: 6GB. null-e cleans 8 old projects.

### Case Study: The Package Cache

Composer cache: 4.2GB of old package metadata. null-e clears 3.8GB safely.

### Case Study: The Framework Updater

Projects with Laravel 8, 9, and 10. Old versions in vendor/. null-e cleans and frees 2GB.

**<!-- TODO: INSERT IMAGE - Testimonials or case study graphics -->

---

## The PHP Developer's Cleanup Workflow

### Step 1: Scan Everything

```bash
# Find all PHP bloat
null-e ~/projects ~/work ~/php-projects
```

Laravel, Symfony, WordPress—all detected.

### Step 2: Identify Stale Projects

```bash
# Find old projects
null-e stale ~/projects --days 180

# Safe to clean - you haven't touched them in 6 months
```

### Step 3: Clean Global Caches

```bash
# Clean Composer cache
null-e caches --clean
```

Reclaim 2-5GB instantly.

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
alias phpclean='null-e caches --clean-all && null-e stale ~/projects --days 90 --clean'

# Run monthly
```

**<!-- TODO: INSERT IMAGE - Workflow diagram: Scan → Identify → Clean → Automate -->

---

## Preventing PHP Storage Bloat

### Use --optimize-autoloader

```bash
composer install --optimize-autoloader
```

Creates classmap for faster loading. Doesn't reduce size, but better performance.

### Use --no-dev in Production

```bash
# Production (CI/CD)
composer install --no-dev --optimize-autoloader
```

Skips 50-100MB of development packages.

### Clean After Deployment

```bash
# On production servers
composer install --no-dev
rm -rf vendor/**/tests/
rm -rf vendor/**/docs/
```

(But use with caution—some packages need test files)

### Use null-e Monthly

```bash
# Monthly maintenance
null-e ~/projects --clean
```

Prevent bloat before it becomes a problem.

**<!-- TODO: INSERT IMAGE - Code snippets showing PHP optimization tips -->

---

## Take Back Your Disk Space Today

Don't let vendor/ directories own your disk.

**[Install null-e →](https://github.com/us/null-e)**

```bash
# Install
cargo install null-e

# Scan your PHP projects
null-e ~/projects

# Find stale projects (6+ months old)
null-e stale ~/projects --days 180

# Clean safely with git protection
null-e clean ~/projects
```

### What You'll Reclaim

| Category | Typical Savings |
|----------|---------------|
| Stale vendor/ directories | 5-20 GB |
| Composer cache | 2-5 GB |
| Global vendor | 100-500 MB |
| Old lock files | 10-50 MB |
| **Total** | **7-25 GB** |

That's not just disk space. That's:
- ✅ Faster composer installs (clean slate)
- ✅ Cleaner project directories
- ✅ No confusion about which vendor/ is current
- ✅ More space for active projects
- ✅ Professional pride in a clean machine

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
null-e sweep
```

Clean up the PHP vendor bloat. Reclaim your disk.

```
     .---.
    |o   o|   "Directive: Clean all the vendor/ directories!"
    |  ^  |
    | === |
    `-----'
     /| |\
```

**[View on GitHub →](https://github.com/us/null-e)**

---

### More PHP Cleanup Guides

- [PHP vendor/ Directory Cleanup Guide](/php-vendor-directory-cleanup/)
- [Composer Cache Management](/composer-cache-management/)
- [Laravel Storage Optimization](/laravel-storage-optimization/)
- [Production vendor/ Optimization](/production-vendor-optimization/)

**<!-- TODO: INSERT IMAGE - Related posts grid with PHP-specific thumbnails -->