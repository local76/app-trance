# ❖ rSaver — Windows Screensavers Manager ❖

A lightweight, modern Windows Screen Saver Management TUI dashboard built in Rust. `rSaver` offers a centralized controller for discovering, previewing, configuring, and cycling screensavers on mixed-DPI multi-monitor environments without touching intrusive registry editors.

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│  ❖  rSaver  ❖  │  user@localhost  │  Screen: 1920x1080 (96 DPI)  │  Power: AC (Charging)  │  Theme: Dark   │
├──────────────────────────────────────────────────────────────────────────────────────────────────────────┤
```

You can install `rSaver` globally via the Windows Package Manager (WinGet):
```powershell
winget install TourianDynamics.rsav
```

---

## 🚀 What rSaver Does
`rSaver` manages the native Windows Screen Saver system by bridging standard OS-level registry settings with modern terminal-based configuration. Key capabilities include:
*   **Automatic Discovery**: Scans Windows system folders (`System32`, `SysWOW64`, etc.) and a dedicated user folder in `%APPDATA%\rSaver\screensavers` for `.scr` executables.
*   **Config Sync Alerts (Out-of-Sync Detection)**: Automatically monitors registry changes in the background and hot-reloads `rSaver` if the system screensaver is modified externally (e.g., via the native Windows Settings).
*   **Prevent System Sleep**: Easily toggle system sleep prevention on/off directly from the dashboard (useful for presentations, downloads, or simply keeping custom screensavers running indefinitely).
*   **High-DPI Scaling & Aesthetics**: Dynamically resizes the layout to a compact `110x38` terminal window and adopts the Windows accent color for highlighted interfaces.
*   **Curated Screensaver Catalog**: Discover, download, and manage a collection of curated retro terminal screensavers directly from the TUI interface.

---

## 🖥️ Curated Screensaver Collection
`rSaver` comes integrated with a catalog of retro terminal-style screensavers optimized for Windows 11:
*   **`win-beams.scr`**: Colorful sweeping spotlight beams bouncing off terminal walls.
*   **`win-bhop.scr`**: Animated cyber-themed scrolling panels.
*   **`win-matrix.scr`**: Classic cascading rain of digital characters.
*   **`win-pipe.scr`**: Retro 3D pipe generation.
*   **`win-star.scr`**: Fast-paced starfield simulation.

---

## 🩺 CLI Subcommands & Flags
`rSaver` acts as both a dashboard and a screensaver command-line handler.

```powershell
rsav.exe [OPTIONS] [COMMAND]
```

### Options:
*   `--theme <THEME>` : Force a specific TUI theme (`dark`, `light`, `high-contrast`, `no-color`).

### Commands:
*   `tui` : Launch the interactive TUI dashboard (default when run without arguments).
*   `run` : Launch the currently active screensaver in fullscreen mode (`rsav run`).
*   `stop` : Kill any active screensavers running on the system.
*   `toggle-active` : Enable or disable the screensaver timeout system-wide.
*   `doctor` : Verify registry access, discovery folders, and log files. Pass `--fix` to auto-heal missing configuration assets.

---

## ⌨️ TUI Keyboard Controls
Use the following shortcuts to navigate the dashboard:
*   **`Tab / Shift-Tab`** : Cycle focus between *Global Preferences* and the *Screensaver List*.
*   **`↑ / ↓` or `k / j`** : Navigate lists and settings.
*   **`Enter`** : Trigger selection, toggle checkboxes, or open configurations.
*   **`Space`** : Preview the highlighted screensaver in fullscreen.
*   **`a`** : Apply the highlighted screensaver as the active system-wide screensaver.
*   **`c`** : Open the custom settings dialog for the highlighted screensaver (if supported).
*   **`f`** : Toggle search filtering on the screensaver list.
*   **`d`** : (Online Tab) Download and install the selected screensaver from the catalog.
*   **`q / Esc`** : Quit `rSaver` or close active overlay popups.

---

## 💾 Custom Preferences & Data Storage
All data is stored locally under your Windows user profile:
*   **rSaver Custom Preferences**: Stored at `%APPDATA%\rSaver\config.yaml` (contains last-selected screensaver, prevent-sleep status, custom cycle interval, and catalog feed URLs).
*   **Screensaver Drop Path**: Put custom `.scr` screensavers in `%APPDATA%\rSaver\screensavers` to have `rSaver` discover them.
*   **Logs File**: Diagnostics are written to `%APPDATA%\rSaver\rSaver.log` so they do not clutter raw terminal outputs.

### Custom Feeds:
To add custom online registry catalogs, open `%APPDATA%\rSaver\config.yaml` and add your feed URLs separated by semicolons:
```yaml
feed_urls: https://raw.githubusercontent.com/tourian-dynamics/rSaver/master/registry.json;https://example.com/custom-screensavers.json
```

---

## 🛠️ Building From Source
Ensure you have the Rust compiler toolchain installed on Windows.

1. Clone the repository and navigate to the folder:
    ```powershell
    cd rSaver
    ```
2. Build the release binary:
    ```powershell
    just build
    ```
    The optimized binary will be compiled to `target/release/rsav.exe`. You can rename this to `rsav.scr` to install it directly as a Windows screensaver!
