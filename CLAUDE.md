# CLAUDE.md - AI Assistant Guide for godot-vlc

## Project Overview

**godot-vlc** is a Godot Engine extension that integrates VLC media playback functionality into Godot 4.3+. It's written in Rust using the godot-rust (gdext) bindings and provides native VLC video/audio playback capabilities for Windows and Linux platforms.

- **Language**: Rust (Edition 2021)
- **Primary Framework**: godot-rust (gdext) 0.3.5
- **Build System**: Cargo
- **Target Platforms**: Windows x86_64, Linux x86_64
- **License**: GNU LGPL v2.1+
- **Current Version**: 1.1.2

## Repository Structure

```
godot-vlc/
├── src/                              # Rust source code (~2090 lines)
│   ├── lib.rs                       # Extension entry point, registers VLCInstance singleton
│   ├── vlc_instance.rs              # VLC instance singleton, logging configuration
│   ├── vlc_media.rs                 # VLCMedia resource class (file/MRL loading)
│   ├── vlc_media_player.rs          # VLCMediaPlayer control node (main player)
│   ├── vlc_media_player/
│   │   ├── internal_audio_stream.rs
│   │   └── internal_audio_stream_playback.rs
│   ├── vlc_track.rs                 # Track information wrapper
│   ├── vlc_track_list.rs            # Track list management
│   └── util.rs                      # Utility functions (GString/CString conversion)
├── thirdparty/vlc/                  # VLC library headers and binaries
│   ├── include/                     # VLC C headers for bindgen
│   └── lib/                         # Platform-specific VLC libraries
│       ├── win-x64/                 # Windows VLC binaries
│       └── linux-x64/               # Linux VLC binaries
├── demo/                            # Test/demo Godot project
│   ├── addons/godot-vlc/           # Plugin installation location
│   │   └── godot_vlc.gdextension   # GDExtension configuration
│   ├── main.tscn                    # Demo scene
│   └── test.mp4                     # Test media file
├── scripts/                         # PowerShell build scripts
│   ├── build_debug.ps1             # Debug build
│   ├── build_release.ps1           # Release build
│   ├── publish.ps1                 # Package for distribution
│   └── setup.ps1                   # Development setup
├── build.rs                         # Build script (bindgen for VLC headers)
├── Cargo.toml                       # Rust dependencies and metadata
├── Cargo.lock                       # Locked dependencies
└── .github/workflows/build.yml      # CI/CD pipeline

```

## Core Architecture

### Extension Lifecycle

The extension follows Godot's GDExtension lifecycle:

1. **Initialization** (src/lib.rs:42-49): On `InitLevel::Scene`, registers `VLCInstance` as a singleton
2. **Cleanup** (src/lib.rs:51-63): On deinit, unregisters and frees the singleton

### Key Components

#### 1. VLCInstance (src/vlc_instance.rs)
- Singleton that owns the libVLC instance
- Configures logging levels via Project Settings (`vlc/log_level`)
- Supports custom VLC arguments via Project Settings (`vlc/arguments`)
- Provides logging callbacks (debug, info, warning, error)
- Accessed globally via `vlc_instance::get()`

#### 2. VLCMedia (src/vlc_media.rs)
- Godot Resource class for media files
- Two loading methods:
  - `load_from_file(path)`: Loads from Godot's virtual filesystem
  - `load_from_mrl(mrl)`: Loads from media resource locator (URLs, streams)
- Supports metadata parsing via `parse_request()`
- Exposes VLC metadata constants (title, artist, duration, etc.)
- Uses callbacks for custom file I/O (src/vlc_media.rs:430-478)

#### 3. VLCMediaPlayer (src/vlc_media_player.rs)
- Godot Control node for video playback
- Key features:
  - Video rendering via callbacks to ImageTexture
  - Audio playback via ring buffer and AudioStreamPlayer
  - Stretch modes (scale, keep aspect, etc.)
  - Chapter/title navigation
  - Track selection (audio, video, subtitles)
  - Playback control (play, pause, stop, seek)
- Emits signals: `openning`, `buffering`, `playing`, `paused`, `stopped`, `forward`, `backward`, `stopping`, `video_frame`

### FFI and Unsafe Code

- VLC bindings generated via bindgen in build.rs
- Extensive use of `unsafe` blocks for VLC C API calls
- Callback functions bridge VLC events to Godot (video_*_callback, audio_*_callback)
- Manual memory management for VLC pointers (released in Drop impls)

## Build System

### Dependencies (Cargo.toml)

```toml
[dependencies]
godot = {version = "0.3.5", features = ["experimental-threads", "register-docs", "api-4-3"]}
printf = "0.1.0"      # For VLC log formatting
ringbuf = "0.4.8"     # Audio buffer

[build-dependencies]
bindgen = "0.72.1"    # Generate VLC bindings
```

### Build Process (build.rs)

1. Detects target platform (Windows/Linux x86_64)
2. Links VLC libraries from `thirdparty/vlc/lib/{platform}`
3. Generates Rust bindings from `thirdparty/vlc/include/vlc/vlc.h`
4. Outputs bindings to `OUT_DIR/vlc_bindings.rs` (included in src/lib.rs:32)

### Build Commands

- **Debug build**: `cargo build` or `./scripts/build_debug.ps1`
- **Release build**: `cargo build --release` or `./scripts/build_release.ps1`
- **Package**: `./scripts/publish.ps1` (creates zip with version from Cargo.toml)

### Platform-Specific Notes

**Windows**:
- Requires LLVM/Clang for bindgen (see .github/workflows/build.yml:48-55)
- Outputs: `godot_vlc.dll`, `godot_vlc.pdb`
- VLC dependencies: `libvlc.dll`, `libvlccore.dll`, `plugins/`

**Linux**:
- Uses system Clang for bindgen
- Outputs: `libgodot_vlc.so`
- VLC dependencies: `libvlc.so.12`, `libvlccore.so.9`, `libidn.so.11`, `vlc/`

## Development Workflow

### Setting Up Development Environment

1. Install Rust (1.75+ recommended for Rust 2021 edition)
2. Install LLVM/Clang (for bindgen):
   - Windows: Use KyleMayes/install-llvm-action or install manually
   - Linux: `sudo apt-get install libclang-dev` (Ubuntu/Debian)
3. Clone repository
4. Run `./scripts/setup.ps1` (if available)
5. Build with `cargo build`

### Testing Changes

1. Build debug version: `cargo build`
2. Copy output to demo project:
   - Windows: Copy `target/debug/godot_vlc.dll` to `demo/addons/godot-vlc/bin/win-x64/godot_vlc_debug.dll`
   - Linux: Copy `target/debug/libgodot_vlc.so` to `demo/addons/godot-vlc/bin/linux-x64/libgodot_vlc_debug.so`
3. Open `demo/` in Godot 4.3+
4. Run `main.tscn` to test playback

### CI/CD Pipeline

GitHub Actions workflow (`.github/workflows/build.yml`):
- Triggers: Push/PR to `master` branch
- Two jobs: `build_linux64`, `build_win64`
- Builds both debug and release versions
- Uploads artifacts for each platform

## Code Conventions

### Rust Style

- **Edition**: Rust 2021
- **Indentation**: 4 spaces (see .editorconfig)
- **Line Endings**: LF (Unix-style)
- **Naming**:
  - GDExtension classes: PascalCase (e.g., `VlcMediaPlayer`)
  - Rust structs: PascalCase (e.g., `VlcMedia`)
  - Functions: snake_case (e.g., `load_from_file`)
  - Constants: SCREAMING_SNAKE_CASE (e.g., `STATE_PLAYING`)

### Godot-Specific Conventions

- **Class Renaming**: Rust structs use internal names (e.g., `VlcMedia`), exposed to Godot with different names via `#[class(rename=VLCMedia)]`
- **Signals**: Declared via `#[signal]` macro (src/vlc_media_player.rs:261-278)
- **Properties**: Use `#[export]` and `#[var(get, set=...)]` macros
- **Constants**: Exposed via `#[constant]` macro, often casting VLC enums to i32

### Error Handling

- VLC errors often return -1, null pointers, or false
- Check return values before using (e.g., src/vlc_media.rs:186-189)
- Use `godot_error!`, `godot_warn!`, `godot_print!` macros for logging
- Validate instances with `is_instance_valid()` before use

### Memory Management

- VLC pointers released in `Drop` implementations (src/vlc_media_player.rs:193-205)
- Box raw pointers for opaque data in callbacks
- Use `Box::into_raw()` to pass Rust data to C callbacks
- Use `Box::from_raw()` to reclaim ownership in cleanup

### Licensing Headers

All source files include LGPL v2.1+ header (see src/lib.rs:1-18)

## Common Tasks for AI Assistants

### Adding a New VLC Feature

1. Find the corresponding libVLC function in VLC documentation
2. The binding is auto-generated in `vlc` module (src/lib.rs:31-33)
3. Add a new method to the appropriate struct (`VlcMedia`, `VlcMediaPlayer`, etc.)
4. Mark it with `#[func]` to expose to Godot
5. Add documentation comments (supports Godot's doc format)
6. Handle unsafe calls and error cases
7. Test in demo project

Example pattern:
```rust
#[func]
fn new_feature(&mut self, param: i32) -> bool {
    unsafe { libvlc_some_function(self.player_ptr, param) }
}
```

### Modifying Build Configuration

- **Add Rust dependency**: Edit `Cargo.toml` dependencies section
- **Add VLC compile flag**: Edit `build.rs` (e.g., modify bindgen builder)
- **Change platform support**: Update `build.rs` target detection (line 24-30)

### Debugging Issues

1. **Enable VLC debug logging**: In Godot, set Project Settings > VLC > Log Level to "Debug"
2. **Check VLC initialization**: Verify `VLCInstance` singleton exists
3. **Inspect media state**: Use `get_state()`, `get_parsed_status()`
4. **Verify file paths**: Godot uses `res://` paths, converted via `GFile::open()`
5. **Review callbacks**: Video/audio callbacks run on VLC threads, use `call_deferred()` for Godot calls

### Updating VLC Version

1. Download new VLC SDK for target platforms
2. Replace files in `thirdparty/vlc/include/` and `thirdparty/vlc/lib/`
3. Rebuild to regenerate bindings
4. Test all features (especially callback signatures)
5. Update version references in documentation

## Integration with Godot

### Using the Extension in Godot

```gdscript
# Load media
var media = VLCMedia.load_from_file("res://video.mp4")
# or
var media = VLCMedia.load_from_mrl("http://example.com/stream.m3u8")

# Create player node
var player = VLCMediaPlayer.new()
add_child(player)
player.media = media
player.autoplay = true

# Connect signals
player.playing.connect(_on_playing)
player.stopped.connect(_on_stopped)

# Control playback
player.play()
player.set_position(0.5, false)  # Seek to 50%
player.pause()
player.stop_async()
```

### Project Settings

- `vlc/log_level` (int): 0=Debug, 1=Info, 2=Warning, 3=Error, 4=Disabled
- `vlc/arguments` (Array[String]): Custom VLC command-line arguments

## Important Files Reference

- **src/lib.rs:42**: GDExtension entry point
- **src/vlc_media_player.rs:209-832**: Main player API and callbacks
- **src/vlc_instance.rs:32**: Global VLC instance getter
- **build.rs:34-41**: Bindgen configuration
- **demo/addons/godot-vlc/godot_vlc.gdextension**: Extension manifest
- **.github/workflows/build.yml**: CI build configuration

## Version History Notes

Recent commits suggest focus on:
- v1.1.2: Current release
- Audio player validation (commit b8027de)
- Arguments setting support (commit bba5ef0)

## Tips for AI Assistants

1. **Always check VLC documentation**: VLC API behavior is documented at videolan.org
2. **Thread safety**: VLC callbacks run on VLC threads, use `call_deferred()` or `call_thread_safe()` for Godot API calls
3. **Resource lifecycle**: Media must be parsed before metadata is available
4. **Platform differences**: Windows uses `.dll`, Linux uses `.so` - maintain both paths
5. **Godot version**: API requires Godot 4.3+ (see godot_vlc.gdextension:3)
6. **FFI safety**: Always validate pointers before dereferencing in unsafe blocks
7. **Build artifacts**: The demo/addons folder is the distribution format, not the root project

## Getting Help

- Godot-rust documentation: https://godot-rust.github.io/
- VLC LibVLC documentation: https://videolan.org/developers/vlc/doc/doxygen/html/group__libvlc.html
- Project issues: Check repository issues for known problems
- Godot GDExtension docs: https://docs.godotengine.org/en/stable/tutorials/scripting/gdextension/

---

*This document was generated for AI assistants working on the godot-vlc project. Keep it updated when making significant architectural changes.*
