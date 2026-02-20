use crate::audio_engine::{AudioEngine, PlayerState};
use eframe::egui;
use eframe::egui::{
    Align, Color32, CornerRadius, Layout, Pos2, Rect, RichText, Sense, Stroke, Vec2,
};
use std::path::PathBuf;   //COMPONENETS RS 
                          //COMPONENETS RS 
// =========================================================
// 1. ה-HEADER המקצועי (שתי קומות)
pub fn draw_compact_header(
    ui: &mut egui::Ui,
    engine: &mut AudioEngine,
    playlist: &Vec<std::path::PathBuf>,
    selected_track: &mut Option<usize>,
    title: &str,
    accent_color: egui::Color32,
    icon_play: Option<&egui::TextureHandle>,
    icon_pause: Option<&egui::TextureHandle>,
    icon_next: Option<&egui::TextureHandle>,
    icon_prev: Option<&egui::TextureHandle>,
) {
    ui.vertical(|ui| {
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            // הגדרת גדלים - בדיוק כמו שאהבת
            let btn_size = egui::Vec2::new(45.0, 45.0);
            let main_btn_size = egui::Vec2::new(45.0, 45.0);

            // --- כפתור הקודם ---
            if let Some(tex) = icon_prev {
                let prev_btn = egui::Button::image(
                    egui::Image::new(tex)
                        .fit_to_exact_size(btn_size)
                        .tint(egui::Color32::LIGHT_GRAY), // <-- הטינט זז לפה! לתוך הסוגריים
                ) // <-- הסוגריים של ה-image נסגרים כאן
                .fill(egui::Color32::TRANSPARENT)
                .frame(false); // מחקנו את הטינט מפה

                if ui.add(prev_btn).clicked() {
                    if let Some(idx) = *selected_track {
                        if idx > 0 {
                            *selected_track = Some(idx - 1);
                            load_track(playlist, idx - 1, engine);
                        }
                    }
                }
            }

            ui.add_space(10.0);

            // --- כפתור ראשי (PLAY/PAUSE) ---
            let is_playing = engine.current_state == PlayerState::Playing;
            let current_icon_opt = if is_playing { icon_pause } else { icon_play };

            if let Some(tex) = current_icon_opt {
                let play_btn = egui::Button::image(
                    egui::Image::new(tex)
                        .fit_to_exact_size(main_btn_size)
                        .tint(accent_color), // הצבע הזוהר שלך עבר לכאן!
                ) // <-- כאן נסגרים הסוגריים של ה-image
                .fill(egui::Color32::TRANSPARENT)
                .frame(false); // מחקנו את הטינט מפה

                if ui.add(play_btn).clicked() {
                    if is_playing {
                        engine.pause();
                    } else {
                        engine.play();
                    }
                }
            }
            ui.add_space(10.0);

            // --- כפתור הבא ---
            if let Some(tex) = icon_next {
                let next_btn = egui::Button::image(
                    egui::Image::new(tex)
                        .fit_to_exact_size(btn_size)
                        .tint(egui::Color32::LIGHT_GRAY), // הטינט עבר לתוך הסוגריים של ה-image!
                ) // <-- כאן נסגרים הסוגריים
                .fill(egui::Color32::TRANSPARENT)
                .frame(false);

                if ui.add(next_btn).clicked() {
                    if let Some(idx) = *selected_track {
                        if idx < playlist.len() - 1 {
                            *selected_track = Some(idx + 1);
                            load_track(playlist, idx + 1, engine);
                        }
                    }
                }
            }

            ui.add_space(15.0);

            // כותרת השיר
            ui.label(
                egui::RichText::new(title)
                    .size(18.0)
                    .strong()
                    .color(egui::Color32::WHITE),
            );
        });

        ui.add_space(12.0);

        // הפרוגרס בר הצהוב
        draw_progress_bar(ui, engine, accent_color);

        ui.add_space(5.0);
    });
}


// 2. פרוגרס בר מעוצב (GLOW EFFECT)
// =========================================================
pub fn draw_progress_bar(ui: &mut egui::Ui, engine: &mut AudioEngine, color: Color32) {
    let duration = engine.current_duration;
    let mut position = engine.current_position;

    // --- חיתוך כירוגי 1: שולפים מיקום זמני אם העכבר גורר עכשיו ---
    let id = ui.id().with("seek_drag");
    if let Some(drag_pos) = ui.data_mut(|d| d.get_temp::<f32>(id)) {
        position = drag_pos as f64; // נציג את הגרירה החלקה במקום את המנוע!
    }

    // משתמשים ב-Grid או Horizontal כדי לשים זמנים בצדדים
    ui.horizontal(|ui| {
        // זמן נוכחי (עכשיו יזוז חלק עם העכבר)
        ui.label(RichText::new(format_time(position as f64)).size(11.0).color(color));

        // חישוב רוחב: לוקחים את כל מה שנשאר פחות המקום לזמן בצד השני
        let available_width = ui.available_width() - 40.0;
        let height = 6.0; // עובי הבר

        let (rect, response) =
            ui.allocate_at_least(Vec2::new(available_width, 10.0), Sense::click_and_drag());

        // 1. ציור הרקע (הפס האפור הכהה)
        let bg_rect = Rect::from_min_size(
            Pos2::new(rect.min.x, rect.center().y - height / 2.0),
            Vec2::new(rect.width(), height),
        );
        ui.painter().rect_filled(
            bg_rect,
            CornerRadius::same(3),
            Color32::from_white_alpha(20),
        );

        // 2. ציור המילוי (הפס הצבעוני)
        if duration > 0.0 {
            let percent = (position / duration).clamp(0.0, 1.0);
            let fill_width = rect.width() * percent as f32;

            let fill_rect = Rect::from_min_size(bg_rect.min, Vec2::new(fill_width, height));
            ui.painter()
                .rect_filled(fill_rect, CornerRadius::same(3), color);

            // 3. ה"ידית" (Handle) עם אפקט זוהר
            let handle_center = Pos2::new(bg_rect.min.x + fill_width, bg_rect.center().y);

            // הילה שקופה גדולה
            ui.painter()
                .circle_filled(handle_center, 10.0, color.gamma_multiply(0.2));
            // הילה קטנה יותר
            ui.painter()
                .circle_filled(handle_center, 6.0, color.gamma_multiply(0.5));
            // נקודה לבנה במרכז
            ui.painter()
                .circle_filled(handle_center, 3.0, Color32::WHITE);
        }

        // --- חיתוך כירוגי 2: הלוגיקה החכמה של הגרירה ---
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let p = ((pointer_pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
            let target_time = p as f64 * duration;

            if response.dragged() {
                // רק שומרים בזיכרון ה-UI, לא משגעים את המנוע!
                ui.data_mut(|d| d.insert_temp(id, target_time));
            }

            // שולחים פקודה למנוע מכה אחת בולטת *רק* בעזיבת עכבר או קליק
            if response.drag_stopped() || response.clicked() {
                engine.seek(target_time as f32);
                ui.data_mut(|d| d.remove_temp::<f32>(id)); // מנקים את הזיכרון
            }
        } else if !response.dragged() {
            // אם העכבר ברח משם, נוודא שהזיכרון נקי
            ui.data_mut(|d| d.remove_temp::<f32>(id));
        }

        // זמן סיום
        ui.label(
            RichText::new(format_time(duration as f64))
                .size(11.0)
                .color(Color32::GRAY),
        );
    });
}



fn load_track(playlist: &Vec<PathBuf>, idx: usize, engine: &mut AudioEngine) {
    if let Some(path) = playlist.get(idx)
        && let Some(s) = path.to_str()
    {
        engine.load(s);
        engine.play();
    }
}

fn format_time(seconds: f64) -> String {
    let minutes = (seconds / 60.0).floor() as u64;
    let secs = (seconds % 60.0).floor() as u64;
    format!("{:02}:{:02}", minutes, secs)
}

///======================================// בתוך src/components.rs

pub fn draw_equalizer(
    ui: &mut egui::Ui,
    eq: &mut [f32; 10],
    volume: &mut f32,
    engine: &mut AudioEngine,
) {
    egui::Frame::NONE
        .fill(Color32::from_rgb(20, 22, 26))
        .corner_radius(12)
        .stroke(Stroke::new(1.0, Color32::from_white_alpha(15)))
        .inner_margin(15.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("🎚 EQ & MASTER")
                        .strong()
                        .color(Color32::LIGHT_GRAY),
                );

                // --- התפריט הנפתח ל-Presets ---
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("eq_presets")
                        .selected_text(RichText::new("Presets 🎛").strong())
                        .show_ui(ui, |ui| {
                            for &preset in crate::equalizer::Equalizer::preset_names().iter() {
                                if ui.selectable_label(false, preset).clicked() {
                                    *eq = crate::equalizer::Equalizer::get_preset(preset);
                                    engine.update_eq(*eq);
                                    println!("UI: Applied EQ Preset - {}", preset);
                                }
                            }
                        });
                });
            });

            ui.add_space(10.0);

            // --- העמודות של הסליידרים ---
            ui.columns(11, |columns| {
                for (i, val) in eq.iter_mut().enumerate() {
                    columns[i].vertical_centered(|ui| {
                        let color = match i {
                            0..=2 => Color32::from_rgb(80, 200, 255),
                            3..=6 => Color32::from_rgb(150, 255, 150),
                            _ => Color32::from_rgb(255, 160, 100),
                        };

                        if ui
                            .add(
                                egui::Slider::new(val, -24.0..=12.0)
                                    .vertical()
                                    .show_value(false)
                                    .trailing_fill(true),
                            )
                            .changed()
                        {
                            engine.set_eq(i, *val as f64);
                        }

                        let freq = match i {
                            0 => "29",
                            1 => "59",
                            2 => "119",
                            3 => "237",
                            4 => "474",
                            5 => "947",
                            6 => "1.8k",
                            7 => "3.7k",
                            8 => "7.5k",
                            9 => "15k",
                            _ => "",
                        };
                        ui.label(RichText::new(freq).size(9.0).color(color));
                    });
                }

                // --- סליידר הווליום (העמודה ה-11) ---
                columns[10].vertical_centered(|ui| {
                    // 1. משנים את צבע המילוי רק לעמודה הזו! (למשל: זהב/צהוב זוהר)
                    let vol_color = Color32::from_rgb(255, 200, 0);
                    ui.visuals_mut().selection.bg_fill = vol_color;

                    if ui
                        .add(
                            egui::Slider::new(volume, 0.0..=1.0)
                                .vertical()
                                .show_value(false)
                                .trailing_fill(true), // 2. חובה להוסיף את זה כדי שהצבע ייראה!
                        )
                        .changed()
                    {
                        engine.set_volume(*volume);
                        println!("UI: Volume changed to {:.1}", volume);
                    }

                    // 3. צובעים גם את הטקסט "VOL" למטה באותו צבע
                    ui.label(RichText::new("VOL").size(11.0).strong().color(vol_color));
                });
            });
        });
}

pub fn draw_playlist(
    ui: &mut egui::Ui,
    playlist: &mut Vec<std::path::PathBuf>,
    selected_track: &mut Option<usize>,
    engine: &mut AudioEngine,
    accent_color: Color32, // מביאים את הצבע מה-main!
) {
    ui.add_space(5.0);

    // --- כותרת וכפתור הוספה ---
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("🎵 MY LIBRARY")
                .strong()
                .size(14.0)
                .color(Color32::LIGHT_GRAY),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button(RichText::new("➕ Add Files").size(12.0)).clicked()
                && let Some(paths) = rfd::FileDialog::new()
                    .add_filter("Audio", &["mp3", "wav", "flac", "ogg", "aac", "m4a", "mp4"])
                    .pick_files()
            {
                for path in paths {
                    playlist.push(path);
                }
            }
        });
    });

    ui.add_space(5.0);
    ui.separator();

    // --- אזור הפלייליסט ---
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for (idx, path) in playlist.iter().enumerate() {
                let name = path.file_stem().unwrap_or_default().to_string_lossy();
                let is_selected = Some(idx) == *selected_track;

                // 1. תיקון Frame: שימוש ב-Frame::NONE וב-i8 עבור Margin, ו-corner_radius
                let mut frame = egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(10, 8)) // <-- תוקן ל-i8 (בלי .0)
                    .corner_radius(8); // <-- תוקן מ-rounding

                if is_selected {
                    frame = frame
                        .stroke(Stroke::new(2.0, accent_color))
                        .fill(accent_color.gamma_multiply(0.08));
                } else {
                    frame = frame.stroke(Stroke::new(2.0, Color32::TRANSPARENT));
                }

                // 2. ציור המסגרת
                let frame_response = frame.show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    
                    let text_color = if is_selected { accent_color } else { Color32::LIGHT_GRAY };
                    let prefix = if is_selected { "▶" } else { " " };
                    
                    ui.label(
                        RichText::new(format!("{}  {}", prefix, name))
                            .color(text_color)
                            .size(14.0)
                    );
                }).response;

                // 3. אינטראקציה ואפקט Hover
                let interact_response = ui.interact(frame_response.rect, ui.id().with(idx), egui::Sense::click());
                
                if interact_response.hovered() && !is_selected {
                    // תיקון StrokeKind: הוספת הפרמטר החסר
                    ui.painter().rect_stroke(
                        frame_response.rect,
                        8.0,
                        Stroke::new(1.0, Color32::from_gray(100)),
                        egui::StrokeKind::Inside, // <-- תוקן: הוסף הפרמטר הרביעי ש-egui 0.33 דורש
                    );
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if interact_response.clicked() {
                    *selected_track = Some(idx);
                    load_track(playlist, idx, engine);
                }
            }
        });
}


