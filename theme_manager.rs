use crate::color_config::{COLOR_CONFIG_FILENAME, ColorScheme};
use eframe::egui::{Color32, Context, Rgba, Stroke, Visuals};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AppTheme {
    Light,
    Dark,
    Custom,
}

pub struct ThemeManager {
    current_theme: AppTheme,
    pub custom_visuals: Visuals,
    color_scheme_config: ColorScheme,
}

impl ThemeManager {
    pub fn new() -> Self {
        let loaded_scheme = ColorScheme::load(COLOR_CONFIG_FILENAME);

        let mut initial_visuals = if loaded_scheme.is_dark_mode {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        // Apply loaded colors
        initial_visuals.panel_fill = Color32::from_rgb(
            (loaded_scheme.background[0] * 255.0) as u8,
            (loaded_scheme.background[1] * 255.0) as u8,
            (loaded_scheme.background[2] * 255.0) as u8,
        );
        initial_visuals.window_fill = initial_visuals.panel_fill;

        initial_visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(
            (loaded_scheme.foreground[0] * 255.0) as u8,
            (loaded_scheme.foreground[1] * 255.0) as u8,
            (loaded_scheme.foreground[2] * 255.0) as u8,
        );

        // Ensure text color is consistent
        initial_visuals.override_text_color =
            Some(initial_visuals.widgets.noninteractive.fg_stroke.color);

        let accent_color32 = Color32::from_rgb(
            (loaded_scheme.accent[0] * 255.0) as u8,
            (loaded_scheme.accent[1] * 255.0) as u8,
            (loaded_scheme.accent[2] * 255.0) as u8,
        );

        // Apply accent to selection and active widgets
        initial_visuals.selection.bg_fill = accent_color32;
        initial_visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, accent_color32);
        initial_visuals.widgets.active.bg_fill = accent_color32;
        initial_visuals.widgets.open.bg_fill = accent_color32;

        let initial_theme = if loaded_scheme.background == ColorScheme::default().background
            && loaded_scheme.foreground == ColorScheme::default().foreground
            && loaded_scheme.accent == ColorScheme::default().accent
        {
            if loaded_scheme.is_dark_mode {
                AppTheme::Dark
            } else {
                AppTheme::Light
            }
        } else {
            AppTheme::Custom
        };

        Self {
            current_theme: initial_theme,
            custom_visuals: initial_visuals,
            color_scheme_config: loaded_scheme,
        }
    }

    fn save_current_scheme(&mut self) {
        let bg_rgba = Rgba::from(self.custom_visuals.panel_fill);
        self.color_scheme_config.background = [bg_rgba.r(), bg_rgba.g(), bg_rgba.b()];

        // Get color from override or fallback to stroke
        let text_color = self
            .custom_visuals
            .override_text_color
            .unwrap_or(self.custom_visuals.widgets.noninteractive.fg_stroke.color);
        let fg_rgba = Rgba::from(text_color);
        self.color_scheme_config.foreground = [fg_rgba.r(), fg_rgba.g(), fg_rgba.b()];

        let accent_rgba = Rgba::from(self.custom_visuals.selection.bg_fill);
        self.color_scheme_config.accent = [accent_rgba.r(), accent_rgba.g(), accent_rgba.b()];

        self.color_scheme_config.is_dark_mode = self.is_dark_mode_active();

        if let Err(e) = self.color_scheme_config.save(COLOR_CONFIG_FILENAME) {
            eprintln!("Failed to save color scheme: {}", e);
        }
    }

    pub fn is_dark_mode_active(&self) -> bool {
        match self.current_theme {
            AppTheme::Dark => true,
            AppTheme::Light => false,
            AppTheme::Custom => self.custom_visuals.dark_mode,
        }
    }

    pub fn activate_dark_mode(&mut self, dark_mode_active: bool) {
        if dark_mode_active {
            self.current_theme = AppTheme::Dark;
            self.custom_visuals = Visuals::dark();
        } else {
            self.current_theme = AppTheme::Light;
            self.custom_visuals = Visuals::light();
        }
        self.save_current_scheme();
    }

    // --- New Functions for Full Control ---

    pub fn set_custom_background_color(&mut self, color: Color32) {
        self.custom_visuals.panel_fill = color;
        self.custom_visuals.window_fill = color;
        self.custom_visuals.faint_bg_color = color;

        self.current_theme = AppTheme::Custom;
        self.save_current_scheme();
    }

    pub fn set_custom_text_color(&mut self, color: Color32) {
        self.custom_visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, color);
        self.custom_visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, color);
        self.custom_visuals.override_text_color = Some(color);

        self.current_theme = AppTheme::Custom;
        self.save_current_scheme();
    }

    pub fn set_custom_accent_color(&mut self, color: Color32) {
        self.custom_visuals.selection.bg_fill = color;
        self.custom_visuals.widgets.active.bg_fill = color;
        self.custom_visuals.widgets.open.bg_fill = color;
        self.custom_visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, color);

        self.current_theme = AppTheme::Custom;
        self.save_current_scheme();
    }

    pub fn get_current_background_color(&self) -> Color32 {
        self.custom_visuals.panel_fill
    }

    pub fn get_current_text_color(&self) -> Color32 {
        self.custom_visuals
            .override_text_color
            .unwrap_or(self.custom_visuals.widgets.noninteractive.fg_stroke.color)
    }

    pub fn get_current_accent_color(&self) -> Color32 {
        self.custom_visuals.selection.bg_fill
    }

    pub fn apply_theme(&self, ctx: &Context) {
        ctx.set_visuals(self.custom_visuals.clone());
    }
}
