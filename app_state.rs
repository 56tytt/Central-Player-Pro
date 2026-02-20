use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const STATE_FILENAME: &str = "player_state.json";

// שים לב ל-pub כאן! בלי זה, ה-main לא יכול לראות את זה.
#[derive(Serialize, Deserialize, Debug)]
pub struct AppState {
    pub volume: f32,
    pub playlist: Vec<PathBuf>,
    pub last_played_index: Option<usize>,
    pub is_dark_mode: bool,    // הוספנו גם את זה
    pub accent_color: [u8; 3], // הוספנו שמירת צבע
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            volume: 0.5,
            playlist: Vec::new(),
            last_played_index: None,
            is_dark_mode: true,
            accent_color: [0, 255, 0], // ירוק דיפולטיבי
        }
    }
}

impl AppState {
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(STATE_FILENAME)
            && let Ok(state) = serde_json::from_str(&content)
        {
            return state;
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(STATE_FILENAME, json);
        }
    }
}
