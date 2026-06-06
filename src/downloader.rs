//! Downloader module for curated screensavers from the registry.
//! Supports platform-specific downloads (windows, linux-deb, etc.) so rSaver
//! can fetch the correct binary/package for the current OS.
//!
//! Registry entries can specify a `downloads` map (preferred) or legacy `download_url`.
//! Selection uses `current_platform()` ("windows", "linux-deb", etc.).

use std::collections::HashMap;
use std::io::Read;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// An entry in the curated screensaver online registry.
/// Supports cross-platform downloads via the `downloads` map (preferred).
/// For backward compatibility, a top-level `download_url` is still accepted
/// (treated as the "windows" platform).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RegistryEntry {
    /// Friendly name of the screensaver.
    pub name: String,
    /// Author / developer name.
    pub author: String,
    /// Brief description of the screensaver.
    pub description: String,
    /// Legacy single download URL (used as fallback for "windows").
    #[serde(default)]
    pub download_url: Option<String>,
    /// Platform-specific download URLs, e.g.
    /// "windows": "...beams.scr",
    /// "linux-deb": "...beams.deb"
    #[serde(default)]
    pub downloads: Option<HashMap<String, String>>,
    /// Current version string.
    pub version: String,
}

impl RegistryEntry {
    /// Returns the best download URL for the current platform.
    pub fn download_url_for_current_platform(&self) -> Option<String> {
        let platform = current_platform();
        if let Some(ref map) = self.downloads {
            if let Some(url) = map.get(platform).or_else(|| map.get("linux")).cloned() {
                return Some(url);
            }
        }
        // Fallback to legacy field (assumed windows)
        self.download_url.clone()
    }
}

/// Returns a platform key suitable for the downloads map.
/// "windows", "linux-deb", "linux", "macos", etc.
pub fn current_platform() -> &'static str {
    match std::env::consts::OS {
        "windows" => "windows",
        "linux" => "linux",   // plain xscreensaver ELF binary + .xml (in a tarball)
        "macos" => "macos",
        _ => "unknown",
    }
}

/// The status of a background screensaver file download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadStatus {
    /// File download is currently in progress.
    Downloading,
    /// File download finished successfully.
    Success,
    /// An error occurred during download.
    Error(String),
}

/// The shared thread-safe state tracking the progress of an active download.
#[derive(Debug, Clone)]
pub struct DownloadState {
    /// Friendly name of the downloading screensaver.
    pub name: String,
    /// Fractional download progress (from 0.0 to 1.0).
    pub progress: f64,
    /// Total size of the file in bytes.
    pub total_bytes: u64,
    /// Number of bytes downloaded so far.
    pub downloaded_bytes: u64,
    /// Current execution status of the download.
    pub status: DownloadStatus,
}

/// Fetch registry entry list from the target URL.
/// Each entry may contain platform-specific `downloads` (preferred) or a legacy
/// `download_url`. Callers should use `entry.download_url_for_current_platform()`.
pub fn fetch_registry(url: &str) -> Result<Vec<RegistryEntry>, Box<dyn std::error::Error>> {
    let response = ureq::get(url).call()?;
    let body = response.into_string()?;
    let entries: Vec<RegistryEntry> = serde_json::from_str(&body)?;
    Ok(entries)
}

/// Spawn background download of the specified screensaver for the current platform.
pub fn spawn_download(entry: &RegistryEntry) -> Arc<Mutex<DownloadState>> {
    let name = entry.name.clone();
    let download_url = entry.download_url_for_current_platform()
        .unwrap_or_else(|| entry.download_url.clone().unwrap_or_default());
    
    let state = Arc::new(Mutex::new(DownloadState {
        name: name.clone(),
        progress: 0.0,
        total_bytes: 0,
        downloaded_bytes: 0,
        status: DownloadStatus::Downloading,
    }));

    let thread_state = state.clone();
    
    let dest_path = {
        // Cross-platform screensavers drop directory
        let base = if cfg!(target_os = "windows") {
            crate::config::LocalConfig::config_path()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        } else {
            // Linux / macOS: use XDG data or HOME
            std::env::var("XDG_DATA_HOME")
                .ok()
                .map(PathBuf::from)
                .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".local/share")))
                .map(|p| p.join("rSaver"))
        };

        base.and_then(|parent| {
            let filename = download_url.split('/').next_back().unwrap_or("screensaver.bin").to_string();
            Some(parent.join("screensavers").join(filename))
        })
    };

    std::thread::spawn(move || {
        let res = (|| -> Result<(), Box<dyn std::error::Error>> {
            let Some(path) = dest_path else {
                return Err("Failed to resolve appdata directory".into());
            };
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let response = ureq::get(&download_url).call()?;
            let total_bytes = response
                .header("Content-Length")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);

            let mut reader = response.into_reader();
            let mut file = File::create(&path)?;
            let mut buffer = [0; 8192];
            let mut downloaded: u64 = 0;

            loop {
                let bytes_read = reader.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                file.write_all(&buffer[..bytes_read])?;
                downloaded += bytes_read as u64;

                // Update state
                if let Ok(mut s) = thread_state.lock() {
                    s.downloaded_bytes = downloaded;
                    s.total_bytes = total_bytes;
                    if total_bytes > 0 {
                        s.progress = downloaded as f64 / total_bytes as f64;
                    }
                }
            }

            // Post-download handling for Linux xscreensaver hacks
            if cfg!(target_os = "linux") && path.extension().map_or(false, |e| e == "gz" || path.to_string_lossy().ends_with(".tar.gz")) {
                // Extract tarball and install the binary + .xml for xscreensaver
                let saver_name = name.to_lowercase();
                let extract_dir = path.parent().unwrap().join(format!("{}_extract", saver_name));
                std::fs::create_dir_all(&extract_dir)?;

                // Use system tar (ubiquitous on Linux)
                let status = std::process::Command::new("tar")
                    .args(["-xzf", path.to_str().unwrap(), "-C", extract_dir.to_str().unwrap()])
                    .status()?;

                if !status.success() {
                    return Err("Failed to extract Linux tarball".into());
                }

                // Typical contents: the binary and <name>.xml
                // Place binary in ~/.xscreensaver/
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let xsaver_dir = PathBuf::from(&home).join(".xscreensaver");
                std::fs::create_dir_all(&xsaver_dir)?;

                // Find and move the binary (any executable file)
                for entry in std::fs::read_dir(&extract_dir)? {
                    let entry = entry?;
                    let file_path = entry.path();
                    if file_path.is_file() {
                        let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
                        if !file_name.ends_with(".xml") {
                            let target = xsaver_dir.join(&file_name);
                            std::fs::rename(&file_path, &target)?;
                            // Make executable
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;
                                let mut perms = std::fs::metadata(&target)?.permissions();
                                perms.set_mode(0o755);
                                std::fs::set_permissions(&target, perms)?;
                            }
                        } else {
                            // Place .xml in xscreensaver config dir (user or system)
                            let config_dir = xsaver_dir.join("config"); // or /usr/share/xscreensaver/config
                            std::fs::create_dir_all(&config_dir)?;
                            let target_xml = config_dir.join(&file_name);
                            std::fs::rename(&file_path, &target_xml)?;
                        }
                    }
                }

                // Clean up
                let _ = std::fs::remove_dir_all(&extract_dir);
                let _ = std::fs::remove_file(&path);
            }

            if let Ok(mut s) = thread_state.lock() {
                s.status = DownloadStatus::Success;
                s.progress = 1.0;
            }
            Ok(())
        })();

        if let Err(e) = res {
            if let Ok(mut s) = thread_state.lock() {
                s.status = DownloadStatus::Error(e.to_string());
            }
        }
    });

    state
}
