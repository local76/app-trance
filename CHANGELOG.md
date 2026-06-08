# Changelog

All notable changes to this project will be documented in this file.

## [2.6.6] - 2026-06-08

### Refactored
- Refactored monolithic `src/app.rs` into modular sub-files under `src/app/` (`mod.rs`, `actions.rs`, `keys.rs`, `cycle.rs`), keeping all source files under 500 lines.
- Refactored monolithic `src/ui.rs` into modular sub-files under `src/ui/` (`mod.rs`, `panels.rs`, `utils.rs`), keeping all source files under 500 lines.
- Extracted Win32 API declarations from `src/main.rs` into `src/win32.rs`.
- Resolved all compiler and Clippy warnings across the codebase.
- **Renamed project** from `rIdle` to `rIdle-tui`. The GitHub repository, Cargo package name, binary name, and all user-facing labels now use the `-tui` suffix to make the program's role as a terminal user interface explicit (matching `rTemplate-tui`).
  - Repository: `local76/rIdle` → `local76/rIdle-tui`
  - Crate/binary: `ridle` → `ridle-tui`
  - Config file: `%APPDATA%\rIdle\config.yaml` → `%APPDATA%\rIdle-tui\config.yaml`
  - Downloader cache: `%APPDATA%\rIdle\screensavers\` → `%APPDATA%\rIdle-tui\screensavers\`
  - Linux package names: `ridle` → `ridle-tui`

## [2.6.4] - 2026-06-06
### Changed
- Reorganized repository file layout to align with ARCHITECTURE.md.
- TUI header title updated to "rIdle - Screensaver Manager".
- Disabled borderless console mode.
- Embedded app icon into Windows installer package.

## [3.0.1] - 2026-06-06
### Added
- Added author and maintainer metadata for packaging.

## [3.0.0] - 2026-06-06
### Changed
- Renamed organization to `local76`.
- Renamed executable from `rtem` to `ridle`.
- Reorganized directory structure to group packaging files inside `dist/packages/`.