use crossterm::style::Color;

#[derive(Clone, Copy)]
pub struct TuiTheme {
    pub border: Color,
    pub header: Color,
    pub accent_primary: Color,
    pub accent_secondary: Color,
    pub text_main: Color,
    pub text_dim: Color,
}

pub fn get_theme() -> TuiTheme {
    TuiTheme {
        border: Color::DarkGrey,
        header: Color::Cyan,
        accent_primary: Color::Cyan,
        accent_secondary: Color::Yellow,
        text_main: Color::White,
        text_dim: Color::DarkGrey,
    }
}
