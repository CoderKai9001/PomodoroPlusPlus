use chrono::{DateTime, Local, NaiveDate};
use rusqlite::{Connection, Result, params};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: i64,
    pub start_time: String,
    pub end_time: String,
    pub duration: i64,
    pub tag: String,
    pub session_type: String,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path();
        
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        
        let conn = Connection::open(&db_path)?;
        let db = Database { conn };
        db.initialize_schema()?;
        Ok(db)
    }
    
    fn get_db_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pomodoro++")
            .join("pomodoro.db")
    }
    
    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                duration INTEGER NOT NULL,
                tag TEXT NOT NULL,
                type TEXT NOT NULL
            )",
            [],
        )?;
        
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                name TEXT PRIMARY KEY
            )",
            [],
        )?;
        
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;
        
        // Insert default tags if none exist
        let tag_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tags",
            [],
            |row| row.get(0),
        )?;
        
        if tag_count == 0 {
            self.conn.execute("INSERT INTO tags (name) VALUES (?)", ["Work"])?;
            self.conn.execute("INSERT INTO tags (name) VALUES (?)", ["Study"])?;
        }
        
        Ok(())
    }
    
    // Tag operations
    pub fn get_tags(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT name FROM tags ORDER BY name")?;
        let tags = stmt.query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(tags)
    }
    
    pub fn add_tag(&self, name: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?)",
            [name],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn delete_tag(&self, name: &str) -> Result<()> {
        self.conn.execute("DELETE FROM tags WHERE name = ?", [name])?;
        Ok(())
    }
    
    // Session operations
    pub fn save_session(&self, start_time: &DateTime<Local>, end_time: &DateTime<Local>, 
                        duration: i64, tag: &str, session_type: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (start_time, end_time, duration, tag, type) VALUES (?, ?, ?, ?, ?)",
            params![
                start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                duration,
                tag,
                session_type
            ],
        )?;
        Ok(())
    }
    
    // Config operations
    pub fn get_config(&self, key: &str, default: &str) -> String {
        self.conn.query_row(
            "SELECT value FROM config WHERE key = ?",
            [key],
            |row| row.get(0),
        ).unwrap_or_else(|_| default.to_string())
    }
    
    pub fn set_config(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)",
            [key, value],
        )?;
        Ok(())
    }
    
    // Statistics queries
    pub fn get_weekly_stats(&self, tag: Option<&str>) -> Result<Vec<(String, i64)>> {
        let mut results = Vec::new();
        
        if let Some(t) = tag {
            let mut stmt = self.conn.prepare(
                "SELECT DATE(start_time) as day, SUM(duration) as total
                 FROM sessions
                 WHERE tag = ? AND type = 'work' AND start_time >= DATE('now', '-7 days')
                 GROUP BY day
                 ORDER BY day"
            )?;
            let rows = stmt.query_map([t], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;
            for row in rows {
                if let Ok(r) = row {
                    results.push(r);
                }
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT DATE(start_time) as day, SUM(duration) as total
                 FROM sessions
                 WHERE type = 'work' AND start_time >= DATE('now', '-7 days')
                 GROUP BY day
                 ORDER BY day"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;
            for row in rows {
                if let Ok(r) = row {
                    results.push(r);
                }
            }
        }
        
        Ok(results)
    }
    
    pub fn get_monthly_stats(&self, tag: Option<&str>) -> Result<Vec<(String, i64)>> {
        let mut results = Vec::new();
        
        if let Some(t) = tag {
            let mut stmt = self.conn.prepare(
                "SELECT STRFTIME('%Y-%m', start_time) as month, SUM(duration) as total
                 FROM sessions
                 WHERE tag = ? AND type = 'work'
                 GROUP BY month
                 ORDER BY month DESC
                 LIMIT 12"
            )?;
            let rows = stmt.query_map([t], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;
            for row in rows {
                if let Ok(r) = row {
                    results.push(r);
                }
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT STRFTIME('%Y-%m', start_time) as month, SUM(duration) as total
                 FROM sessions
                 WHERE type = 'work'
                 GROUP BY month
                 ORDER BY month DESC
                 LIMIT 12"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;
            for row in rows {
                if let Ok(r) = row {
                    results.push(r);
                }
            }
        }
        
        Ok(results)
    }
    
    pub fn get_heatmap_data(&self) -> Result<Vec<(NaiveDate, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT DATE(start_time) as day, SUM(duration) as total
             FROM sessions
             WHERE type = 'work' AND start_time >= DATE('now', '-180 days')
             GROUP BY day
             ORDER BY day"
        )?;
        
        let rows = stmt.query_map([], |row| {
            let date_str: String = row.get(0)?;
            let total: i64 = row.get(1)?;
            Ok((date_str, total))
        })?;
        
        Ok(rows
            .filter_map(|r| r.ok())
            .filter_map(|(date_str, total)| {
                NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .ok()
                    .map(|d| (d, total))
            })
            .collect())
    }

    #[allow(dead_code)]
    pub fn get_total_today(&self) -> i64 {
        self.conn.query_row(
            "SELECT COALESCE(SUM(duration), 0) FROM sessions 
             WHERE type = 'work' AND DATE(start_time) = DATE('now')",
            [],
            |row| row.get(0),
        ).unwrap_or(0)
    }
}
