pub struct Playlist {
    pub items: Vec<String>,
    pub current_index: Option<usize>,
}

impl Playlist {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            current_index: None,
        }
    }

    /// הוספת שיר לרשימה
    pub fn add(&mut self, path: String) {
        self.items.push(path);
        // אם זה השיר הראשון, נגדיר אותו כנוכחי
        if self.current_index.is_none() {
            self.current_index = Some(0);
        }
    }

    /// קבלת הנתיב של השיר הנוכחי
    pub fn get_current(&self) -> Option<&String> {
        self.current_index.and_then(|idx| self.items.get(idx))
    }

    /// מעבר לשיר הבא (מחזיר את הנתיב אם קיים)
    pub fn next(&mut self) -> Option<String> {
        if let Some(idx) = self.current_index {
            if idx + 1 < self.items.len() {
                self.current_index = Some(idx + 1);
                return self.get_current().cloned();
            }
        }
        None
    }

    /// מעבר לשיר הקודם
    pub fn previous(&mut self) -> Option<String> {
        if let Some(idx) = self.current_index {
            if idx > 0 {
                self.current_index = Some(idx - 1);
                return self.get_current().cloned();
            }
        }
        None
    }

    /// בחירת שיר ספציפי לפי אינדקס
    pub fn select(&mut self, index: usize) -> Option<String> {
        if index < self.items.len() {
            self.current_index = Some(index);
            return self.get_current().cloned();
        }
        None
    }

    /// הסרת שיר מהרשימה
    pub fn remove(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
            // תיקון האינדקס הנוכחי אם צריך
            if let Some(curr) = self.current_index {
                if curr >= self.items.len() && !self.items.is_empty() {
                    self.current_index = Some(self.items.len() - 1);
                } else if self.items.is_empty() {
                    self.current_index = None;
                }
            }
        }
    }
}