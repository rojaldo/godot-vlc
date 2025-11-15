# CLAUDE.md - AI Assistant Development Guide

## Project Overview

**godot-vlc** is a VLC media player extension for Godot Engine 4.3+. It provides VLC-based video and audio playback capabilities through a Rust-based GDExtension that binds to the libVLC library.

- **License**: LGPL v2.1
- **Language**: Rust (2021 edition)
- **Current Version**: 1.1.2
- **Target Platforms**: Windows x86_64, Linux x86_64
- **Godot Version**: 4.3+ (API compatibility: 4.3 minimum)

## Repository Structure

```
godot-vlc/
├── src/                              # Rust source code
│   ├── lib.rs                        # Main library entry point, gdextension registration
│   ├── vlc_instance.rs               # VLC singleton instance manager
│   ├── vlc_media.rs                  # VLCMedia resource class
│   ├── vlc_media_player.rs           # VLCMediaPlayer control node
│   ├── vlc_track.rs                  # Track information wrapper
│   ├── vlc_track_list.rs             # Track list management
│   ├── util.rs                       # Utility functions
│   └── vlc_media_player/             # Media player submodules
│       ├── internal_audio_stream.rs
│       └── internal_audio_stream_playback.rs
├── thirdparty/vlc/                   # VLC library and headers
│   ├── include/                      # libVLC C headers
│   └── lib/                          # Platform-specific VLC libraries
│       ├── linux-x64/
│       └── win-x64/
├── demo/                             # Demo Godot project
│   ├── addons/godot-vlc/             # GDExtension addon files
│   │   ├── godot_vlc.gdextension     # Extension configuration
│   │   ├── format_loader.gd          # Custom resource loader for media files
│   │   └── icons/                    # Editor icons
│   └── main.tscn                     # Demo scene
├── scripts/                          # Build scripts (PowerShell)
│   ├── build_debug.ps1
│   ├── build_release.ps1
│   ├── publish.ps1
│   └── setup.ps1
├── .github/workflows/                # CI/CD workflows
│   └── build.yml                     # GitHub Actions build pipeline
├── Cargo.toml                        # Rust package configuration
├── build.rs                          # Build script (generates VLC bindings)
└── README.md                         # User documentation
```

## Architecture

### Core Components

1. **VLCInstance** (singleton)
   - Manages the global libVLC instance
   - Handles VLC logging configuration
   - Registered as a Godot singleton on Scene init level
   - Configurable via project settings: `vlc/log_level`, `vlc/arguments`

2. **VLCMedia** (Resource)
   - Represents a media file or stream
   - Can load from:
     - Godot resources (`res://`)
     - Filesystem paths (`load_from_file`)
     - Media Resource Locators (`load_from_mrl`)
   - Supports metadata parsing, chapters, and track information

3. **VLCMediaPlayer** (Control node)
   - Main playback control
   - Renders video to TextureRect
   - Handles audio through custom AudioStream
   - Provides playback controls, volume, position, etc.

4. **VLCTrack** and **VLCTrackList**
   - Represent audio/video/subtitle tracks
   - Allow track selection and information retrieval

### Bindings Generation

The project uses `bindgen` to automatically generate Rust FFI bindings from VLC's C headers:
- Triggered during build via `build.rs`
- Bindings are written to `$OUT_DIR/vlc_bindings.rs`
- Included in `lib.rs` via `include!` macro

## Build System

### Prerequisites

- Rust toolchain (2021 edition)
- Cargo
- Clang/LLVM (required for bindgen, especially on Windows)
- PowerShell (for build scripts)

### Build Commands

**Debug Build:**
```bash
./scripts/build_debug.ps1
# or directly:
cargo build
```

**Release Build:**
```bash
./scripts/build_release.ps1
# or directly:
cargo build -r
```

### Build Configuration

- **RUSTFLAGS**: Set to `-Clink-arg=-Wl,-rpath,$ORIGIN` for Linux builds (enables finding VLC libraries)
- **Platform Detection**: `build.rs` detects target platform and sets appropriate library search paths
- **Output**:
  - Linux: `libgodot_vlc.so` / `libgodot_vlc_debug.so`
  - Windows: `godot_vlc.dll` / `godot_vlc_debug.dll` (+ .pdb files)

### Dependencies

**Rust Crates:**
- `godot = 0.3.5` (features: `experimental-threads`, `register-docs`, `api-4-3`)
- `printf = 0.1.0` - For VLC log formatting
- `ringbuf = 0.4.8` - Audio buffer management
- `bindgen = 0.72.1` (build-time)

**Native Libraries:**
- libVLC 3.x (bundled in `thirdparty/vlc/lib/`)
- Platform-specific VLC dependencies (see `demo/addons/godot-vlc/godot_vlc.gdextension`)

## CI/CD

### GitHub Actions Workflow

`.github/workflows/build.yml` runs on:
- Push to `master`
- Pull requests to `master`

**Jobs:**
1. `build_linux64` (Ubuntu 22.04)
   - Builds debug and release
   - Uploads artifacts to `linux64/`

2. `build_win64` (Windows latest)
   - Installs LLVM 11.0 for bindgen
   - Sets LIBCLANG_PATH
   - Builds debug and release
   - Uploads artifacts to `win64/`

## Code Conventions

### Rust Style

- **Edition**: 2021
- **Indentation**: 4 spaces (enforced by `.editorconfig`)
- **Line Endings**: LF
- **Charset**: UTF-8

### File Headers

All source files include LGPL license headers:
```rust
/*
* Copyright (c) 2025 xiSage
*
* This library is free software; you can redistribute it and/or
* modify it under the terms of the GNU Lesser General Public
* License as published by the Free Software Foundation; either
* version 2.1 of the License, or (at your option) any later version.
* ...
*/
```

### Naming Conventions

- **Rust Modules**: Snake_case (e.g., `vlc_media_player.rs`)
- **GDExtension Classes**: PascalCase with VLC prefix (e.g., `VLCMediaPlayer`, `VLCMedia`)
  - Renamed in Godot via `#[class(rename=...)]`
- **Godot Resources**: Use `#[class(base=Resource)]`
- **Godot Nodes**: Use `#[class(base=Control)]` or other node types

### Linter Allowances

The VLC bindings module has specific allowances due to generated code:
```rust
#[allow(
    dead_code,
    non_camel_case_types,
    non_upper_case_globals,
    non_snake_case,
    clippy::upper_case_acronyms,
    unused_imports
)]
mod vlc {
    include!(concat!(env!("OUT_DIR"), "/vlc_bindings.rs"));
}
```

## Key Files to Know

### `src/lib.rs`
- Entry point for the GDExtension
- Registers VLCInstance singleton on `InitLevel::Scene`
- Unregisters and frees singleton on deinit

### `build.rs`
- Platform-specific library path configuration
- Generates VLC bindings using bindgen
- Links against libvlc

### `demo/addons/godot-vlc/godot_vlc.gdextension`
- Configures GDExtension for Godot
- Maps platform/build type to binary paths
- Lists native library dependencies (VLC DLLs/SOs)
- Sets custom icons for VLCMedia and VLCMediaPlayer

### `demo/addons/godot-vlc/format_loader.gd`
- Custom `ResourceFormatLoader` for automatic media file loading
- Recognizes 80+ audio/video extensions
- Returns `VLCMedia` resources when loading recognized media files

## Development Workflows

### Adding New Features

1. **Identify VLC API**: Check `thirdparty/vlc/include/vlc/` headers
2. **Add Rust Wrapper**: Create wrapper in appropriate module (`vlc_media.rs`, `vlc_media_player.rs`, etc.)
3. **Expose to Godot**: Use `#[godot_api]` impl block with `#[func]` attributes
4. **Document**: Use doc comments (rendered in Godot with `register-docs` feature)
5. **Test**: Use demo project to verify functionality
6. **Build**: Run both debug and release builds

### Modifying VLC Bindings

If you need to update VLC headers or change bindgen configuration:
1. Update headers in `thirdparty/vlc/include/`
2. Modify `build.rs` bindgen configuration
3. Run `cargo clean` to force regeneration
4. Run `cargo build`

### Testing

Currently, testing is manual via the demo project:
1. Open `demo/project.godot` in Godot 4.3+
2. Test with `demo/test.mp4` or your own media files
3. Verify playback, controls, and features

### Debugging

**Rust Side:**
- Build debug version: `./scripts/build_debug.ps1`
- Use Godot's debug build
- VLC logs route through `godot_print!`, `godot_warn!`, `godot_error!`
- Configure `vlc/log_level` in project settings (Debug/Info/Warning/Error/Disabled)

**Godot Side:**
- Check Godot console for VLC logs
- Use Godot debugger for GDScript (format_loader.gd)

### Version Bumping

1. Update version in `Cargo.toml`
2. Rebuild (version is compiled into binaries)
3. Commit with version tag (e.g., `v1.1.2`)
4. Push tag for release workflow (if configured)

## Common Pitfalls

### Platform-Specific Issues

- **Windows**: Requires LLVM/Clang for bindgen
  - Set `LIBCLANG_PATH` environment variable
  - Use LLVM 11.0 or compatible version

- **Linux**: Requires proper rpath for finding VLC libraries
  - Build scripts set `-Wl,-rpath,$ORIGIN`
  - Ensure VLC .so files are in correct relative path

### VLC Library Loading

- Libraries must be bundled with the GDExtension
- Paths in `godot_vlc.gdextension` must match actual file layout
- VLC plugins directory is required (not just main libraries)

### Godot API Compatibility

- This extension requires Godot 4.3+ (`api-4-3` feature)
- Uses `experimental-threads` feature for threading support
- `reloadable = true` in .gdextension allows hot-reloading during development

### Memory Safety

- VLC uses raw pointers and manual memory management
- Ensure proper cleanup in Drop implementations
- Use `unsafe` blocks responsibly with clear comments

## Project Settings

The extension adds custom project settings in Godot:

- **vlc/log_level** (int, enum)
  - 0 = Debug, 1 = Info, 2 = Warning, 3 = Error, 4 = Disabled
  - Default: 4 (Error)
  - Requires restart

- **vlc/arguments** (Array[String])
  - Command-line arguments passed to libVLC
  - Default: empty array
  - Requires restart

## Resource Loading

Media files can be loaded in three ways:

1. **Automatic Resource Loading**
   - Place media in `res://` folder
   - Godot auto-loads as VLCMedia via `format_loader.gd`
   - Supports 80+ extensions

2. **From File Path**
   ```gdscript
   var media = VLCMedia.load_from_file("path/to/file.mp4")
   ```

3. **From MRL**
   ```gdscript
   var media = VLCMedia.load_from_mrl("http://example.com/stream")
   ```

## Editor Configuration

- **EditorConfig**: `.editorconfig` enforces style
  - 4 space indentation
  - LF line endings
  - UTF-8 encoding
- **Applies to**: All files (Rust, GDScript, etc.)

## Future Considerations

When working on this codebase, keep in mind:

- VLC library version compatibility (currently 3.x)
- Godot API stability (4.3+ required)
- Cross-platform consistency
- Memory safety in FFI code
- Thread safety (VLC callbacks may run on different threads)
- Error handling (VLC functions often return NULL/error codes)

## Support and Resources

- **VLC Documentation**: https://www.videolan.org/developers/vlc.html
- **Godot-Rust Documentation**: https://godot-rust.github.io/
- **bindgen User Guide**: https://rust-lang.github.io/rust-bindgen/

## Quick Reference

**Build for development:**
```bash
./scripts/build_debug.ps1
```

**Build for release:**
```bash
./scripts/build_release.ps1
```

**Clean build:**
```bash
cargo clean
cargo build
```

**Check code:**
```bash
cargo clippy
cargo fmt --check
```

**View VLC bindings:**
```bash
cargo build
cat target/debug/build/godot-vlc-*/out/vlc_bindings.rs
```

---

*Last updated: 2025-11-15 for version 1.1.2*
