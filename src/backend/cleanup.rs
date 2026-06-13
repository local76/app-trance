use crate::config::{GlobalConfig, LocalConfig};
use crate::backend::preview::Screensaver;
use crate::theme::TuiTheme;
use crate::win32;


/// Clean up config values if active/selected screensavers no longer exist.
pub fn sanitize_config(
    global: &mut GlobalConfig,
    local: &mut LocalConfig,
    screensavers: &[Screensaver],
) {
    if !global.active_scr.is_empty() {
        let path = std::path::Path::new(&global.active_scr);
        let exists = path.exists() || screensavers.iter().any(|s| s.path.file_name() == path.file_name());
        if !exists {
            warn!("Active screensaver in registry is missing (path={:?}), resetting registry.", global.active_scr);
            let first_valid = screensavers.iter().find(|s| s.path.exists());
            if let Some(s) = first_valid {
                global.active_scr = s.path.to_string_lossy().into_owned();
                global.active = true;
            } else {
                global.active_scr = String::new();
                global.active = false;
            }
            let _ = global.save();
        }
    }

    if let Some(ref name) = local.last_selected {
        let exists = screensavers.iter().any(|s| s.path.file_name().and_then(|f| f.to_str()) == Some(name));
        if !exists {
            local.last_selected = None;
            let _ = local.save();
        }
    }
}

/// Logs system environment metrics to tracing.
pub fn log_environment(theme: &TuiTheme) {
    let metrics = win32::SystemMetrics::query();
    info!(
        "environment: screen={}, dpi={}, window_dpi={}, dark_mode={}, high_contrast={}, no_color={}, accent={:?}, ac_online={}, battery={}",
        format!("{}x{}", metrics.screen_w, metrics.screen_h),
        metrics.dpi,
        metrics.window_dpi,
        metrics.dark_mode,
        metrics.high_contrast,
        theme.no_color,
        metrics.accent,
        metrics.power.ac_online,
        metrics.power.battery_percent
    );
}
