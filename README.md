# 🛡️ AI Image Cleaner (`ai-image-cleaner`)

**English** | [Tiếng Việt](README.vi.md)

[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![C2PA](https://img.shields.io/badge/C2PA-Stripped-success.svg)](https://c2pa.org/)
[![Rayon](https://img.shields.io/badge/multi--threaded-Rayon-red.svg)](https://github.com/rayon-rs/rayon)

A high-performance command-line tool (CLI) written in **Rust** designed to sanitize AI-generated images by stripping **C2PA Content Credentials**, **EXIF/XMP/IPTC metadata**, **generation prompt chunks**, disrupting invisible **SynthID watermarks**, and erasing stamped **Gemini sparkle logos** across **PNG, JPEG, and WebP** formats.

Allows you to publish images to major social media platforms (**Facebook, Instagram, Threads, X/Twitter, TikTok, LinkedIn**) without being automatically detected and labeled as **"AI-generated / AI info"**.

---

## 📌 Table of Contents

- [Why AI Image Cleaner?](#-why-ai-image-cleaner)
- [Key Features](#-key-features)
- [How It Works](#-how-it-works)
- [Installation & Build](#-installation--build)
- [Usage Guide](#-usage-guide)
  - [1. Single File Cleaning (Standard Mode)](#1-single-file-cleaning-standard-mode)
  - [2. Erasing Visible Gemini Corner Logos](#2-erasing-visible-gemini-corner-logos---erase-logo)
  - [3. Deep Clean Mode (Disrupting SynthID)](#3-deep-clean-mode-disrupting-synthid---deep)
  - [4. Multi-Threaded Batch Processing](#4-multi-threaded-batch-processing)
  - [5. Dry-Run Inspection](#5-dry-run-inspection---dry-run)
- [CLI Reference Table](#-cli-reference-table)
- [Performance](#-performance)
- [License](#-license)

---

## 🧐 Why AI Image Cleaner?

When images are generated using tools such as **Google Gemini, Midjourney, Stable Diffusion, ComfyUI, DALL-E 3, or Adobe Firefly**, multiple layers of provenance and tracking data are embedded:

1. **C2PA Manifests (Content Credentials):**
   - Cryptographic provenance standard developed by Adobe, Microsoft, Google, Meta, and OpenAI.
   - Embedded as JUMBF boxes in the `caBX` chunk (PNG) or APP11 marker (JPEG).
   - **Meta (Facebook/Instagram) and TikTok scan these manifests directly to apply automated "AI Info" labels.**
2. **Standard Metadata & Prompt Text Chunks:**
   - PNG text chunks (`tEXt`, `zTXt`, `iTXt`) storing full generation prompts, seeds, sampler settings, and workflow JSON (especially from ComfyUI / Automatic1111).
   - EXIF/XMP tags identifying generator software.
3. **Invisible Frequency-Domain Watermarking (Google SynthID / Stable Signature):**
   - Watermarks embedded into spatial pixel distributions and discrete cosine transform (DCT) latents, imperceptible to the human eye but machine-detectable.
4. **Visible Corner Logos:**
   - Semi-transparent 4-pointed sparkle stars stamped in the corner (e.g., Google Gemini web app exports).

`ai-image-cleaner` tackles all four layers using native Rust image-processing pipelines.

---

## ⚡ Key Features

- 🧼 **100% C2PA & Metadata Eradication:** Decodes raw pixel buffers and repacks into clean containers, wiping all `caBX`, `c2pa`, `eXIf`, `tEXt`, `zTXt`, `iTXt`, APP11, and APP13 markers with **zero visual quality loss**.
- ✨ **Visible Logo Eraser (`--erase-logo`):** Automatically detects and seamlessly inpaints the Google Gemini 4-pointed sparkle star in the corner without cropping out the image composition.
- 🔬 **Deep Clean Mode (`--deep`):** Applies micro-perturbations (1–2px boundary crop/resample + alternating micro-dithering) to disrupt frequency-domain latent watermarks (SynthID).
- 🚀 **Rayon Multi-Core Batching:** Scans and processes thousands of images in parallel across all CPU cores.
- 🛡️ **Data Safety:** Never overwrites source files by default (saves to `./cleaned/` or a custom `--output` path); in-place overwrite requires an explicit `--in-place` flag.

---

## 🔬 How It Works

```
[ Input AI Image ]
       │
       ├───> 1. Inspect & Strip Metadata (EXIF, XMP, C2PA caBX, Prompt Chunks)
       │
       ├───> 2. [Optional: --erase-logo] Inpaint visible corner watermarks (Gemini Star)
       │
       ├───> 3. [Optional: --deep] Micro-dithering & DCT Resample to neutralize SynthID
       │
       └───> 4. Repack into Clean Container (Clean PNG / 95% High-Q JPEG / Lossless WebP)
                 └──> [ 100% Clean Image — No Social Media AI Detection ]
```

---

## 🛠️ Installation & Build

Requires [Rust & Cargo](https://rustup.rs/) (version 1.80+):

```bash
# Clone the repository
git clone https://github.com/pmkhang/ai-image-cleaner.git
cd ai-image-cleaner

# Compile optimized release binary
cargo build --release

# The compiled binary is located at: target/release/ai-image-cleaner
```

*(Optional) Install globally to PATH:*
```bash
cp target/release/ai-image-cleaner ~/.local/bin/
```

---

## 📖 Usage Guide

### 1. Single File Cleaning (Standard Mode)
Removes all C2PA, EXIF, and generation prompt metadata while preserving 100% pixel fidelity:

```bash
# Outputs clean image to ./cleaned/my_image.png
ai-image-cleaner my_image.png

# Specify custom output path
ai-image-cleaner my_image.png -o clean.png

# Overwrite in-place
ai-image-cleaner my_image.png --in-place
```

### 2. Erasing Visible Gemini Corner Logos (`--erase-logo`)
If an image from Google Gemini has the visual sparkle star watermark in the bottom-right corner:

```bash
ai-image-cleaner gemini_photo.png --erase-logo -o clean.png
```

### 3. Deep Clean Mode (Disrupting SynthID) (`--deep`)
Neutralizes invisible frequency-domain watermarks in addition to stripping metadata:

```bash
ai-image-cleaner gemini_photo.png --deep --strength medium -o clean.png
```

*Combine both logo erasure and deep invisible watermark disruption:*
```bash
ai-image-cleaner gemini_photo.png --erase-logo --deep -o clean.png
```

### 4. Multi-Threaded Batch Processing
Utilize all CPU cores to clean entire photo folders:

```bash
# Process all images in directory and save to ./output_folder/
ai-image-cleaner ./ai_gallery/ -o ./output_folder/

# Recursively scan subdirectories
ai-image-cleaner ./ai_gallery/ --recursive -o ./output_folder/

# Limit worker threads (e.g., 4 threads)
ai-image-cleaner ./ai_gallery/ --threads 4 -o ./output_folder/
```

### 5. Dry-Run Inspection (`--dry-run`)
Inspect images for embedded AI metadata, C2PA manifests, and prompt tags without writing any files:

```bash
ai-image-cleaner ./ai_gallery/ --dry-run -v
```

---

## 📋 CLI Reference Table

| Option | Short | Default | Description |
| :--- | :---: | :---: | :--- |
| `<INPUT>` | - | *(Required)* | Path to source image file or directory |
| `--output <PATH>` | `-o` | `./cleaned/` | Output file path or destination directory |
| `--in-place` | `-i` | `false` | Overwrite source file(s) in-place (mutually exclusive with `-o`) |
| `--erase-logo` | - | `false` | Detect and seamlessly inpaint visible corner watermarks (Gemini star) |
| `--deep` | `-d` | `false` | Enable micro-perturbation mode to disrupt SynthID watermarks |
| `--strength <LVL>` | - | `medium` | Perturbation intensity: `low`, `medium`, `high` |
| `--recursive` | `-r` | `false` | Recursively scan subdirectories when input is a directory |
| `--threads <N>` | `-t` | CPU Cores | Number of worker threads (Rayon thread pool) |
| `--dry-run` | - | `false` | Inspect and report detected metadata without modifying files |
| `--verbose` | `-v` | `false` | Print detailed logs for each stripped chunk and tag |
| `--help` | `-h` | - | Show full command-line help |

---

## 📊 Performance

Engineered in Rust with zero unnecessary memory copies and multi-core Rayon parallelism:
- **2MB PNG Image:** ~0.05 seconds / image
- **4K 8MB PNG Image:** ~0.65 seconds / image (even with Deep Mode active)
- **Batch of 1,000 images:** Parallelized across all logical CPU cores in seconds.

---

## 📄 License

Distributed under the [MIT License](LICENSE).
Free to use, modify, and distribute for personal and commercial projects.
