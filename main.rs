use eframe::egui::{self, Color32, RichText};

mod app_state;
mod audio_engine;
mod components;
use audio_engine::{AudioEngine, PlayerState};
mod color_config;
mod theme_manager;
use app_state::AppState;
use theme_manager::ThemeManager;
mod equalizer;

// =========================================================
// מבנה האפליקציה
// =========================================================
struct MusicApp {
    volume: f32,
    eq: [f32; 10],
    engine: AudioEngine,
    playlist: Vec<std::path::PathBuf>,
    selected_track: Option<usize>,
    //is_dark_mode: bool,
    theme_manager: ThemeManager, // המנהל החדש
    time_for_animation: f32,
    is_theme_window_open: bool,
    show_about: bool,
    btn_play: Option<egui::TextureHandle>,
    btn_pause: Option<egui::TextureHandle>,
    btn_next: Option<egui::TextureHandle>,
    btn_prev: Option<egui::TextureHandle>,
}

// שינינו מ-impl Default ל-impl רגיל
impl MusicApp {
    // הנה הפונקציה החדשה שמקבלת את ctx!
    pub fn new(ctx: eframe::egui::Context) -> Self {
        // 1. טעינת הזיכרון מהקובץ
        let saved_state = AppState::load();

        // שמנו mut כדי שנוכל לשנות את הווליום ולטעון שיר בהמשך
        let app = Self {
            volume: saved_state.volume,
            eq: [0.0; 10],
            engine: AudioEngine::new(ctx), // עכשיו זה עובד פרפקט!

            playlist: saved_state.playlist,
            selected_track: saved_state.last_played_index,
            theme_manager: ThemeManager::new(),
            time_for_animation: 0.0,
            is_theme_window_open: false,
            show_about: false,
            btn_play: None,
            btn_pause: None,
            btn_next: None,
            btn_prev: None,
        };

        // 2. עדכון המנוע בווליום השמור
        app.engine.set_volume(app.volume);

        // 3. טעינת השיר האחרון - תיקנו פה את שגיאת ה-let chains לסוגריים מקוננים!
        if let Some(idx) = app.selected_track {
            if let Some(path) = app.playlist.get(idx) {
                if let Some(s) = path.to_str() {
                    app.engine.load(s);
                }
            }
        }

        app
    }
}

// =========================================================
// פונקציות עזר (Logic)
// =========================================================
impl MusicApp {
    // 1. Fixed: Added the missing visualizer function inside the impl block
    fn render_visualizer(&self, ui: &mut egui::Ui) {
        let (response, painter) =
            ui.allocate_painter(egui::vec2(ui.available_width(), 30.0), egui::Sense::hover());

        // תיקון ה-Lock: נועלים ישירות את הנתונים בתוך ה-engine
        let data: Vec<f32> = if let Ok(data_lock) = self.engine.spectrum_data.lock() {
            data_lock.clone()
        } else {
            Vec::new()
        };

        if data.is_empty() {
            return;
        }

        let rect = response.rect;
        let data_len = data.len() as f32;
        let bar_width = rect.width() / data_len;
        let bottom_y = rect.max.y;

        let accent_color = self.theme_manager.get_current_accent_color();

        for (i, &db_value) in data.iter().enumerate() {
            // הוספת f32 מפורש כדי שהקומפיילר לא יתבלבל בטיפוסים
            let val: f32 = db_value;
            let height_factor = ((val + 60.0) / 60.0).clamp(0.05, 1.0);
            let bar_height = height_factor * rect.height();

            let x = rect.min.x + (i as f32 * bar_width);
            let y = bottom_y - bar_height;

            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(x + 1.0, y),
                    egui::pos2(x + bar_width - 1.0, bottom_y),
                ),
                // תיקון: שימוש ב-2 בתור u8 (בלי .0)
                egui::CornerRadius::same(2),
                accent_color.linear_multiply(height_factor),
            );
        }
    }

    fn scan_folder_recursive(&mut self, path: &std::path::Path) {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    // תת-תיקייה? סרוק אותה רקורסיבית!
                    self.scan_folder_recursive(&p);
                } else if let Some(ext) = p.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    // תומך בכל הפורמטים שרצית
                    if ["mp3", "wav", "ogg", "flac", "m4a", "mp4"].contains(&ext_str.as_str())
                        && !self.playlist.contains(&p)
                    {
                        self.playlist.push(p);
                    }
                }
            }
        }
    }

    // הפונקציה שתופעל מהכפתור
    fn import_folder_pro(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            println!("📂 Scanning folder: {:?}", path);
            self.scan_folder_recursive(&path);
            println!("✅ Scan complete. Total tracks: {}", self.playlist.len());
        }
    }

    pub fn import_files(&mut self) {
        if let Some(paths) = rfd::FileDialog::new()
            .add_filter("Audio Files", &["mp3", "wav", "ogg", "flac", "m4a"])
            .pick_files()
        {
            for path in paths {
                if !self.playlist.contains(&path) {
                    self.playlist.push(path);
                }
            }
        }
    }

    fn get_track_info(&self) -> (String, String) {
        if let Some(idx) = self.selected_track
            && let Some(path) = self.playlist.get(idx)
        {
            let filename = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            return (filename, "Unknown Artist".to_string());
        }
        ("No Track Selected".to_string(), "".to_string())
    }

    fn play_next(&mut self) {
        if let Some(current_idx) = self.selected_track
            && current_idx < self.playlist.len() - 1
        {
            let next_idx = current_idx + 1;
            self.selected_track = Some(next_idx);
            if let Some(path) = self.playlist.get(next_idx)
                && let Some(path_str) = path.to_str()
            {
                self.engine.load(path_str);
                self.engine.play();
            }
        }
    }
}

// =========================================================
// Main Entry Point
// =========================================================

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Central Player Pro",
        options,
        Box::new(|cc| {
            // 1. יוצרים את האפליקציה מה-Default (טוען זיכרון שמור)
            let mut app = MusicApp::new(cc.egui_ctx.clone());

            // 2. מעדכנים את האייקונים לתוך המשתנה app
            app.btn_play = Some(load_icon(
                &cc.egui_ctx,
                "play",
                include_bytes!("../assets/play.png"),
            ));
            app.btn_pause = Some(load_icon(
                &cc.egui_ctx,
                "pause",
                include_bytes!("../assets/pause.png"),
            ));
            app.btn_next = Some(load_icon(
                &cc.egui_ctx,
                "next",
                include_bytes!("../assets/next.png"),
            ));
            app.btn_prev = Some(load_icon(
                &cc.egui_ctx,
                "prev",
                include_bytes!("../assets/prev.png"),
            ));

            // 3. הגדרת פונטים
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "my_font".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/rb.ttf")).into(),
            );

            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.insert(0, "my_font".to_owned());
            }
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                family.insert(0, "my_font".to_owned());
            }

            cc.egui_ctx.set_fonts(fonts);
            cc.egui_ctx.set_visuals(egui::Visuals::dark());

            // 4. חשוב: מחזירים את ה-app שכבר טענו לתוכה את האייקונים!
            Ok(Box::new(app))
        }),
    )
}

impl eframe::App for MusicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- 1. Logic & Transitions ---

        ctx.input(|i| {
            if !self.playlist.is_empty() {
                // חץ למטה - רד שיר אחד
                if i.key_pressed(egui::Key::ArrowDown) {
                    let current = self.selected_track.unwrap_or(0);
                    if current < self.playlist.len() - 1 {
                        self.selected_track = Some(current + 1);
                    }
                }
                // חץ למעלה - עלה שיר אחד
                if i.key_pressed(egui::Key::ArrowUp) {
                    let current = self.selected_track.unwrap_or(0);
                    if current > 0 {
                        self.selected_track = Some(current - 1);
                    }
                }
                // אינטר - נגן את השיר הנבחר
                if i.key_pressed(egui::Key::Enter) {
                    if let Some(idx) = self.selected_track {
                        if let Some(path) = self.playlist.get(idx) {
                            if let Some(s) = path.to_str() {
                                self.engine.load(s);
                                self.engine.play();
                            }
                        }
                    }
                }
            }
        });

        let engine_eos = self.engine.update();
        let time_is_up = self.engine.current_state == PlayerState::Playing
            && self.engine.current_duration > 0.0
            && self.engine.current_position >= (self.engine.current_duration - 0.5);

        self.theme_manager.apply_theme(ctx);
        self.time_for_animation = ctx.input(|i| i.time as f32);

        if engine_eos || time_is_up {
            self.play_next();
        }

        // --- 2. Top Menu Bar ---
        egui::TopBottomPanel::top("top_menu").show(ctx, |ui: &mut egui::Ui| {
            egui::MenuBar::new().ui(ui, |ui: &mut egui::Ui| {
                ui.menu_button("File", |ui: &mut egui::Ui| {
                    if ui.button("📂 Open File...").clicked() {
                        ui.close();
                    }

                    // --- האופציות החדשות שלך ---
                    if ui.button("➕ Add Audio Files...").clicked() {
                        self.import_files(); // הקריאה לפונקציה שבנינו
                        ui.close();
                    }

                    if ui.button("📂 Add Music Folder (Recursive)...").clicked() {
                        self.import_folder_pro(); // הסריקה העמוקה שבנינו
                        ui.close();
                    }

                    if ui.button("❌ Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("View", |ui: &mut egui::Ui| {
                    ui.menu_button("Theme", |ui: &mut egui::Ui| {
                        if ui.button("Dark Mode").clicked() {
                            self.theme_manager.activate_dark_mode(true);
                            ui.close();
                        }
                        if ui.button("Light Mode").clicked() {
                            self.theme_manager.activate_dark_mode(false);
                            ui.close();
                        }
                        if ui.button("🎨 Theme Settings...").clicked() {
                            self.is_theme_window_open = true;
                            ui.close(); // סוגר את התפריט כי החלון נפתח
                        }

                        ui.separator();
                        ui.label("Accent Color:");

                        let mut current_accent = self.theme_manager.get_current_accent_color();
                        if ui.color_edit_button_srgba(&mut current_accent).changed() {
                            self.theme_manager.set_custom_accent_color(current_accent);
                            ctx.request_repaint();
                        }
                    });
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("ℹ About").clicked() {
                        self.show_about = true;
                        ui.close();
                    }
                });

                // Fixed: Version label moved INSIDE the bar scope
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui: &mut egui::Ui| {
                        ui.label(
                            egui::RichText::new("Central Player Pro v2.5")
                                .weak()
                                .size(10.0),
                        );
                    },
                );
            });
        });

        // --- About Window ---
        if self.show_about {
            egui::Window::new("ℹ About Central Player")
                .open(&mut self.show_about) // זה מטפל לבד בכפתור ה-X לסגירה
                .resizable(false)
                .collapsible(false)
                .default_width(320.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.add_space(10.0);
                    ui.vertical_centered(|ui| {
                        ui.heading(
                            egui::RichText::new("🚀 CENTRAL PLAYER PRO")
                                .size(24.0)
                                .strong()
                                .color(self.theme_manager.get_current_accent_color()), // משתמש בצבע שבחרת!
                        );
                        ui.label(egui::RichText::new("v2.5 Stable").size(14.0).weak());

                        ui.add_space(15.0);
                        ui.label("The ultimate Linux music experience.");
                        ui.label("Built with Rust & GStreamer.");

                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.label("Developed by:");
                        ui.label(egui::RichText::new("Shay Kadosh").strong().size(16.0));
                        ui.label("Software Engineer from Ashkelon 🇮🇱");

                        ui.add_space(5.0);
                        ui.label("AI Partner:");
                        ui.label(
                            egui::RichText::new("Gemini").color(Color32::from_rgb(0, 200, 255)),
                        );

                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.hyperlink_to("🐙 Source Code (GitHub)", "https://github.com/YourRepo");
                        ui.add_space(5.0);
                        ui.hyperlink_to("🦀 Powered by egui", "https://github.com/emilk/egui");
                    });
                    ui.add_space(10.0);
                });
            ctx.request_repaint();
        }

        // --- 3. Player Header & Visualizer ---
        egui::TopBottomPanel::top("player_header")
            .frame(
                egui::Frame::NONE
                    .fill(self.theme_manager.get_current_background_color())
                    .inner_margin(10.0)
                    .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(5))),
            )
            .show(ctx, |ui: &mut egui::Ui| {
                let (title, _artist) = self.get_track_info();
                let accent = self.theme_manager.get_current_accent_color();

                components::draw_compact_header(
                    ui,
                    &mut self.engine,
                    &self.playlist,
                    &mut self.selected_track,
                    &title,
                    accent,
                    // משתמשים ב-as_ref() כדי להפוך Option<T> ל-Option<&T>
                    self.btn_play.as_ref(),
                    self.btn_pause.as_ref(),
                    self.btn_next.as_ref(),
                    self.btn_prev.as_ref(),
                );

                ui.add_space(8.0);
                self.render_visualizer(ui);
                ui.add_space(4.0);
            });

        // --- 4. Status Bar ---
        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(15, 15, 20))
                    .inner_margin(6.0),
            )
            .show(ctx, |ui: &mut egui::Ui| {
                ui.horizontal(|ui: &mut egui::Ui| {
                    match self.engine.current_state {
                        PlayerState::Playing => {
                            ui.label(RichText::new("🔊").color(Color32::GREEN).size(14.0));
                            ui.label(
                                RichText::new("PLAYING")
                                    .color(Color32::GREEN)
                                    .size(11.0)
                                    .strong(),
                            );
                        }
                        PlayerState::Paused => {
                            ui.label(RichText::new("⏸").color(Color32::YELLOW).size(14.0));
                            ui.label(
                                RichText::new("PAUSED")
                                    .color(Color32::YELLOW)
                                    .size(11.0)
                                    .strong(),
                            );
                        }
                        PlayerState::Stopped => {
                            ui.label(RichText::new("⏹").color(Color32::GRAY).size(14.0));
                            ui.label(
                                RichText::new("STOPPED")
                                    .color(Color32::GRAY)
                                    .size(11.0)
                                    .strong(),
                            );
                        }
                        PlayerState::Loading => {
                            ui.label(RichText::new("⏳").color(Color32::LIGHT_BLUE).size(14.0));
                            ui.label(
                                RichText::new("LOADING")
                                    .color(Color32::LIGHT_BLUE)
                                    .size(11.0)
                                    .strong(),
                            );
                        }
                    };

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.label(
                        RichText::new("UHD/4K Mode")
                            .color(Color32::from_rgb(255, 50, 200))
                            .size(15.0),
                    );
                    ui.label(RichText::new("|").color(Color32::from_white_alpha(50)));
                    ui.label(
                        RichText::new("325.1kHz")
                            .color(Color32::from_rgb(0, 255, 255))
                            .size(15.0),
                    );
                    ui.label(
                        RichText::new("GstremerBASS")
                            .color(Color32::from_rgb(50, 255, 50))
                            .size(15.0),
                    );
                    ui.label(RichText::new("|").color(Color32::from_white_alpha(50)));
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui: &mut egui::Ui| {
                            ui.label(
                                RichText::new("v2.5 Stable")
                                    .size(15.0)
                                    .color(Color32::from_rgb(255, 180, 0)),
                            );
                        },
                    );
                });
            });

        // --- 5. Equalizer Panel ---
        egui::TopBottomPanel::bottom("equalizer_panel")
            .resizable(true)
            .min_height(150.0)
            .frame(
                egui::Frame::NONE
                    .fill(if self.theme_manager.is_dark_mode_active() {
                        Color32::from_rgb(25, 27, 33)
                    } else {
                        Color32::from_gray(240)
                    })
                    .inner_margin(10.0),
            )
            .show(ctx, |ui: &mut egui::Ui| {
                components::draw_equalizer(ui, &mut self.eq, &mut self.volume, &mut self.engine);
            });

        // --- 6. Central Panel (Playlist) ---
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            let current_accent = self.theme_manager.get_current_accent_color();
            
            // 2. עכשיו מעבירים אותו לפונקציה בתור הארגומנט ה-5!
            components::draw_playlist(
                ui,
                &mut self.playlist,
                &mut self.selected_track,
                &mut self.engine,
                current_accent, // <--- זה מה שהיה חסר לקומפיילר!
            );
        });
                







        

        // --- 7. Floating Theme Studio Window ---
        if self.is_theme_window_open {
            egui::Window::new("Appearance Studio")
                .open(&mut self.is_theme_window_open)
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("Global Theme");
                    ui.horizontal(|ui| {
                        if ui.button("🌙 Dark Preset").clicked() {
                            self.theme_manager.activate_dark_mode(true);
                            ctx.request_repaint();
                        }
                        if ui.button("☀ Light Preset").clicked() {
                            self.theme_manager.activate_dark_mode(false);
                            ctx.request_repaint();
                        }
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.heading("Custom Colors");
                    ui.add_space(5.0);

                    // Grid Layout for perfect alignment
                    egui::Grid::new("colors_grid")
                        .num_columns(2)
                        .spacing([40.0, 12.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // 1. Background Control
                            ui.label("Background:");
                            let mut bg = self.theme_manager.get_current_background_color();
                            if ui.color_edit_button_srgba(&mut bg).changed() {
                                self.theme_manager.set_custom_background_color(bg);
                                ctx.request_repaint();
                            }
                            ui.end_row();

                            // 2. Text Control
                            ui.label("Text & Icons:");
                            let mut text = self.theme_manager.get_current_text_color();
                            if ui.color_edit_button_srgba(&mut text).changed() {
                                self.theme_manager.set_custom_text_color(text);
                                ctx.request_repaint();
                            }
                            ui.end_row();

                            // 3. Accent Control
                            ui.label("Accent / Buttons:");
                            let mut accent = self.theme_manager.get_current_accent_color();
                            if ui.color_edit_button_srgba(&mut accent).changed() {
                                self.theme_manager.set_custom_accent_color(accent);
                                ctx.request_repaint();
                            }
                            ui.end_row();
                        });

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(5.0);

                    // Reset Button
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        if ui.button("🔄 Reset to Defaults").clicked() {
                            self.theme_manager.activate_dark_mode(true);
                            ctx.request_repaint();
                        }
                    });
                });
        }

        if matches!(
            self.engine.current_state,
            PlayerState::Playing | PlayerState::Loading
        ) {
            ctx.request_repaint();
        }
    }
}

// הוסף את זה בסוף הקובץ main.rs
impl Drop for MusicApp {
    fn drop(&mut self) {
        // המרה של הצבע הנוכחי למערך מספרים לשמירה
        let current_accent = self.theme_manager.get_current_accent_color();
        let accent_array = [current_accent.r(), current_accent.g(), current_accent.b()];

        let state = AppState {
            volume: self.volume,
            playlist: self.playlist.clone(),
            last_played_index: self.selected_track,
            is_dark_mode: self.theme_manager.is_dark_mode_active(),
            accent_color: accent_array,
        };

        state.save();
        println!("💾 App state saved successfully!");
    }
}

// פונקציית עזר לטעינת תמונה מהזיכרון
fn load_icon(ctx: &egui::Context, name: &str, bytes: &[u8]) -> egui::TextureHandle {
    let image = image::load_from_memory(bytes)
        .expect("Failed to load image")
        .to_rgba8();
    let size = [image.width() as _, image.height() as _];
    let pixels = image.as_flat_samples();

    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

    ctx.load_texture(
        name,
        color_image,
        egui::TextureOptions::LINEAR, // פילטר איכותי (מונע פיקסליזציה)
    )
}
