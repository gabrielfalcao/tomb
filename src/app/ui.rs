use super::config::TombConfig;
use regex::Regex;
use tui::style::{Color, Style};

pub fn parse_rgb_hex(color: &str) -> Option<(u8, u8, u8)> {
    let re = Regex::new(r"[#]?([0-9A-Fa-f]{2})([0-9A-Fa-f]{2})([0-9A-Fa-f]{2})").unwrap();
    for cap in re.captures_iter(color) {
        return Some((
            u8::from_str_radix(&cap[1], 16).unwrap(),
            u8::from_str_radix(&cap[2], 16).unwrap(),
            u8::from_str_radix(&cap[3], 16).unwrap(),
        ));
    }
    None
}
#[cfg(test)]
mod tests {
    use super::{parse_rgb_hex, rgb_to_color};
    use tui::style::Color;

    #[test]
    fn test_parse_rgb_hex() {
        assert_eq!(parse_rgb_hex("ffffff"), Some((255, 255, 255)));
        assert_eq!(parse_rgb_hex("#ffffff"), Some((255, 255, 255)));
    }
    #[test]
    fn test_rgb_to_color() {
        assert_eq!(rgb_to_color("ffffff"), Some(Color::Rgb(255, 255, 255)));
        assert_eq!(rgb_to_color("#ffffff"), Some(Color::Rgb(255, 255, 255)));
    }
}

pub fn rgb_to_color(color: &str) -> Option<Color> {
    match parse_rgb_hex(color) {
        Some((r, g, b)) => Some(Color::Rgb(r, g, b)),
        None => None,
    }
}
pub fn color_default() -> Color {
    match TombConfig::load().color_default.to_lowercase().as_str() {
        "blue" => Color::Blue,
        "cyan" => Color::Cyan,
        "green" => Color::Green,
        "magenta" => Color::Magenta,
        "red" => Color::Red,
        "yellow" => Color::Yellow,
        "gray" => Color::DarkGray,
        color => rgb_to_color(color).unwrap_or(color_text()),
    }
}
pub fn color_light() -> Color {
    match TombConfig::load().color_light.to_lowercase().as_str() {
        "blue" => Color::LightBlue,
        "cyan" => Color::LightCyan,
        "green" => Color::LightGreen,
        "magenta" => Color::LightMagenta,
        "red" => Color::LightRed,
        "yellow" => Color::LightYellow,
        "gray" => Color::Gray,
        color => rgb_to_color(color).unwrap_or(color_blurred()),
    }
}
pub fn color_blurred() -> Color {
    Color::DarkGray
}

pub fn color_text() -> Color {
    Color::White
}
pub fn color_error_bg() -> Color {
    Color::Rgb(0, 0, 0)
}
pub fn color_error_fg() -> Color {
    Color::Rgb(155, 155, 155)
}

pub fn block_style() -> Style {
    Style::default().fg(color_default())
}

pub fn paragraph_style() -> Style {
    Style::default().fg(color_blurred())
}
