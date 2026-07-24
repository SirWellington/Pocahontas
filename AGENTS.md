# Praetorian AI Agent Instructions

These instructions help AI coding agents understand the workspace structure, build commands, and the current model/face-detection architecture.

## Key points

- This is a Tauri desktop app with a React frontend in `src/` and a Rust backend in `src-tauri/`.
- Frontend UI calls backend commands through `src/services/catalog.ts` and state is managed in `src/hooks/useCatalog.ts`.
- The Rust backend exposes catalog, image, face, and Python-upscaling commands in `src-tauri/src/lib.rs`.
- The catalog engine is implemented in `src-tauri/src/catalog/manager.rs` with SQLite and image metadata handling.
- Face detection is implemented under `src-tauri/src/face/` using ONNX Runtime and a RetinaFace model.
- The Python bridge is implemented under `src-tauri/src/python/` and is used for external upscaling workflows.

## Model / AI conventions

- Model code is primarily in Rust; do not move face-detection logic into the React frontend.
- The face detection flow is feature-gated with `#[cfg(feature = "face-detection")]` and initializes `retinaface.onnx` in `src-tauri/src/lib.rs`.
- Use the existing Tauri command patterns when adding or changing model features: backend command -> `src/services/catalog.ts` invoke -> frontend hook/component.
- Prefer GPU execution providers for ONNX Runtime when available, but maintain fallback behavior.

## Build and run commands

- `npm install` to install the frontend dependencies.
- `npm run dev` to start Vite in development.
- `npm run build` to build the frontend (`tsc && vite build`).
- `npm run tauri` to run the Tauri app.
- `npm run tauri build` to build the desktop application in `src-tauri/target/release`.

## Documentation links

- Project overview: [README.md](README.md)
- AI / model requirements: [REQUIREMENTS.md](REQUIREMENTS.md)
- Initial Prompt: [PROMPT.md](PROMPT.md)

## Guidance for AI agents

- Keep UI and backend responsibilities distinct.
- Preserve the existing Tauri command and IPC structure.
- When updating face or model-related behavior, check both Rust backend and TypeScript frontend call sites.
- Avoid duplicating README or requirements content; link to those docs instead.
