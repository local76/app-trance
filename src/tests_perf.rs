use std::time::{Duration, Instant};
use ratatui::{backend::TestBackend, Terminal};
use crate::app::App;
use crate::config::{GlobalConfig, LocalConfig};
use crate::theme::TuiTheme;
use crate::ui::render;

#[test]
fn test_ui_rendering_perf_budget() {
    let screensavers = Vec::new();
    let global = GlobalConfig::default();
    let local = LocalConfig::default();
    let theme = TuiTheme::no_color(true);
    let mut app = App::new(screensavers, global, local, theme);
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("Failed to create test terminal");
    
    // Warmup
    terminal.draw(|f| render(&mut app, f)).unwrap();
    
    // Benchmark 100 frames
    const FRAMES: usize = 100;
    let start = Instant::now();
    for _ in 0..FRAMES {
        terminal.draw(|f| render(&mut app, f)).unwrap();
    }
    let elapsed = start.elapsed();
    
    let budget = Duration::from_millis(3000);
    assert!(
        elapsed < budget,
        "100 frames took {:?}, exceeding budget of {:?}",
        elapsed,
        budget
    );
    println!("TUI Render Loop Performance: {} frames in {:?}", FRAMES, elapsed);
}
