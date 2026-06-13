//! Generic file-based diagnostics logging.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};

static LOG_FILE: OnceLock<Mutex<Option<File>>> = OnceLock::new();
static LOG_APP_NAME: OnceLock<String> = OnceLock::new();
static EVENT_LOG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Set the per-app log file folder name (e.g. `"helm"`).
pub fn set_log_app_name(name: &str) {
    let _ = LOG_APP_NAME.set(name.to_string());
}

/// Enable or disable event logging syncing. Stubbed for self-contained helm.
pub fn set_event_log_enabled(enabled: bool) {
    EVENT_LOG_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Check if event logging syncing is enabled. Stubbed for self-contained helm.
pub fn is_event_log_enabled() -> bool {
    EVENT_LOG_ENABLED.load(Ordering::Relaxed)
}

fn get_log_app_name() -> &'static str {
    LOG_APP_NAME.get_or_init(|| "trance".to_string())
}

/// Helper to resolve the standard AppData folder for diagnostics logging.
pub fn get_appdata_log_path() -> Option<PathBuf> {
    crate::config::LocalConfig::config_path()
        .and_then(|p| p.parent().map(|p| p.join("trance.log")))
}

fn get_log_file() -> &'static Mutex<Option<File>> {
    LOG_FILE.get_or_init(|| {
        let file_opt = get_appdata_log_path()
            .or_else(|| Some(PathBuf::from("trance.log")))
            .and_then(|path| {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .ok()
            });
        Mutex::new(file_opt)
    })
}

/// Thread-safe silent logger helper that appends diagnostic logs to a local file.
pub fn log_message(level: &str, msg: &str) {
    let mutex = get_log_file();
    if let Ok(mut guard) = mutex.lock() {
        if let Some(ref mut file) = *guard {
            let epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let _ = writeln!(file, "[{}] [{}] {}", epoch, level, msg);
        }
    }
}

/// Log an informational message to the diagnostics log.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::logger::log_message("INFO", &format!($($arg)*));
    };
}

/// Log a warning message to the diagnostics log.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::logger::log_message("WARN", &format!($($arg)*));
    };
}

/// Log an error message to the diagnostics log.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::logger::log_message("ERROR", &format!($($arg)*));
    };
}

/// Log a debugging message to the diagnostics log.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::logger::log_message("DEBUG", &format!($($arg)*));
    };
}

