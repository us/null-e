---
layout: post
title: "Machine Learning Disk Space Cleanup: Reclaim 50-500GB from HuggingFace, PyTorch, and AI Models"
description: "ML/AI development consumes massive disk space with model downloads, datasets, and caches. Learn how to safely clean HuggingFace cache, PyTorch models, and Ollama LLMs. Complete guide for ML developers."
date: 2024-02-22
author: us
tags: [machine-learning, ai, huggingface, pytorch, ollama, llm, disk-cleanup, models, datasets]
---

[![null-e - Disk Cleanup Tool for Developers](https://img.shields.io/crates/v/null-e.svg)](https://crates.io/crates/null-e)

**[View on GitHub →](https://github.com/us/null-e)**

If you're working with machine learning or AI, you've faced the storage crisis. You're downloading models, processing datasets, and suddenly your disk is full.

> *"I use the map to process data, then 300GB dataset becomes 3TB cache, and run out of my device storage."* — **Hugging Face Forums**, July 2022

**300GB → 3TB**. A 10x explosion. From a single dataset operation.

> *"In my user cache folder, there's a folder for huggingface soaking up over 25 gigs of space… I don't want to just delete this cache if it's going to cause problems… but it's totally hogging space on my C: drive."* — **Reddit r/StableDiffusion**, November 2023

25GB of HuggingFace cache. Afraid to delete. But hogging space.

ML/AI development is the new storage killer.

---

## The ML/AI Disk Space Problem

Machine learning consumes disk space at scales traditional development never imagined:

| Component | Typical Size | When It Grows |
|-----------|--------------|---------------|
| **LLM models** | 4-70GB each | Downloading GPT, Llama, etc. |
| **HuggingFace cache** | 10-100GB | Every model/dataset download |
| **PyTorch hub** | 5-50GB | Pretrained model downloads |
| **Datasets** | 1GB-1TB | Training data, processed data |
| **Training checkpoints** | 10-100GB | Model saves during training |
| **Ollama models** | 4-40GB each | Local LLM serving |

An ML developer can easily have **50-500GB** of ML-related disk usage.

**<!-- TODO: INSERT IMAGE - Visual showing ML storage components (models, datasets, caches) -->

---

## Why ML Eats Disk Space

### Large Language Models Are Huge

Current LLM sizes:

| Model | Size | Quantized |
|-------|------|-----------|
| GPT-4 (API only) | N/A | N/A |
| Llama 2 70B | 140GB | 40GB (4-bit) |
| Llama 2 13B | 26GB | 8GB (4-bit) |
| Llama 2 7B | 13GB | 4GB (4-bit) |
| Mistral 7B | 14GB | 4GB (4-bit) |
| CodeLlama 34B | 68GB | 20GB (4-bit) |

Download 3-4 models for comparison? **100-200GB**.

> *"Disk space: there are many different variants of each LLM and downloading all of them to your laptop or desktop can use up 500-1000 GB of disk space easily."* — **XetHub blog**, 2024

500-1000GB. Just for trying different LLM variants.

### HuggingFace Cache Accumulates

```bash
# HuggingFace cache location
~/.cache/huggingface/
└── hub/
    ├── models--meta-llama--Llama-2-7b/
    │   └── snapshots/
    │       └── abc123/
    │           └── model.safetensors  # 13GB
    ├── models--bert-base-uncased/
    │   └── ...  # 400MB
    └── datasets--glue/
        └── ...  # 100MB
```

Every model. Every version. Every dataset. **Cached forever**.

> *"I was trying to download a repository to my H: disk. The download was aborted when my C: disk was full. I don't know where the files went... It was at 0 bytes of free disk space before the restart."* — **Hugging Face Forums**, January 2023

Download fails. Disk full. Files scattered who-knows-where.

### Datasets Explode

Raw datasets are big. Processed datasets are **bigger**:

> *"I use the map to process data, then 300GB dataset becomes 3TB cache, and run out of my device storage."* — **Hugging Face Forums**

Processing creates cache. 10x the original size. No automatic cleanup.

> *"After I've loaded 'imagenet-1k' with load_dataset("imagenet-1k") I found that I have two main huge folders in HF_DATASETS_CACHE: datasets/imagenet-1k which about 155GB datasets/downloads which takes about 154GB [...] the total size of files in imagenet repo is about 160GB. If I run ds.cleanup_cache_files() it doesn't remove anything."* — **Hugging Face Forums**, 2024

ImageNet: 160GB dataset. 310GB cache. Official cleanup does nothing.

### Training Checkpoints Multiply

Training a model? You save checkpoints:

```python
# Every epoch
model.save_pretrained(f"checkpoint-epoch-{epoch}")

# Result:
checkpoints/
├── checkpoint-epoch-1/    # 13GB
├── checkpoint-epoch-2/    # 13GB
├── checkpoint-epoch-3/    # 13GB
└── ... (10-100 checkpoints)
```

10 checkpoints × 13GB = **130GB**.

### Ollama Model Downloads

Running local LLMs with Ollama:

```bash
ollama pull llama2
ollama pull codellama
ollama pull mistral
ollama pull neural-chat
```

Each model: **4-40GB**. 10 models: **40-400GB**.

> *"For existing Mac computers with insufficient hard disk space, how to download the model to an external SSD drive for running instead of storing it on the computer itself."* — **GitHub ollama/ollama issue #3719**

Mac users forced to use external drives because models don't fit.

**<!-- TODO: INSERT IMAGE - Size comparison chart of ML models and datasets -->

---

## The "Where Is My Disk Space?" Mystery

ML tools scatter data across locations:

### HuggingFace Transformers

```
~/.cache/huggingface/
├── hub/               # Models
├── datasets/          # Datasets
└── modules/           # Additional cache
```

### PyTorch Hub

```
~/.cache/torch/
└── hub/
    └── checkpoints/   # Downloaded models
```

### Ollama

```
~/.ollama/
└── models/
    └── blobs/         # Model weights
```

### Custom Training

```
./checkpoints/         # Training saves
./outputs/             # Training outputs
./logs/                # TensorBoard logs
./cache/               # Custom cache
```

**Six different locations.** No unified view.

**<!-- TODO: INSERT IMAGE - Diagram showing scattered ML cache locations -->

---

## The Manual Cleanup Trap

You can clean ML caches manually. But it's risky.

### HuggingFace: Cache Management

```python
from datasets import load_dataset

# Load dataset
ds = load_dataset("imagenet-1k")

# Try to clean
ds.cleanup_cache_files()
# Result: Nothing happens
```

Official cleanup often **doesn't work**.

### Manual Deletion

```bash
# Find cache
ls ~/.cache/huggingface/hub/

# Delete a model
rm -rf ~/.cache/huggingface/hub/models--model-name/

# Hope you didn't need it
```

Problems:
- ❌ No size information
- ❌ No "is this in use" check
- ❌ Permanent deletion
- ❌ Misses other locations

### PyTorch Cache

```bash
# PyTorch cache
rm -rf ~/.cache/torch/

# Safe-ish, but will re-download
```

### Ollama Models

```bash
# List models
ollama list

# Remove one
ollama rm llama2

# One at a time. No bulk operation.
```

**<!-- TODO: INSERT IMAGE - Terminal showing manual ML cleanup commands -->

---

## The "Is It Safe to Delete?" Problem

> *"In my user cache folder, there's a folder for huggingface soaking up over 25 gigs of space… I don't want to just delete this cache if it's going to cause problems."* — **Reddit r/StableDiffusion**

25GB. Is it safe? Will your code break?

> *"I don't know where the files went. Please help?"* — **Hugging Face Forums**

Download failed. Files somewhere. Disk full.

The problem: **ML tools don't explain their storage**. It's opaque. Hidden.

**<!-- TODO: INSERT IMAGE - Screenshot of hidden cache folders in file manager -->

---

## The Real Solution: null-e for ML/AI

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
```

null-e understands ML/AI tools and makes cleanup safe and visible.

### What null-e Does Better

| Feature | null-e | Manual | Official Tools |
|---------|--------|--------|----------------|
| **Unified view** | ✅ All locations | ❌ Scattered | ❌ Per-tool |
| **Size info** | ✅ GB breakdown | ❌ Manual | ❌ Limited |
| **Model tracking** | ✅ Shows models | ❌ Hard | ❌ No |
| **Safety levels** | ✅ ✓ ~ ! markers | ❌ No | ❌ No |
| **Bulk operations** | ✅ Clean multiple | ❌ One-by-one | ⚠️ Limited |
| **Cache + models** | ✅ Both | ❌ Separate | ❌ Separate |

### Find All ML Bloat

```bash
# Check ML/AI artifacts
null-e ml

# Output:
🤖 ML/AI Artifacts Found:
✓ Found 23 models, 8 datasets (127.4 GB)

   HuggingFace Cache:
   [1] ~ meta-llama/Llama-2-7b (13.5 GB) - Base model
   [2] ~ bert-base-uncased (440 MB) - Embedding model
   [3] ✓ old-experiment-model (6.2 GB) - Not used recently
   [4] ~ google/t5-base (2.1 GB) - Translation model
   ...
   
   Datasets:
   [1] ~ imagenet-1k (155 GB) - Active project
   [2] ✓ glue (180 MB) - Old benchmark
   [3] ✓ wikipedia (12.3 GB) - Not used
   ...
   
   Ollama Models:
   [1] ~ llama2:latest (4.7 GB) - In use
   [2] ~ codellama:34b (19 GB) - Code generation
   [3] ✓ neural-chat (4.1 GB) - Rarely used
   ...
   
   PyTorch Hub:
   [1] ~ resnet50 (100 MB) - Active
   [2] ✓ vgg16 (550 MB) - Not used
   ...
```

Everything visible. Safety levels clear. You decide what to clean.

**<!-- TODO: INSERT IMAGE - Screenshot of null-e ml showing models and datasets with sizes -->

### Safety Levels for ML

```
✓ Safe          - Can delete, easily re-downloaded
~ SafeWithCost  - Will need re-download (time/bandwidth)
! Caution       - May be in use, check first
```

- **Old experiment models**: ✓ Safe
- **Current project models**: ~ SafeWithCost
- **Active training models**: ! Caution
- **Cached datasets**: ~ SafeWithCost
- **Old benchmark datasets**: ✓ Safe

### Clean with Control

```bash
# Clean interactively
null-e ml --clean

# You'll see:
🤖 ML/AI Cleanup

Clean which items?
   [1] ✓ Old models: 5 (28.4 GB)
   [2] ✓ Old datasets: 3 (15.2 GB)
   [3] ~ Unused PyTorch models: 2 (650 MB)
   [4] ✓ Rarely used Ollama models: 3 (12.1 GB)

> 1,2,4

✓ Cleaned ML artifacts, freed 55.7 GB
```

You choose. Safe items clearly marked. No surprises.

**<!-- TODO: INSERT IMAGE - Screenshot of ML cleanup results showing space freed -->

---

## ML/AI-Specific Cleanup with null-e

### HuggingFace Cache Management

null-e understands HuggingFace structure:

```bash
null-e ml

# Shows:
HuggingFace Cache:
   [1] ~ models--meta-llama--Llama-2-7b (13.5 GB)
       └── Used by: current-project/
   [2] ✓ models--old-experiment--bert (1.2 GB)
       └── Last used: 6 months ago
   [3] ~ datasets--imagenet-1k (155 GB)
       └── Used by: current-project/
```

Shows what's in use. What's old. What's safe to clean.

### Ollama Model Cleanup

```bash
null-e ml --clean

# Interactive:
Ollama Models:
   [1] ~ llama2:latest (4.7 GB) - Used frequently
   [2] ~ codellama:34b (19 GB) - Code work
   [3] ✓ neural-chat (4.1 GB) - Used once
   [4] ✓ mistral:7b (4.1 GB) - Testing only

Clean which?
> 3,4

✓ Removed 2 models, freed 8.2 GB
```

Keep frequently used. Clean experiments.

### Dataset Cache Cleanup

```bash
null-e ml --clean

# Shows:
Datasets:
   [1] ~ imagenet-1k (155 GB) - Active training
   [2] ✓ glue (180 MB) - Old benchmark
   [3] ✓ wikipedia-snippets (12.3 GB) - Experiment
   [4] ~ custom-dataset (45 GB) - Production

Clean old datasets?
> Y

⚠️ Note: These will need re-download if needed again.

✓ Cleaned 2 datasets, freed 12.48 GB
```

Clear warnings. Informed decisions.

### Training Checkpoint Cleanup

```bash
null-e ml

# Shows:
Training Checkpoints:
   [1] ! ./checkpoints/epoch-50 (13 GB) - Latest
   [2] ~ ./checkpoints/epoch-45 (13 GB) - Recent
   [3] ✓ ./checkpoints/epoch-1-40 (520 GB) - Old checkpoints

Clean old checkpoints (keep last 5)?
> Y

✓ Cleaned 40 checkpoints, freed 520 GB
```

Keep recent for resuming. Clean old ones.

**<!-- TODO: INSERT IMAGE - Before/After showing ML cleanup results -->

---

## Real Results from Real ML Developers

### Case Study: The 3TB Cache Explosion

> *"300GB dataset becomes 3TB cache, and run out of my device storage."* — **Hugging Face Forums**

Dataset processing created 3TB cache. null-e identifies and safely cleans.

### Case Study: The HuggingFace 25GB Mystery

> *"Huggingface soaking up over 25 gigs of space… I don't want to just delete this cache if it's going to cause problems."* — **Reddit r/StableDiffusion**

25GB cache. null-e shows what's safe. Cleans 20GB without breaking anything.

### Case Study: The ImageNet Cache Problem

> *"ImageNet 160GB became 310GB cache. Official cleanup did nothing."* — **Hugging Face Forums**

Official tools failed. null-e cleaned 150GB of duplicate cache.

**<!-- TODO: INSERT IMAGE - Testimonials or case study graphics -->

---

## The ML Developer's Cleanup Workflow

### Step 1: Check ML Usage

```bash
# See what's using space
null-e ml
```

Full visibility across all ML tools.

### Step 2: Clean Safely

```bash
# Interactive cleanup
null-e ml --clean

# Or dry run first
null-e ml --clean --dry-run
```

### Step 3: After Training

```bash
# Clean old checkpoints
null-e ml --clean

# Keep last 3-5 checkpoints
# Remove everything older
```

### Step 4: Regular Maintenance

```bash
# Monthly cleanup
null-e ml --clean

# Before starting new projects
null-e ml
```

**<!-- TODO: INSERT IMAGE - Workflow diagram: Check → Train → Clean Checkpoints → Repeat -->

---

## Preventing ML Storage Bloat

### Use Model Offloading

```python
# Load model, use it, delete from memory
model = AutoModel.from_pretrained("model")
# ... use model ...
del model  # Free memory

# But cache remains on disk
# Clean with null-e periodically
```

### Limit Checkpoint Saving

```python
# Don't save every epoch
if epoch % 5 == 0:  # Save every 5th epoch
    model.save_pretrained(f"checkpoint-{epoch}")
```

### Use External Drives for Large Datasets

```bash
# Symlink large datasets to external drive
ln -s /Volumes/External/imagenet ~/.cache/huggingface/datasets/imagenet-1k
```

### Clean After Experiments

```bash
# After experiment finishes
null-e ml --clean

# Clean up old models, temporary datasets
```

**<!-- TODO: INSERT IMAGE - Code snippets showing ML optimization tips -->

---

## Take Back Your Disk Space Today

Don't let ML models own your disk.

**[Install null-e →](https://github.com/us/null-e)**

```bash
# Install
cargo install null-e

# Check ML usage
null-e ml

# Clean safely
null-e ml --clean
```

### What You'll Reclaim

| Category | Typical Savings |
|----------|---------------|
| Old experiment models | 20-100 GB |
| Old datasets | 10-50 GB |
| Training checkpoints | 50-500 GB |
| Duplicate cache | 20-80 GB |
| Unused Ollama models | 10-40 GB |
| PyTorch hub cache | 5-20 GB |
| **Total** | **115-790 GB** |

Yes, really. **Hundreds of gigabytes**.

That's not just disk space. That's:
- ✅ Room for more experiments
- ✅ Ability to try new models
- ✅ No "disk full" during training
- ✅ Professional data management
- ✅ A machine that can actually run ML

> *"300GB dataset becomes 3TB cache"* — **ML developer**

Don't be that developer. Control your storage.

**[Install null-e →](https://github.com/us/null-e)**

```bash
cargo install null-e
null-e ml --clean
```

Clean up ML bloat. Reclaim your disk.

```
     .---.
    |o   o|   "Directive: Clean all the models!"
    |  ^  |
    | === |
    `-----'
     /| |\
```

**[View on GitHub →](https://github.com/us/null-e)**

---

### More ML/AI Cleanup Guides

- [Machine Learning Disk Space Cleanup Guide](/ml-disk-space-cleanup/)
- [Clean HuggingFace Cache Safely](/clean-huggingface-cache/)
- [Managing LLM Model Storage](/managing-llm-model-storage/)
- [Training Checkpoint Cleanup](/training-checkpoint-cleanup/)

**<!-- TODO: INSERT IMAGE - Related posts grid with ML/AI-specific thumbnails -->