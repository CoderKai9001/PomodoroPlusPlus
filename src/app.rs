use std::process::Command;
use crate::db::Database;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Home,
    Stats,
    Heatmap,
    TagInput,
    DeleteConfirm,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PomodoroMode {
    Work,
    Break,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatsView {
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    pub current_screen: Screen,
    pub previous_screen: Screen,
    pub timer_running: bool,
    pub mode: PomodoroMode,
    pub remaining_seconds: u64,
    pub selected_tag_index: usize,
    pub tags: Vec<String>,
    pub work_duration: u64,
    pub break_duration: u64,
    pub db: Database,
    pub should_quit: bool,
    
    // Stats state
    pub stats_view: StatsView,
    pub stats_tag_index: usize, // 0 = All, 1+ = specific tag
    
    // Input state
    pub input_mode: InputMode,
    pub input_buffer: String,
    
    // Session tracking
    pub session_start: Option<chrono::DateTime<chrono::Local>>,
}

impl App {
    pub fn new() -> Result<Self, rusqlite::Error> {
        let db = Database::new()?;
        let tags = db.get_tags()?;
        
        let work_duration: u64 = db.get_config("work_duration", "1500").parse().unwrap_or(1500);
        let break_duration: u64 = db.get_config("break_duration", "300").parse().unwrap_or(300);
        
        Ok(App {
            current_screen: Screen::Home,
            previous_screen: Screen::Home,
            timer_running: false,
            mode: PomodoroMode::Work,
            remaining_seconds: work_duration,
            selected_tag_index: 0,
            tags,
            work_duration,
            break_duration,
            db,
            should_quit: false,
            stats_view: StatsView::Weekly,
            stats_tag_index: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            session_start: None,
        })
    }
    
    pub fn selected_tag(&self) -> Option<&str> {
        self.tags.get(self.selected_tag_index).map(|s| s.as_str())
    }
    
    pub fn toggle_timer(&mut self) {
        if self.timer_running {
            self.timer_running = false;
        } else {
            self.timer_running = true;
            if self.session_start.is_none() {
                self.session_start = Some(chrono::Local::now());
            }
        }
    }
    
    pub fn reset_timer(&mut self) {
        self.timer_running = false;
        self.session_start = None;
        self.remaining_seconds = match self.mode {
            PomodoroMode::Work => self.work_duration,
            PomodoroMode::Break => self.break_duration,
        };
    }
    
    pub fn tick(&mut self) {
        if self.timer_running && self.remaining_seconds > 0 {
            self.remaining_seconds -= 1;
            
            if self.remaining_seconds == 0 {
                self.complete_session();
            }
        }
    }
    
    fn complete_session(&mut self) {
        let now = chrono::Local::now();
        
        // Determine notification message based on current mode (before switching)
        let notification = match self.mode {
            PomodoroMode::Work => ("Pomodoro++", "Work session complete! Time for a break."),
            PomodoroMode::Break => ("Pomodoro++", "Break is over! Back to work."),
        };
        
        if let Some(start) = self.session_start.take() {
            let duration = match self.mode {
                PomodoroMode::Work => self.work_duration as i64,
                PomodoroMode::Break => self.break_duration as i64,
            };
            
            let tag = self.selected_tag().unwrap_or("Work").to_string();
            let session_type = match self.mode {
                PomodoroMode::Work => "work",
                PomodoroMode::Break => "break",
            };
            
            let _ = self.db.save_session(&start, &now, duration, &tag, session_type);
        }
        
        // Play sound and send notification
        Self::play_notification_sound();
        Self::send_notification(notification.0, notification.1);
        
        // Switch mode
        self.mode = match self.mode {
            PomodoroMode::Work => PomodoroMode::Break,
            PomodoroMode::Break => PomodoroMode::Work,
        };
        
        self.remaining_seconds = match self.mode {
            PomodoroMode::Work => self.work_duration,
            PomodoroMode::Break => self.break_duration,
        };
        
        self.timer_running = false;
    }
    
    fn play_notification_sound() {
        // Play sound using paplay in background
        let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/home"));
        let sound_path = format!("{}/Music/sf/vieboom.mp3", home);
        
        std::thread::spawn(move || {
            let _ = Command::new("paplay")
                .arg(&sound_path)
                .spawn();
        });
    }
    
    fn send_notification(title: &str, message: &str) {
        // Send desktop notification using notify-send in background
        let title = title.to_string();
        let msg = message.to_string();
        std::thread::spawn(move || {
            let _ = Command::new("notify-send")
                .arg(&title)
                .arg(&msg)
                .spawn();
        });
    }
    
    pub fn next_tag(&mut self) {
        if !self.tags.is_empty() {
            self.selected_tag_index = (self.selected_tag_index + 1) % self.tags.len();
        }
    }
    
    pub fn prev_tag(&mut self) {
        if !self.tags.is_empty() {
            self.selected_tag_index = if self.selected_tag_index == 0 {
                self.tags.len() - 1
            } else {
                self.selected_tag_index - 1
            };
        }
    }
    
    pub fn add_tag(&mut self, name: String) {
        if !name.is_empty() && !self.tags.contains(&name) {
            let _ = self.db.add_tag(&name);
            self.tags.push(name);
        }
    }
    
    pub fn delete_selected_tag(&mut self) {
        if !self.tags.is_empty() && self.selected_tag_index < self.tags.len() {
            let tag_name = self.tags[self.selected_tag_index].clone();
            let _ = self.db.delete_tag(&tag_name);
            self.tags.remove(self.selected_tag_index);
            
            // Adjust selected index if needed
            if self.selected_tag_index >= self.tags.len() && !self.tags.is_empty() {
                self.selected_tag_index = self.tags.len() - 1;
            }
        }
    }
    
    pub fn get_tag_to_delete(&self) -> Option<&str> {
        self.tags.get(self.selected_tag_index).map(|s| s.as_str())
    }
    
    #[allow(dead_code)]
    pub fn refresh_tags(&mut self) {
        if let Ok(tags) = self.db.get_tags() {
            self.tags = tags;
        }
    }
    
    pub fn navigate_to(&mut self, screen: Screen) {
        self.previous_screen = self.current_screen;
        self.current_screen = screen;
    }
    
    pub fn toggle_stats_view(&mut self) {
        self.stats_view = match self.stats_view {
            StatsView::Weekly => StatsView::Monthly,
            StatsView::Monthly => StatsView::Weekly,
        };
    }
    
    pub fn next_stats_tag(&mut self) {
        self.stats_tag_index = (self.stats_tag_index + 1) % (self.tags.len() + 1);
    }
    
    pub fn prev_stats_tag(&mut self) {
        if self.stats_tag_index == 0 {
            self.stats_tag_index = self.tags.len();
        } else {
            self.stats_tag_index -= 1;
        }
    }
    
    pub fn get_stats_tag(&self) -> Option<&str> {
        if self.stats_tag_index == 0 {
            None
        } else {
            self.tags.get(self.stats_tag_index - 1).map(|s| s.as_str())
        }
    }
    
    pub fn format_time(&self) -> String {
        let minutes = self.remaining_seconds / 60;
        let seconds = self.remaining_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
    
    pub fn adjust_work_duration(&mut self, delta: i64) {
        let new_val = (self.work_duration as i64 + delta).max(60).min(7200) as u64;
        self.work_duration = new_val;
        let _ = self.db.set_config("work_duration", &new_val.to_string());
        
        if self.mode == PomodoroMode::Work && !self.timer_running {
            self.remaining_seconds = new_val;
        }
    }
    
    pub fn adjust_break_duration(&mut self, delta: i64) {
        let new_val = (self.break_duration as i64 + delta).max(60).min(3600) as u64;
        self.break_duration = new_val;
        let _ = self.db.set_config("break_duration", &new_val.to_string());
        
        if self.mode == PomodoroMode::Break && !self.timer_running {
            self.remaining_seconds = new_val;
        }
    }
}
