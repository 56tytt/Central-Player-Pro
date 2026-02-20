use eframe::egui::{Rgba, Visuals};
use serde::{Deserialize, Serialize}; // ייבוא ישיר של ה-Traits
use std::{fs, io, path::Path};

pub const COLOR_CONFIG_FILENAME: &str = "color_scheme.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ColorScheme {
    pub background: [f32; 3],
    pub foreground: [f32; 3],
    pub accent: [f32; 3],
    pub is_dark_mode: bool,
}

impl Default for ColorScheme {
    fn default() -> Self {
        let light_visuals = Visuals::light();
        let bg_rgba = Rgba::from(light_visuals.panel_fill);
        let fg_rgba = Rgba::from(light_visuals.widgets.noninteractive.fg_stroke.color);
        let accent_rgba = Rgba::from(light_visuals.selection.bg_fill);

        Self {
            background: [bg_rgba.r(), bg_rgba.g(), bg_rgba.b()],
            foreground: [fg_rgba.r(), fg_rgba.g(), fg_rgba.b()],
            accent: [accent_rgba.r(), accent_rgba.g(), accent_rgba.b()],
            is_dark_mode: false,
        }
    }
}

impl ColorScheme {
    pub fn save(&self, path_str: &str) -> Result<(), io::Error> {
        // המרה ל-String תוך שימוש ב-crate של serde_json
        let json = serde_json::to_string_pretty(self).map_err(|e| io::Error::other(e))?;
        fs::write(path_str, json)
    }

    pub fn load(path_str: &str) -> Self {
        if Path::new(path_str).exists()
            && let Ok(content) = fs::read_to_string(path_str)
            && let Ok(scheme) = serde_json::from_str::<Self>(&content)
        {
            return scheme;
        }
        Self::default()
    }
}
