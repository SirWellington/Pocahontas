# Phase 1 Prompt
Prompt: "I am going to build this in stages. For now, just understand the architecture and do not generate code yet."

# Phase 2 Prompt - The Skeleton Setup
Prompt: "Initialize the Tauri v2 + Rust + React project structure. Configure `Cargo.toml` with dependencies for SQLite, libvips, and ONNX Runtime (with CUDA support). Create the basic database schema but don't implement the full business logic yet. At each phase of the project make sure the app is buildable and compiles successfully."

# Phase 3 Prompt - The UI Shell
Prompt: "Create a React layout using Tailwind CSS that mimics Lightroom's dark mode interface (Sidebar, Main Grid, Right Panel). Connect it to Tauri but use mock data for now."

# Phase 4 Prompts - Feature by Feature work

## Iteration A
Prompt: "Implement the Rust backend logic to scan a directory and save paths to SQLite."

## Iteration B
Prompt: "Implement the libvips background worker to generate thumbnails and smart previews."

## Iteration C
Prompt: "Integrate ONNX Runtime for face detection using GPU acceleration."

---

# Full Project Requirements

## Role
Act as a Senior Systems Architect specializing in Rust and Tauri applications. You are building "Praetorian," a high-performance, local-first photo library application designed to replace Lightroom Classic's cataloging features while offering modern AI capabilities.

## Tech Stack & Architecture Constraints
1. **Framework:** Tauri v2 (System WebView) + React/TypeScript/Tailwind CSS.
2. **Backend Language:** Rust.
3. **Database:** SQLite (`rusqlite` or `sqlx`). This acts as the "Catalog" file (`.praetorian`).
4. **Image Processing Engine:** `libvips-sys`. We need a tiered processing pipeline: 
   - Tiny Thumbnail (for grid view)
   - Smart Preview (medium/large resolution for editing/viewing without loading full RAW)
   - Full Resolution (on demand).
5. **Metadata Parsing:** Use a robust Rust library (e.g., `exiftool-rs` or `image-exif`) to read detailed camera data.
6. **AI & GPU Acceleration:**
   - Use **ONNX Runtime** with CUDA/Vulkan execution providers to leverage the user's GPU (Nvidia/AMD) for face detection.
   - Priority: Accuracy over speed for face recognition, but it must run asynchronously in a background thread to avoid UI lag.

## Core Requirements & Features (MVP)

### 1. The "Catalog" System
- **Database Location:** The app should allow the user to select/create a `.praetorian` catalog file (SQLite). This database stores metadata, paths, and face embeddings.
- **Metadata Sync:** While the DB is source of truth, provide an option to export ratings/tags back to XMP sidecars or DNG files (IPTC).
- **Import Workflow:** Ability to scan directories (e.g., SD Card) and index photos into the catalog without moving them.

### 2. Tiered Image Loading Pipeline (The "Smart Preview" Logic)
Implement a background worker that triggers on import:
1. Generate a tiny thumbnail for the grid view.
2. Generate a "Smart Preview" (e.g., 2048px width, compressed JPEG) stored alongside the catalog or in a cache folder. 
3. When a user clicks an image, load the Smart Preview immediately; only fetch the full RAW file if they zoom in deeply or export.

### 3. Photo Details & EXIF Metadata View
- **Metadata Panel:** A dedicated UI panel (similar to Lightroom's right-hand sidebar) that displays detailed info for selected images.
- **Data Points:** Must display Camera Make/Model, Lens Model, Date/Time Original, ISO, Aperture, Shutter Speed, Focal Length, and GPS Location (if available).
- **Indexing:** Store this EXIF data in the SQLite catalog to allow fast filtering later (e.g., "Show all photos taken with Canon 5D").

### 4. Face Recognition Engine
- **Model:** Use a high-accuracy model (e.g., YOLOv8-face or RetinaFace) via ONNX Runtime.
- **GPU Support:** Configure Rust to automatically detect and use GPU acceleration if available.
- **Re-indexing:** Add a feature button "Rebuild Face Index" that triggers a full rescan of the library asynchronously.

### 5. AI Upscaling Bridge (Python Integration)
- We need a bridge between Rust and Python for upscaling features.
- Implement a mechanism where Rust can spawn a Python subprocess (or use `pyo3`) to run an external script (e.g., Real-ESRGAN) on selected images.

## UI/UX Guidelines (Lightroom Cloud Style)
- **Aesthetic:** Clean, dark-mode default, minimal chrome, focus on the image grid. Look to ./design/mockups/00000-4162059048.png to base the look and feel on.
- **Performance:** The main gallery must use "Virtual Scrolling" (windowing) to render thousands of thumbnails without lagging the main thread.
- **Responsiveness:** UI should not freeze during heavy background tasks (thumbnailing or face detection).

## Implementation Plan for this Session
Please provide:
1. **Database Schema:** A robust SQLite schema handling `Images`, `Faces`, `People` (face embeddings), and `Folders`. Ensure it includes columns for all major EXIF tags mentioned above.
2. **The Background Worker Architecture:** Show how to use Rust's `tokio` runtime to handle the asynchronous generation of Smart Previews without blocking the Tauri UI thread.
3. **GPU Configuration:** How to set up ONNX Runtime in Cargo.toml to enable CUDA support for face detection.
4. **Python Bridge Skeleton:** A basic example of how the Rust backend would call a Python script to upscale an image and return the result path.

Start by defining the `Cargo.toml` dependencies required for GPU-accelerated AI and libvips, then outline the core database schema.


---
 
# Design Mockups

```
Create a high-fidelity desktop application UI mockup for a professional photo management application named "Praetorian."

Design language:
- Modern, premium creative software
- Dark mode
- Inspired by Lightroom Cloud, Capture One, Arc Browser, and Linear
- Matte charcoal interface (#181818 to #222222)
- Soft elevation with subtle shadows
- 8px spacing system
- Rounded corners (8px)
- Thin separators instead of heavy borders
- Clean typography similar to Inter or SF Pro
- Pixel-perfect alignment
- Production-quality Figma design
- Extremely polished UX
- No futuristic or glass-heavy effects

Window:
- Native desktop application
- 16:10 aspect ratio
- Large 1440p workspace
- macOS-style window chrome
- Professional creative application

Layout:
Left Sidebar
- Library
- Collections
- Favorites
- Recent Imports
- External Drives
- Folder tree
- Smart Albums
- Collapsible sections
- Clean monochrome icons

Center Workspace
- Primary content area
- Dense but elegant layout
- Excellent spacing
- Strong visual hierarchy

Right Sidebar
- Context-sensitive inspector
- Metadata
- EXIF
- Camera
- Lens
- Histogram
- Keywords
- Rating
- Color labels

The interface should feel like software used by professional photographers managing hundreds of thousands of RAW images.

Generate the "Import Photos" dialog.

The dialog should appear centered over the library.

Dialog layout:

Left Column
- Connected devices
- SD cards
- External SSDs
- NAS locations
- Local folders
- Recently used sources

Center
- Thumbnail preview grid
- 30–50 photo thumbnails
- Portrait photography
- Every thumbnail is unique
- Mix of RAW and JPEG indicators
- Multi-select checkboxes
- File names
- Capture dates
- File sizes

Top Toolbar
- Breadcrumb navigation
- Search
- Filter
- Sort
- Select All
- Number of selected photos

Right Panel
Import Options

Destination
- Folder picker
- Create dated folder
- Custom folder name

File Handling
- Copy
- Move
- Leave in place

Renaming
- Rename using template
- Sequence numbering

Metadata
- Apply copyright
- Apply keywords
- Apply preset
- Apply star rating

AI Options
- Generate Smart Keywords
- Detect Faces
- Detect Duplicate Photos
- Estimate Image Quality
- Build Search Index

Bottom Bar
- Selected item count
- Total storage required
- Cancel button
- Large blue "Import 248 Photos" button

Visual Details
- Beautiful thumbnail photography
- Realistic icons
- Crisp typography
- Realistic spacing
- Professional UI polish
- Subtle hover states
- Soft blue accent color
- Clean monochrome iconography
- Consistent design system

The mockup should be indistinguishable from a real shipping desktop application screenshot. No placeholder boxes, no lorem ipsum, no wireframe styling.
```