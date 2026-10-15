[README.md](https://github.com/user-attachments/files/33515771/README.md)


# Dataset Manager

A Tauri-powered React application for managing, converting, and augmenting computer vision datasets. Supports YOLO, COCO, Pascal VOC, Label Studio, and TFRecord formats.

## Features

- **Format Conversion** - Convert between YOLO, COCO, Pascal VOC, Label Studio, and TFRecord
- **Dataset Explorer** - Virtualized grid view with thumbnails, search, and class filtering
- **Data Augmentation** - Pipeline-based augmentation (Flip, Rotate, Brightness, Contrast, Noise, Blur)
- **Dataset Mixing** - Combine multiple datasets with class balancing strategies
- **File Organization** - Template-based renaming and class folder structure
- **Train/Val/Test Split** - Stratified splitting with multi-format export
- **Label Studio Export** - One-click export for Label Studio annotation workflow

## Supported Formats

| Format | Read | Write | Structure |
|--------|------|-------|-----------|
| **YOLO** | ✅ | ✅ | `images/` + `labels/*.txt` + `classes.txt` |
| **COCO** | ✅ | ✅ | `annotations.json` + `images/` |
| **Pascal VOC** | ✅ | ✅ | `Annotations/*.xml` + `JPEGImages/` |
| **Label Studio** | ✅ | ✅ | `export.json` (Tasks format) |
| **TFRecord** | ❌ | ✅ | `*.tfrecord` files |

## Folder Structure for YOLO Format

Your dataset folder must follow this structure:

```
your_dataset/
├── images/
│   ├── image1.jpg
│   ├── image2.jpg
│   └── ...
├── labels/
│   ├── image1.txt
│   ├── image2.txt
│   └── ...
└── classes.txt          # Required! One class name per line
```

Example `classes.txt`:
```
person
car
bicycle
```

## Installation

### Prerequisites
- **Node.js** 18+
- **Rust** 1.70+ (install via [rustup.rs](https://rustup.rs/))
- **Tauri CLI**: `cargo install tauri-cli`

### Development Setup

```bash
# Clone and enter directory
cd dataset-manager

# Install frontend dependencies
npm install

# Start development server (frontend + backend)
npm run tauri:dev
```

### Production Build

```bash
# Build frontend + compile Rust backend
npm run tauri:build

# Output installers:
# - src-tauri/target/release/bundle/nsis/dataset-manager_0.1.0_x64-setup.exe
# - src-tauri/target/release/bundle/msi/dataset-manager_0.1.0_x64_en-US.msi
```

## Usage

1. **Launch** the application
2. **Click "New Workspace"** and select your dataset folder (parent folder containing `images/` and `labels/`)
3. **Dataset loads** - explore images in Explorer tab
4. **Use tools** from sidebar:
   - **Converter** - Change format (e.g., YOLO → COCO)
   - **Augment** - Build pipeline → Run → Output to folder
   - **Mixer** - Add multiple datasets → Mix with balancing
   - **Organizer** - Rename files with templates
   - **Splitter** - Train/Val/Test split ratios
   - **Dashboard** - Overview stats & Label Studio export

## Architecture

```
src/
├── pages/              # React pages (Explorer, Converter, Augment, etc.)
├── hooks/              # Custom React hooks (useDataset, useWorkspace)
├── types/              # TypeScript interfaces
├── components/         # Reusable UI components
└── utils/              # Helper functions

src-tauri/
├── src/
│   ├── main.rs         # Tauri entry point
│   ├── commands.rs     # Tauri commands (invoke from frontend)
│   ├── types.rs        # Rust structs matching TypeScript
│   ├── workspace/      # Format detection & scanning
│   ├── formats/        # Format parsers (YOLO, COCO, VOC, etc.)
│   ├── converter/      # Format conversion pipeline
│   ├── organizer/      # File renaming & structuring
│   ├── mixer/          # Dataset mixing
│   ├── augment/        # Augmentation pipeline
│   └── splitter/       # Train/Val/Test splitting
└── Cargo.toml          # Rust dependencies
```

## Key Technologies

- **Frontend**: React 18, TypeScript, Vite, Tailwind CSS, React Router, react-window
- **Backend**: Rust, Tauri 1.5, image crate, serde, rayon
- **Communication**: Tauri IPC (`invoke` from frontend → Rust commands)

## Performance Optimizations

- **Header-only image decoding** - Reads dimensions without full decode (10-50x faster)
- **Parallel YOLO parsing** - Rayon multi-threaded label processing
- **Virtualized grid** - react-window renders only visible thumbnails
- **Debounced search/filter** - 300ms debounce
- **Thumbnail generation** - Base64 encoded 200px previews
- **Pagination** - Backend offset/limit for large datasets
- **Progress events** - Real-time scan progress via Tauri events

## Common Issues

| Issue | Solution |
|-------|----------|
| "Unknown dataset format" | Ensure `classes.txt` exists in YOLO folder |
| White screen in Explorer | Wait for scan to complete; check console for errors |
| Empty output after Augment | Check output folder permissions |
| "Missing required key" errors | Ensure Tauri/Rust versions match; rebuild |

## License

MIT
