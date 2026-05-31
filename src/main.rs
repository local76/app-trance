#![cfg_attr(not(debug_assertions), windows_subsystem = "console")]

mod registry;
mod theme;
mod preview;

use std::path::PathBuf;
use std::io::{stdout, Write};
use std::time::Duration;
use crossterm::{
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, Event, KeyCode, KeyModifiers},
    cursor, style, execute, queue
};
use winapi::um::winbase::SetThreadExecutionState;
use winapi::um::winnt::{ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED, ES_AWAYMODE_REQUIRED};
use winapi::um::winuser::LockWorkStation;

// Win32 Borderless Console Declares
unsafe extern "system" {
    fn GetConsoleWindow() -> winapi::shared::windef::HWND;
    fn GetWindowLongW(hwnd: winapi::shared::windef::HWND, index: i32) -> i32;
    fn SetWindowLongW(hwnd: winapi::shared::windef::HWND, index: i32, new_long: i32) -> i32;
    fn SetWindowPos(
        hwnd: winapi::shared::windef::HWND,
        hwnd_insert_after: winapi::shared::windef::HWND,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> i32;
}

const GWL_STYLE: i32 = -16;
const WS_CAPTION: i32 = 0x00C00000;
const WS_THICKFRAME: i32 = 0x00040000;
const WS_MINIMIZEBOX: i32 = 0x00020000;
const WS_MAXIMIZEBOX: i32 = 0x00010000;
const WS_SYSMENU: i32 = 0x00080000;

const SWP_FRAMECHANGED: u32 = 0x0020;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;

// Double-buffering data structures for flicker-free rendering
#[derive(Clone, PartialEq, Copy)]
struct Cell {
    c: char,
    fg: style::Color,
    bg: style::Color,
    bold: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            c: ' ',
            fg: style::Color::Reset,
            bg: style::Color::Reset,
            bold: false,
        }
    }
}

struct ScreenBuffer {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

impl ScreenBuffer {
    fn new(width: u16, height: u16) -> Self {
        ScreenBuffer {
            width,
            height,
            cells: vec![Cell::default(); (width as usize) * (height as usize)],
        }
    }

    fn set(&mut self, x: u16, y: u16, c: char, fg: style::Color, bg: style::Color, bold: bool) {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.cells[idx] = Cell { c, fg, bg, bold };
        }
    }

    fn print_string(&mut self, mut x: u16, y: u16, s: &str, fg: style::Color, bg: style::Color, bold: bool) {
        for ch in s.chars() {
            self.set(x, y, ch, fg, bg, bold);
            x += 1;
        }
    }
}

fn draw_diff(old: Option<&ScreenBuffer>, new: &ScreenBuffer) -> std::io::Result<()> {
    let mut stdout_handle = stdout();
    let mut current_fg = style::Color::Reset;
    let mut current_bg = style::Color::Reset;
    let mut current_bold = false;
    
    // Clear terminal screen completely once if there is no previous buffer
    if old.is_none() {
        queue!(
            stdout_handle,
            style::SetBackgroundColor(style::Color::Reset),
            terminal::Clear(terminal::ClearType::All)
        )?;
    }

    // Set initial state
    queue!(
        stdout_handle,
        style::SetForegroundColor(current_fg),
        style::SetBackgroundColor(current_bg),
        style::SetAttribute(style::Attribute::Reset)
    )?;

    for y in 0..new.height {
        let mut x = 0;
        while x < new.width {
            let idx = (y as usize) * (new.width as usize) + (x as usize);
            let new_cell = new.cells[idx];
            
            // Check if cell has changed (if old exists)
            let changed = match old {
                Some(o) if o.width == new.width && o.height == new.height => {
                    o.cells[idx] != new_cell
                }
                _ => true,
            };
            
            if changed {
                // Move cursor to (x, y)
                queue!(stdout_handle, cursor::MoveTo(x, y))?;
                
                // Update styling if changed
                if new_cell.fg != current_fg {
                    queue!(stdout_handle, style::SetForegroundColor(new_cell.fg))?;
                    current_fg = new_cell.fg;
                }
                if new_cell.bg != current_bg {
                    queue!(stdout_handle, style::SetBackgroundColor(new_cell.bg))?;
                    current_bg = new_cell.bg;
                }
                if new_cell.bold != current_bold {
                    if new_cell.bold {
                        queue!(stdout_handle, style::SetAttribute(style::Attribute::Bold))?;
                    } else {
                        queue!(stdout_handle, style::SetAttribute(style::Attribute::Reset))?;
                        // Resetting attributes resets colors, so we must re-apply them!
                        queue!(
                            stdout_handle,
                            style::SetForegroundColor(current_fg),
                            style::SetBackgroundColor(current_bg)
                        )?;
                    }
                    current_bold = new_cell.bold;
                }
                
                // Print character
                queue!(stdout_handle, style::Print(new_cell.c))?;
            }
            x += 1;
        }
    }
    stdout_handle.flush()?;
    Ok(())
}

fn draw_border_box_buf(buf: &mut ScreenBuffer, x: u16, y: u16, w: u16, h: u16, title: &str, active: bool, theme: &theme::TuiTheme) {
    let border_color = if active { theme.accent_primary } else { theme.border };
    
    // Top border
    buf.set(x, y, '┌', border_color, style::Color::Reset, false);
    for dx in 1..(w - 1) {
        buf.set(x + dx, y, '─', border_color, style::Color::Reset, false);
    }
    buf.set(x + w - 1, y, '┐', border_color, style::Color::Reset, false);
    
    // Title
    if !title.is_empty() {
        let display_title = format!(" {} ", title);
        let title_color = if active { theme.accent_primary } else { theme.header };
        buf.print_string(x + 2, y, &display_title, title_color, style::Color::Reset, true);
    }
    
    // Sides
    for row in 1..(h - 1) {
        buf.set(x, y + row, '│', border_color, style::Color::Reset, false);
        buf.set(x + w - 1, y + row, '│', border_color, style::Color::Reset, false);
    }
    
    // Bottom border
    buf.set(x, y + h - 1, '└', border_color, style::Color::Reset, false);
    for dx in 1..(w - 1) {
        buf.set(x + dx, y + h - 1, '─', border_color, style::Color::Reset, false);
    }
    buf.set(x + w - 1, y + h - 1, '┘', border_color, style::Color::Reset, false);
}

#[derive(Clone, Copy, PartialEq)]
enum FocusedSection {
    GlobalPrefs,
    SaverList,
    ConfigFields,
}

struct AppState {
    screensavers: Vec<(String, PathBuf, registry::ScreensaverConfig)>,
    highlighted_index: usize,
    scroll_offset: usize,
    focused_section: FocusedSection,
    config_field_index: usize,
    global_field_index: usize,
    global_config: registry::GlobalConfig,
    active_theme: theme::TuiTheme,
    prevent_sleep: bool,
    prev_screen: Option<ScreenBuffer>,
}

fn get_config_path() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    Some(PathBuf::from(appdata).join(".omaxi").join("apps").join("ssm").join("config.yaml"))
}

struct LocalConfig {
    last_selected: Option<String>,
    prevent_sleep: bool,
}

fn load_local_config() -> Option<LocalConfig> {
    let path = get_config_path()?;
    let content = std::fs::read_to_string(path).ok()?;
    let mut last_selected = None;
    let mut prevent_sleep = false;
    
    for line in content.lines() {
        if let Some(val) = line.strip_prefix("last_selected: ") {
            last_selected = Some(val.to_string());
        } else if let Some(val) = line.strip_prefix("prevent_sleep: ") {
            prevent_sleep = val.trim() == "true";
        }
    }
    Some(LocalConfig { last_selected, prevent_sleep })
}

fn save_local_config(last_selected: &str, prevent_sleep: bool) {
    if let Some(path) = get_config_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = format!(
            "last_selected: {}\nprevent_sleep: {}\n",
            last_selected, prevent_sleep
        );
        let _ = std::fs::write(path, content);
    }
}


fn render(state: &mut AppState, width: u16, height: u16) -> std::io::Result<()> {
    let theme = &state.active_theme;
    
    let mut buf = ScreenBuffer::new(width, height);

    if width < 60 || height < 24 {
        buf.print_string(2, 2, "Terminal size too small. Please resize to at least 60x24.", theme.accent_secondary, style::Color::Reset, false);
        buf.print_string(2, 3, &format!("Current size: {}x{}", width, height), theme.text_dim, style::Color::Reset, false);
        draw_diff(state.prev_screen.as_ref(), &buf)?;
        state.prev_screen = Some(buf);
        return Ok(());
    }

    let screensavers_len = state.screensavers.len();
    let fields_len = if !state.screensavers.is_empty() {
        state.screensavers[state.highlighted_index].2.fields.len()
    } else {
        0
    };

    // Determine active highlight panels
    let sys_active = state.focused_section == FocusedSection::GlobalPrefs;
    let saver_list_active = state.focused_section == FocusedSection::SaverList;
    let saver_conf_active = state.focused_section == FocusedSection::ConfigFields;

    // Top Title Bar
    buf.print_string(2, 1, "SCREEN SAVER MANAGEMENT (SSM)", theme.accent_secondary, style::Color::Reset, true);
    buf.print_string(2, 2, &"─".repeat((width - 4) as usize), theme.text_dim, style::Color::Reset, false);

    // Compute dynamic layout heights
    let sys_h = 6; // Fits 4 internal rows
    
    let conf_h = if fields_len > 0 {
        (fields_len as u16) + 3
    } else {
        4
    };

    // Total height of static layout bits: Title (2) + Help (6) + Global prefs (sys_h) + borders/separators (1) = sys_h + conf_h + 9
    let min_non_list_height = conf_h + sys_h + 9;

    let available_list_height = if height > min_non_list_height {
        height - min_non_list_height
    } else {
        3
    };

    let list_h = ((screensavers_len as u16) + 2)
        .min(available_list_height)
        .max(3);

    // Coordinates mapping
    let sys_x = 2;
    let sys_y = 3;
    let sys_w = width - 4;

    let list_x = 2;
    let list_y = sys_y + sys_h;
    let list_w = width - 4;

    let conf_x = 2;
    let conf_y = list_y + list_h;
    let conf_w = width - 4;

    // 1. Global System Configurations Box (Top, h=sys_h)
    draw_border_box_buf(&mut buf, sys_x, sys_y, sys_w, sys_h, "GLOBAL SYSTEM PREFERENCES", sys_active, theme);
    
    let active_focused = sys_active && state.global_field_index == 0;
    let timeout_focused = sys_active && state.global_field_index == 1;
    let sleep_focused = sys_active && state.global_field_index == 2;

    let is_sys_active = state.global_config.active;
    let system_status = if is_sys_active { "ACTIVE" } else { "DISABLED" };
    let status_color = if is_sys_active { theme.accent_secondary } else { theme.text_dim };
    
    let sleep_status = if state.prevent_sleep { "ACTIVE (SYSTEM AWAKE)" } else { "DISABLED (NORMAL)" };
    let sleep_status_color = if state.prevent_sleep { theme.accent_secondary } else { theme.text_dim };

    // Active focused row
    let active_color = if active_focused { theme.accent_secondary } else { theme.text_main };
    buf.print_string(sys_x + 2, sys_y + 1, if active_focused { "▶ " } else { "  " }, if active_focused { theme.accent_primary } else { style::Color::Reset }, style::Color::Reset, false);
    buf.print_string(sys_x + 4, sys_y + 1, "Global Active Status: ", active_color, style::Color::Reset, false);
    buf.print_string(sys_x + 26, sys_y + 1, system_status, status_color, style::Color::Reset, true);

    // Timeout row
    let timeout_color = if timeout_focused { theme.accent_secondary } else { theme.text_main };
    buf.print_string(sys_x + 2, sys_y + 2, if timeout_focused { "▶ " } else { "  " }, if timeout_focused { theme.accent_primary } else { style::Color::Reset }, style::Color::Reset, false);
    buf.print_string(sys_x + 4, sys_y + 2, "Screen Saver Timeout: ", timeout_color, style::Color::Reset, false);
    buf.print_string(sys_x + 26, sys_y + 2, &format!("{} minutes", state.global_config.timeout / 60), theme.accent_primary, style::Color::Reset, false);

    // Sleep row
    let sleep_color = if sleep_focused { theme.accent_secondary } else { theme.text_main };
    buf.print_string(sys_x + 2, sys_y + 3, if sleep_focused { "▶ " } else { "  " }, if sleep_focused { theme.accent_primary } else { style::Color::Reset }, style::Color::Reset, false);
    buf.print_string(sys_x + 4, sys_y + 3, "Ignore System Sleep:  ", sleep_color, style::Color::Reset, false);
    buf.print_string(sys_x + 26, sys_y + 3, sleep_status, sleep_status_color, style::Color::Reset, true);

    // Applied Saver row
    let reg_active_path = PathBuf::from(&state.global_config.active_scr);
    let active_scr_name = reg_active_path.file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("None");
    buf.print_string(sys_x + 4, sys_y + 4, "Applied Screensaver: ", theme.text_dim, style::Color::Reset, false);
    buf.print_string(sys_x + 25, sys_y + 4, active_scr_name, theme.header, style::Color::Reset, false);

    // 2. Screen Savers Box (Middle, h=list_h)
    draw_border_box_buf(&mut buf, list_x, list_y, list_w, list_h, "SCREEN SAVERS", saver_list_active, theme);

    let max_visible_savers = (list_h - 2) as usize;
    if state.screensavers.is_empty() {
        buf.print_string(list_x + 2, list_y + 2, "No savers found.", theme.text_dim, style::Color::Reset, false);
    } else {
        let sel = state.highlighted_index;
        if sel < state.scroll_offset {
            state.scroll_offset = sel;
        } else if sel >= state.scroll_offset + max_visible_savers {
            state.scroll_offset = sel - max_visible_savers + 1;
        }

        let active_filename = reg_active_path.file_name()
            .and_then(|f| f.to_str())
            .unwrap_or_default()
            .to_lowercase();

        for i in 0..max_visible_savers {
            let idx = state.scroll_offset + i;
            if idx >= state.screensavers.len() {
                break;
            }
            let (name, path, _) = &state.screensavers[idx];
            let is_highlighted = state.highlighted_index == idx;
            let is_focused = saver_list_active && state.highlighted_index == idx;
            
            let is_active_saver = path.file_name()
                .and_then(|f| f.to_str())
                .unwrap_or_default()
                .to_lowercase() == active_filename;
                
            let row_y = list_y + 1 + i as u16;
            
            let indicator = if is_focused {
                "▶ "
            } else if is_highlighted {
                "▷ "
            } else {
                "  "
            };
            
            let name_color = if is_focused || is_highlighted {
                theme.text_main
            } else {
                theme.text_dim
            };
            
            buf.print_string(list_x + 1, row_y, indicator, if is_focused { theme.accent_primary } else { theme.text_dim }, style::Color::Reset, false);
            buf.print_string(list_x + 3, row_y, &format!("{:<20}", name), name_color, style::Color::Reset, is_highlighted);
            if is_active_saver {
                buf.print_string(list_x + 24, row_y, "[Applied]", theme.accent_secondary, style::Color::Reset, false);
            }
        }
        
        // Scrollbar
        if state.screensavers.len() > max_visible_savers {
            let total = state.screensavers.len() as f32;
            let percent_offset = state.scroll_offset as f32 / total;
            let percent_visible = max_visible_savers as f32 / total;
            
            let bar_start = ((list_h - 2) as f32 * percent_offset) as u16;
            let bar_len = (((list_h - 2) as f32 * percent_visible) as u16).max(1);
            
            for row in 1..(list_h - 1) {
                let is_bar = row - 1 >= bar_start && row - 1 < bar_start + bar_len;
                buf.set(list_x + list_w - 1, list_y + row, if is_bar { '█' } else { '│' }, if is_bar { theme.accent_primary } else { theme.border }, style::Color::Reset, false);
            }
        }
    }

    // 3. Saver Settings Configuration Box (Bottom, h=conf_h)
    let active_name = state.screensavers.get(state.highlighted_index)
        .map(|s| s.0.as_str())
        .unwrap_or("No Saver Selected");
        
    draw_border_box_buf(&mut buf, conf_x, conf_y, conf_w, conf_h, &format!("CONFIGURATION: {}", active_name), saver_conf_active, theme);

    if let Some(idx) = state.screensavers.get(state.highlighted_index) {
        let config = &idx.2;
        if config.fields.is_empty() {
            buf.print_string(conf_x + 3, conf_y + 2, "No configurable parameters for this screensaver.", theme.text_dim, style::Color::Reset, false);
        } else {
            for (f_idx, field) in config.fields.iter().enumerate() {
                let row_y = conf_y + 2 + f_idx as u16;
                if row_y >= conf_y + conf_h - 1 {
                    break;
                }
                
                let is_focused = saver_conf_active && state.config_field_index == f_idx;
                let field_color = if is_focused { theme.accent_secondary } else { theme.text_main };
                
                buf.print_string(conf_x + 2, row_y, if is_focused { "▶ " } else { "  " }, if is_focused { theme.accent_primary } else { theme.text_dim }, style::Color::Reset, false);
                
                match field {
                    registry::ConfigField::Integer { label, min, max, value, .. } => {
                        buf.print_string(conf_x + 4, row_y, &format!("{:<16}: ", label), field_color, style::Color::Reset, false);
                        
                        // Render slider
                        let slider_width = (conf_w - 36).max(10) as usize;
                        let val_range = (max - min) as f32;
                        let progress = if val_range > 0.0 {
                            (value - min) as f32 / val_range
                        } else {
                            0.0
                        };
                        let filled = (progress * slider_width as f32) as usize;
                        let empty = slider_width - filled;
                        
                        let slider_start = conf_x + 22;
                        buf.print_string(slider_start, row_y, "[", theme.border, style::Color::Reset, false);
                        buf.print_string(slider_start + 1, row_y, &"█".repeat(filled), theme.accent_primary, style::Color::Reset, false);
                        buf.print_string(slider_start + 1 + filled as u16, row_y, &"░".repeat(empty), theme.text_dim, style::Color::Reset, false);
                        buf.print_string(slider_start + 1 + slider_width as u16, row_y, "] ", theme.border, style::Color::Reset, false);
                        buf.print_string(slider_start + 3 + slider_width as u16, row_y, &format!("{:<4}", value), theme.accent_primary, style::Color::Reset, false);
                    }
                    registry::ConfigField::Boolean { label, value, .. } => {
                        let check = if *value { "[X] " } else { "[ ] " };
                        let check_color = if *value { theme.accent_primary } else { theme.text_dim };
                        buf.print_string(conf_x + 4, row_y, check, check_color, style::Color::Reset, false);
                        buf.print_string(conf_x + 8, row_y, label, field_color, style::Color::Reset, false);
                    }
                }
            }
        }
    }

    // 4. Help/Status Bar at bottom (y=height-6..height-1)
    let help_x = 2;
    buf.print_string(help_x, height - 6, "[TAB] Cycle Pane", theme.accent_primary, style::Color::Reset, false);
    buf.print_string(help_x, height - 5, "[Up/Down] Navigate Options", theme.accent_primary, style::Color::Reset, false);
    buf.print_string(help_x, height - 4, "[Space/Enter] Toggle/Select", theme.accent_primary, style::Color::Reset, false);
    buf.print_string(help_x, height - 3, "[Left/Right] Adjust Value", theme.accent_primary, style::Color::Reset, false);
    buf.print_string(help_x, height - 2, "[P] Preview Screensaver", theme.accent_primary, style::Color::Reset, false);
    buf.print_string(help_x, height - 1, "[Q / Esc] Quit Application", theme.accent_primary, style::Color::Reset, false);

    // Compare and draw changes
    draw_diff(state.prev_screen.as_ref(), &buf)?;
    state.prev_screen = Some(buf);
    Ok(())
}

fn handle_key_event(state: &mut AppState, code: KeyCode) -> std::io::Result<bool> {
    let fields_len = if !state.screensavers.is_empty() {
        state.screensavers[state.highlighted_index].2.fields.len()
    } else {
        0
    };
    
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            return Ok(true);
        }
        
        KeyCode::Tab => {
            state.focused_section = match state.focused_section {
                FocusedSection::GlobalPrefs => {
                    if !state.screensavers.is_empty() {
                        FocusedSection::SaverList
                    } else if fields_len > 0 {
                        FocusedSection::ConfigFields
                    } else {
                        FocusedSection::GlobalPrefs
                    }
                }
                FocusedSection::SaverList => {
                    if fields_len > 0 {
                        FocusedSection::ConfigFields
                    } else {
                        FocusedSection::GlobalPrefs
                    }
                }
                FocusedSection::ConfigFields => FocusedSection::GlobalPrefs,
            };
        }
        
        KeyCode::Up => {
            match state.focused_section {
                FocusedSection::GlobalPrefs => {
                    if state.global_field_index > 0 {
                        state.global_field_index -= 1;
                    } else if fields_len > 0 {
                        state.focused_section = FocusedSection::ConfigFields;
                        state.config_field_index = fields_len - 1;
                    } else if !state.screensavers.is_empty() {
                        state.focused_section = FocusedSection::SaverList;
                        state.highlighted_index = state.screensavers.len() - 1;
                    } else {
                        state.global_field_index = 2;
                    }
                }
                FocusedSection::SaverList => {
                    if state.highlighted_index > 0 {
                        state.highlighted_index -= 1;
                    } else {
                        // Move to GlobalPrefs
                        state.focused_section = FocusedSection::GlobalPrefs;
                        state.global_field_index = 2; // Last element in GlobalPrefs
                    }
                }
                FocusedSection::ConfigFields => {
                    if state.config_field_index > 0 {
                        state.config_field_index -= 1;
                    } else if !state.screensavers.is_empty() {
                        // Move to SaverList
                        state.focused_section = FocusedSection::SaverList;
                        state.highlighted_index = state.screensavers.len() - 1;
                    } else {
                        // Move to GlobalPrefs
                        state.focused_section = FocusedSection::GlobalPrefs;
                        state.global_field_index = 2;
                    }
                }
            }
        }
        
        KeyCode::Down => {
            match state.focused_section {
                FocusedSection::GlobalPrefs => {
                    if state.global_field_index < 2 {
                        state.global_field_index += 1;
                    } else if !state.screensavers.is_empty() {
                        // Move to SaverList
                        state.focused_section = FocusedSection::SaverList;
                        state.highlighted_index = 0;
                    } else if fields_len > 0 {
                        // Move to ConfigFields
                        state.focused_section = FocusedSection::ConfigFields;
                        state.config_field_index = 0;
                    } else {
                        state.global_field_index = 0;
                    }
                }
                FocusedSection::SaverList => {
                    if !state.screensavers.is_empty() && state.highlighted_index < state.screensavers.len() - 1 {
                        state.highlighted_index += 1;
                    } else if fields_len > 0 {
                        // Move to ConfigFields
                        state.focused_section = FocusedSection::ConfigFields;
                        state.config_field_index = 0;
                    } else {
                        // Move to GlobalPrefs
                        state.focused_section = FocusedSection::GlobalPrefs;
                        state.global_field_index = 0;
                    }
                }
                FocusedSection::ConfigFields => {
                    if fields_len > 0 && state.config_field_index < fields_len - 1 {
                        state.config_field_index += 1;
                    } else {
                        // Move to GlobalPrefs
                        state.focused_section = FocusedSection::GlobalPrefs;
                        state.global_field_index = 0;
                    }
                }
            }
        }
        
        KeyCode::Left | KeyCode::Char('-') => {
            if state.focused_section == FocusedSection::ConfigFields {
                let field = &mut state.screensavers[state.highlighted_index].2.fields[state.config_field_index];
                if let registry::ConfigField::Integer { min, value, .. } = field {
                    if *value > *min {
                        *value -= 1;
                        let (_, _, config) = &state.screensavers[state.highlighted_index];
                        let _ = registry::save_screensaver_fields(config.registry_name, &config.fields);
                    }
                }
            } else if state.focused_section == FocusedSection::GlobalPrefs && state.global_field_index == 1 {
                // Focus on Global Timeout
                if state.global_config.timeout > 60 {
                    state.global_config.timeout -= 60;
                    let _ = registry::save_global_config(&state.global_config);
                }
            }
        }
        
        KeyCode::Right | KeyCode::Char('+') | KeyCode::Char('=') => {
            if state.focused_section == FocusedSection::ConfigFields {
                let field = &mut state.screensavers[state.highlighted_index].2.fields[state.config_field_index];
                if let registry::ConfigField::Integer { max, value, .. } = field {
                    if *value < *max {
                        *value += 1;
                        let (_, _, config) = &state.screensavers[state.highlighted_index];
                        let _ = registry::save_screensaver_fields(config.registry_name, &config.fields);
                    }
                }
            } else if state.focused_section == FocusedSection::GlobalPrefs && state.global_field_index == 1 {
                // Focus on Global Timeout
                if state.global_config.timeout < 7200 {
                    state.global_config.timeout += 60;
                    let _ = registry::save_global_config(&state.global_config);
                }
            }
        }
        
        KeyCode::Char(' ') | KeyCode::Enter => {
            match state.focused_section {
                FocusedSection::GlobalPrefs => {
                    match state.global_field_index {
                        0 => {
                            // Toggle Active
                            state.global_config.active = !state.global_config.active;
                            let _ = registry::save_global_config(&state.global_config);
                        }
                        2 => {
                            // Toggle prevent sleep
                            state.prevent_sleep = !state.prevent_sleep;
                            let active_file = state.screensavers.get(state.highlighted_index)
                                .and_then(|(_, path, _)| path.file_name().and_then(|f| f.to_str()))
                                .unwrap_or_default();
                            save_local_config(active_file, state.prevent_sleep);
                        }
                        _ => {}
                    }
                }
                FocusedSection::SaverList => {
                    if let Some((_, path, _)) = state.screensavers.get(state.highlighted_index) {
                        state.global_config.active_scr = path.to_string_lossy().into_owned();
                        if registry::save_global_config(&state.global_config).is_ok() {
                            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                                save_local_config(filename, state.prevent_sleep);
                            }
                        }
                    }
                }
                FocusedSection::ConfigFields => {
                    let field = &mut state.screensavers[state.highlighted_index].2.fields[state.config_field_index];
                    if let registry::ConfigField::Boolean { value, .. } = field {
                        *value = !*value;
                        let (_, _, config) = &state.screensavers[state.highlighted_index];
                        let _ = registry::save_screensaver_fields(config.registry_name, &config.fields);
                    }
                }
            }
        }

        KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char('t') | KeyCode::Char('T') => {
            if let Some((_, path, _)) = state.screensavers.get(state.highlighted_index) {
                let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or_default().to_lowercase();
                if filename.contains("ssm") {
                    let _ = std::process::Command::new(path)
                        .arg("--start")
                        .spawn();
                } else {
                    let _ = std::process::Command::new(path)
                        .arg("/s")
                        .spawn();
                }
            }
        }

        _ => {}
    }
    Ok(false)
}

fn run_random_cycle() {
    let discovered = preview::discover_screensavers();
    let filtered: Vec<PathBuf> = discovered.into_iter()
        .map(|(_, path)| path)
        .filter(|path| {
            let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or_default().to_lowercase();
            !filename.contains("ssm") && !filename.contains("uninstall")
        })
        .collect();
        
    if filtered.is_empty() {
        return;
    }

    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    loop {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let idx = (seed as usize) % filtered.len();
        let target_path = &filtered[idx];
        
        let mut child = match std::process::Command::new(target_path)
            .arg("/s")
            .spawn() {
                Ok(child) => child,
                Err(_) => break,
            };
            
        let start_time = std::time::Instant::now();
        let mut child_exited = false;
        while start_time.elapsed() < std::time::Duration::from_secs(30) {
            match child.try_wait() {
                Ok(Some(_)) => {
                    child_exited = true;
                    break;
                }
                Ok(None) => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(_) => {
                    child_exited = true;
                    break;
                }
            }
        }
        
        if child_exited {
            break;
        } else {
            let _ = child.kill();
        }
    }
}

fn run_tui() -> std::io::Result<()> {
    // Strip console border/titlebar on startup
    let hwnd = unsafe { GetConsoleWindow() };
    let original_style = if !hwnd.is_null() {
        let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) };
        let new_style = style & !(WS_CAPTION | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU);
        unsafe {
            SetWindowLongW(hwnd, GWL_STYLE, new_style);
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        Some(style)
    } else {
        None
    };

    let discovered = preview::discover_screensavers();
    let registered_configs = registry::get_screensavers();
    
    let mut screensavers = Vec::new();
    if let Ok(exe_path) = std::env::current_exe() {
        screensavers.push((
            "Random Cycle".to_string(),
            exe_path,
            registry::ScreensaverConfig {
                name: "Random Cycle",
                registry_name: "OmaxiRandom",
                binary_name: "ssm.exe",
                fields: vec![],
            }
        ));
    }
    for (pretty_name, path) in discovered {
        let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or_default().to_lowercase();
        let mut config = registered_configs.iter()
            .find(|rc| rc.binary_name.to_lowercase() == filename)
            .cloned()
            .unwrap_or_else(|| {
                registry::ScreensaverConfig {
                    name: "Custom Screensaver",
                    registry_name: "OmaxiCustom",
                    binary_name: "custom.scr",
                    fields: vec![],
                }
            });
        
        registry::load_screensaver_fields(config.registry_name, &mut config.fields);
        screensavers.push((pretty_name, path, config));
    }

    let global_config = registry::load_global_config();
    let active_theme = theme::get_theme();
    
    let mut last_selected: Option<String> = None;
    let mut prevent_sleep = false;

    if let Some(local) = load_local_config() {
        last_selected = local.last_selected;
        prevent_sleep = local.prevent_sleep;
    }

    let mut selected_index = if !screensavers.is_empty() { Some(0) } else { None };
    if let Some(saved_name) = &last_selected {
        if let Some(pos) = screensavers.iter().position(|(_, path, _)| {
            path.file_name().and_then(|f| f.to_str()) == Some(saved_name.as_str())
        }) {
            selected_index = Some(pos);
        }
    }

    let initial_idx = selected_index.unwrap_or(0);
    let mut state = AppState {
        screensavers,
        highlighted_index: initial_idx,
        scroll_offset: 0,
        focused_section: FocusedSection::GlobalPrefs,
        config_field_index: 0,
        global_field_index: 0,
        global_config,
        active_theme,
        prevent_sleep,
        prev_screen: None,
    };

    // 2. Setup Terminal Raw Mode
    terminal::enable_raw_mode()?;
    let mut stdout_handle = stdout();
    execute!(stdout_handle, EnterAlternateScreen, cursor::Hide)?;

    // 3. Event-driven TUI loop
    let mut needs_redraw = true;
    loop {
        // Continuous Win32 Sleep prevention check
        unsafe {
            if state.prevent_sleep {
                SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED);
            } else {
                SetThreadExecutionState(ES_CONTINUOUS);
            }
        }

        if needs_redraw {
            let (width, height) = terminal::size()?;
            render(&mut state, width, height)?;
            needs_redraw = false;
        }
        
        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == event::KeyEventKind::Press {
                        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                            break;
                        }
                        let should_exit = handle_key_event(&mut state, key.code)?;
                        if should_exit {
                            break;
                        }
                        needs_redraw = true;
                    }
                }
                Event::Resize(_, _) => {
                    state.prev_screen = None; // Reset the buffer on resize
                    needs_redraw = true;
                }
                _ => {}
            }
        }
    }

    // 4. Teardown TUI Terminal Mode
    execute!(stdout_handle, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;

    // Restore original console window style on teardown
    if let Some(style) = original_style {
        if !hwnd.is_null() {
            unsafe {
                SetWindowLongW(hwnd, GWL_STYLE, style);
                SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
                    0,
                    0,
                    0,
                    0,
                    SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--start" | "/s" | "/S" => {
                let global = registry::load_global_config();
                if !global.active_scr.is_empty() {
                    let path = PathBuf::from(&global.active_scr);
                    if path.exists() {
                        let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or_default().to_lowercase();
                        if filename.contains("ssm") {
                            run_random_cycle();
                        } else {
                            let mut child = std::process::Command::new(path)
                                .arg("/s")
                                .spawn()?;
                            let _ = child.wait();
                        }
                    } else {
                        eprintln!("Error: Active screensaver path does not exist: {}", global.active_scr);
                    }
                } else {
                    eprintln!("Error: No active screensaver is configured.");
                }
                return Ok(());
            }
            "/p" | "/P" => {
                // Ignore preview window request
                return Ok(());
            }
            "/c" | "/C" => {
                // Configure - launch TUI
                run_tui()?;
                return Ok(());
            }
            "--stop" => {
                let targets = [
                    "omaxi-beams.scr", "omaxi-bounce.scr", "omaxi-matrix.scr", 
                    "omaxi-pour.scr", "omaxi-vectors.scr",
                    "omaxi-beams.exe", "omaxi-bounce.exe", "omaxi-matrix.exe", 
                    "omaxi-pour.exe", "omaxi-vectors.exe"
                ];
                for target in &targets {
                    let _ = std::process::Command::new("taskkill")
                        .args(&["/F", "/IM", target])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status();
                }
                println!("Stopped all running Omaxi screensavers.");
                return Ok(());
            }
            "--toggle-active" => {
                let mut global = registry::load_global_config();
                global.active = !global.active;
                if let Err(e) = registry::save_global_config(&global) {
                    eprintln!("Error toggling screensaver: {}", e);
                } else {
                    println!("Screensaver active state set to: {}", global.active);
                }
                return Ok(());
            }
            "--lock" => {
                // Lock windows workstation
                unsafe {
                    LockWorkStation();
                }
                
                // Immediately launch active screensaver fullscreen
                let global = registry::load_global_config();
                if !global.active_scr.is_empty() {
                    let path = PathBuf::from(&global.active_scr);
                    if path.exists() {
                        let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or_default().to_lowercase();
                        if filename.contains("ssm") {
                            run_random_cycle();
                        } else {
                            let mut child = std::process::Command::new(path)
                                .arg("/s")
                                .spawn()?;
                            let _ = child.wait();
                        }
                    }
                }
                return Ok(());
            }
            "--help" | "-h" => {
                println!("SSM: Screen Saver Management v0.1.0");
                println!("Usage:");
                println!("  ssm.exe                     Launch TUI dashboard configuration panel");
                println!("  ssm.exe --start            Launch the active screensaver fullscreen");
                println!("  ssm.exe --stop             Force terminate any running screensavers");
                println!("  ssm.exe --toggle-active    Toggle system screensaver active state");
                println!("  ssm.exe --lock             Lock workstation and start screensaver");
                println!("  ssm.exe --help             Show this help information");
                return Ok(());
            }
            _ => {
                eprintln!("Unknown command line argument: {}", args[1]);
                eprintln!("Run with --help to see usage details.");
                std::process::exit(1);
            }
        }
    }

    run_tui()?;
    Ok(())
}
