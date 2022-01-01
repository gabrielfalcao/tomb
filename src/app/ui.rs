use super::config::TombConfig;
use tui::style::Color;

pub fn color_default() -> Color {
    TombConfig::load().ui_color_default()
}
pub fn color_light() -> Color {
    TombConfig::load().ui_color_light()
}
