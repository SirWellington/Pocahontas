# Role
Act as a Senior Systems Architect specializing in Rust and Tauri applications. You are building "Praetorian," a high-performance, local-first photo library application designed to replace Lightroom Classic's cataloging features while offering modern AI capabilities.

# Tech Stack & Architecture Constraints
1. **Framework:** Tauri v2 (System WebView) + React/TypeScript/Tailwind CSS.
2. **Backend Language:** Rust.
3. **Database:** SQLite (`rusqlite` or `sqlx`). This acts as the "Catalog" file.
4. **Image Processing Engine:** `libvips-sys`. We need a tiered processing pipeline: 
   - Tiny Thumbnail (for grid view)
   - Smart Preview (medium/large resolution for editing/viewing without loading full RAW)
   - Full Resolution (on demand).
5. **AI & GPU Acceleration:**
   - Use **ONNX Runtime** with CUDA/Vulkan execution providers to leverage the user's GPU (Nvidia/AMD) for face detection.
   - Priority: Accuracy over speed for face recognition, but it must run asynchronously in a background thread to avoid UI lag.

# Core Requirements & Features (MVP)

## 1. The "Catalog" System
- **Database Location:** The app should allow the user to select/create a `.praetorian` catalog file (SQLite). This database stores metadata, paths, and face embeddings.
- **Metadata Sync:** While the DB is source of truth, provide an option to export ratings/tags back to XMP sidecars or DNG files (IPTC).
- **Import Workflow:** Ability to scan directories (e.g., SD Card) and index photos into the catalog without moving them.

## 2. Tiered Image Loading Pipeline (The "Smart Preview" Logic)
Implement a background worker that triggers on import:
1. Generate a tiny thumbnail for the grid view.
2. Generate a "Smart Preview" (e.g., 2048px width, compressed JPEG) stored alongside the catalog or in a cache folder. 
3. When a user clicks an image, load the Smart Preview immediately; only fetch the full RAW file if they zoom in deeply or export.

## 3. Face Recognition Engine
- **Model:** Use a high-accuracy model (e.g., YOLOv8-face or RetinaFace) via ONNX Runtime.
- **GPU Support:** Configure Rust to automatically detect and use GPU acceleration if available.
- **Re-indexing:** Add a feature button "Rebuild Face Index" that triggers a full rescan of the library asynchronously.

## 4. AI Upscaling Bridge (Python Integration)
- We need a bridge between Rust and Python for upscaling features.
- Implement a mechanism where Rust can spawn a Python subprocess (or use `pyo3`) to run an external script (e.g., Real-ESRGAN) on selected images.

# Implementation Plan for this Session
Please provide:
1. **Database Schema:** A robust SQLite schema handling `Images`, `Faces`, `People` (face embeddings), and `Folders`.
2. **The Background Worker Architecture:** Show how to use Rust's `tokio` runtime to handle the asynchronous generation of Smart Previews without blocking the Tauri UI thread.
3. **GPU Configuration:** How to set up ONNX Runtime in Cargo.toml to enable CUDA support for face detection.
4. **Python Bridge Skeleton:** A basic example of how the Rust backend would call a Python script to upscale an image and return the result path.

Start by defining the `Cargo.toml` dependencies required for GPU-accelerated AI and libvips, then outline the core database schema.
