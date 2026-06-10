# Changelog

## [2026.6.10] - 2026-06-10

### Changed
- **4.2 Path Modernization**: Updated path imports to align with the `library` 4.2 restructured API (using simplified flat namespaces `apps` and `toolkit`).
- **AppData Directory Realignment**: Moved user configuration, database, and log files into a nested %APPDATA%\local76\app\trance structure to organize the ecosystem's configuration space.
- **Repository Rename**: Renamed repository and local directory to app-trance for cleaner ecosystem taxonomy.

## [2026.6.9] - 2026-06-09

### Renamed
- **Project rename**: `trance` was previously `trance-App` / `rIdle`. The Cargo package name, binary name, file paths, registry keys, AppData paths, and docs are now lowercase `trance`. Behavior and features are unchanged.

### Refactored
- **App Blueprint alignment**: Re-architected directory and module tree to standard App layout. Renamed `src/ui/panels.rs` to `src/ui/widgets.rs`. Created `src/backend/` directory, moving `src/runner.rs` to `src/backend/mod.rs`, `src/preview.rs` to `src/backend/preview.rs`, the Windows and mock screensaver modules to `src/backend/saver/`, and the downloader modules to `src/backend/downloader/`.

### Changed
- README rewritten in the new register: screensaver picker feature list, install matrix, CLI flags, configuration, build instructions, license.
- Drop the legacy "r*" and "Local freedom" branding throughout.
- Drop the per-repo `rApps` umbrella and `build_all.ps1` from this repo; build orchestration lives in [`toolkit`](https://github.com/local76/toolkit).
- The `registry.json` entries now reference lowercase scene names (`glyphs`, `flame`, `cosmos`, etc.) and the new GitHub release URLs for [`screensavers`](https://github.com/local76/screensavers).

## [2.6.6] - 2026-06-08

### Refactored
- Refactored monolithic `src/app.rs` into modular sub-files under `src/app/` (`mod.rs`, `actions.rs`, `keys.rs`, `cycle.rs`), keeping all source files under 500 lines.
- Refactored monolithic `src/ui.rs` into modular sub-files under `src/ui/` (`mod.rs`, `panels.rs`, `utils.rs`), keeping all source files under 500 lines.
- Extracted Win32 API declarations from `src/main.rs` into `src/win32.rs`.
- Resolved all compiler and Clippy warnings across the codebase.
- **Renamed project** from `trance` to `trance-App`. The GitHub repository, Cargo package name, binary name, and all user-facing labels now use the `-App` suffix to make the program's role as a terminal user interface explicit (matching `template-App`).
  - Repository: `local76/trance` → `local76/trance-App`
  - Crate/binary: `trance` → `trance-App`
  - Config file: `%APPDATA%\trance\config.yaml` → `%APPDATA%\trance-App\config.yaml`
  - Downloader cache: `%APPDATA%\trance\screensavers\` → `%APPDATA%\trance-App\screensavers\`
  - Linux package names: `trance` → `trance-App`

## [2.6.4] - 2026-06-06
### Changed
- Reorganized repository file layout to align with ARCHITECTURE.md.
- App header title updated to "trance - Screensaver Manager".
- Disabled borderless console mode.
- Embedded app icon into Windows installer package.

## [3.0.1] - 2026-06-06
### Added
- Added author and maintainer metadata for packaging.

## [3.0.0] - 2026-06-06
### Changed
- Renamed organization to `local76`.
- Renamed executable from `rtem` to `trance`.
- Reorganized directory structure to group packaging files inside `dist/packages/`.
