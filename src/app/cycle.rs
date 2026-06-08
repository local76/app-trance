//! Screensaver cycle execution loops and markdown text parsing utilities.
//!
//! **Taxonomy Classification**: Interface (TUI / Cycle & Document Parsing).

use std::path::{Path, PathBuf};
use crate::config::LocalConfig;
use crate::theme::TuiTheme;

/// Convenience: kick off the random cycle and return when it finishes.
pub fn run_random_cycle() {
    let local_config = LocalConfig::load();
    let exe = std::env::current_exe().ok();

    let candidates: Vec<PathBuf> = local_config.selected_paths
        .iter()
        .map(PathBuf::from)
        .filter(|p| p.exists() && !is_self(p, exe.as_ref()) && !is_uninstall(p))
        .collect();

    if candidates.is_empty() {
        return;
    }

    let cycle_duration = std::time::Duration::from_secs(local_config.random_cycle_secs as u64);

    let mut seed: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut mask = None;
    loop {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let idx = (seed as usize) % candidates.len();
        let target = &candidates[idx];
        let mut child = match std::process::Command::new(target).arg("/s").spawn() {
            Ok(c) => c,
            Err(_) => break,
        };

        if mask.is_some() {
            std::thread::sleep(std::time::Duration::from_millis(300));
            let _ = mask.take();
        }

        let start = std::time::Instant::now();
        let mut exited = false;
        while start.elapsed() < cycle_duration {
            match child.try_wait() {
                Ok(Some(_)) => {
                    exited = true;
                    break;
                }
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                Err(_) => {
                    exited = true;
                    break;
                }
            }
        }
        if exited {
            break;
        }
        mask = crate::win32::CycleMask::new();
        std::thread::sleep(std::time::Duration::from_millis(50));
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/PID", &child.id().to_string()])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();

            let start_wait = std::time::Instant::now();
            let mut grace_exit = false;
            while start_wait.elapsed() < std::time::Duration::from_millis(300) {
                if let Ok(Some(_)) = child.try_wait() {
                    grace_exit = true;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            if !grace_exit {
                let _ = child.kill();
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = child.kill();
        }
    }
}

pub fn is_self(p: &PathBuf, exe: Option<&PathBuf>) -> bool {
    exe.map(|e| e == p).unwrap_or(false)
}

pub fn is_uninstall(p: &Path) -> bool {
    p.file_name()
        .and_then(|f| f.to_str())
        .map(str::to_lowercase)
        .map(|n| n.contains("uninstall"))
        .unwrap_or(false)
}

/// Parse a raw markdown string into styled Ratatui Line objects.
pub fn parse_markdown_to_lines(content: &str, theme: &TuiTheme) -> Vec<ratatui::text::Line<'static>> {
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};

    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut current_paragraph = String::new();

    let flush_paragraph = |para: &mut String, lines: &mut Vec<Line<'static>>| {
        if !para.is_empty() {
            lines.push(Line::from(Span::styled(
                para.clone(),
                Style::default().fg(theme.text_main),
            )));
            para.clear();
        }
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::Rgb(150, 240, 150)),
            )));
            continue;
        }

        if trimmed.is_empty() {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(""));
            continue;
        }

        if let Some(header) = trimmed.strip_prefix("# ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("=== {} ===", header.to_uppercase()),
                Style::default()
                    .fg(theme.accent_primary)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        } else if let Some(header) = trimmed.strip_prefix("## ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("--- {} ---", header),
                Style::default()
                    .fg(theme.accent_primary)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        } else if let Some(header) = trimmed.strip_prefix("### ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(Span::styled(
                header.to_string(),
                Style::default().fg(theme.accent_primary),
            )));
        } else if let Some(item) = trimmed.strip_prefix("* ").or_else(|| trimmed.strip_prefix("- ")) {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(vec![
                Span::styled(" • ", Style::default().fg(theme.accent_primary)),
                Span::styled(item.to_string(), Style::default().fg(theme.text_main)),
            ]));
        } else if trimmed.starts_with("1. ")
            || trimmed.starts_with("2. ")
            || trimmed.starts_with("3. ")
            || trimmed.starts_with("4. ")
            || trimmed.starts_with("5. ")
        {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} ", &trimmed[..3]),
                    Style::default().fg(theme.accent_primary),
                ),
                Span::styled(trimmed[3..].to_string(), Style::default().fg(theme.text_main)),
            ]));
        } else {
            if !current_paragraph.is_empty() {
                current_paragraph.push(' ');
            }
            current_paragraph.push_str(trimmed);
        }
    }

    flush_paragraph(&mut current_paragraph, &mut lines);
    lines
}
