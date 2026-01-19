//! Statistics aggregation

use super::Database;

/// Aggregated statistics for display
#[derive(Debug, Clone)]
pub struct Statistics {
    /// Today's total work seconds
    pub today_work_seconds: i64,
    /// Today's completed pomodoros
    pub today_pomodoros: i32,
    /// This week's total work seconds
    pub week_work_seconds: i64,
    /// Daily hours for this week (Mon-Sun)
    pub week_daily_hours: Vec<f32>,
    /// Current streak
    pub current_streak: i32,
    /// Longest streak ever
    pub longest_streak: i32,
    /// Total work seconds (all time)
    pub total_work_seconds: i64,
    /// Total pomodoros (all time)
    pub total_pomodoros: i32,
}

impl Statistics {
    /// Load statistics from database
    pub fn load(db: &Database) -> Self {
        let (today_work_seconds, today_pomodoros) = db.get_today_stats().unwrap_or((0, 0));
        let week_daily_hours = db.get_week_stats().unwrap_or_else(|_| vec![0.0; 7]);
        let week_work_seconds = (week_daily_hours.iter().sum::<f32>() * 3600.0) as i64;
        let (current_streak, longest_streak) = db.get_streak().unwrap_or((0, 0));
        let (total_work_seconds, total_pomodoros) = db.get_total_stats().unwrap_or((0, 0));

        Self {
            today_work_seconds,
            today_pomodoros,
            week_work_seconds,
            week_daily_hours,
            current_streak,
            longest_streak,
            total_work_seconds,
            total_pomodoros,
        }
    }

    /// Create empty statistics (for when database fails)
    pub fn empty() -> Self {
        Self {
            today_work_seconds: 0,
            today_pomodoros: 0,
            week_work_seconds: 0,
            week_daily_hours: vec![0.0; 7],
            current_streak: 0,
            longest_streak: 0,
            total_work_seconds: 0,
            total_pomodoros: 0,
        }
    }

    /// Get today's hours
    pub fn today_hours(&self) -> f32 {
        (self.today_work_seconds as f32 / 3600.0 * 10.0).round() / 10.0
    }

    /// Get this week's hours
    pub fn week_hours(&self) -> f32 {
        (self.week_work_seconds as f32 / 3600.0 * 10.0).round() / 10.0
    }

    /// Get total hours
    pub fn total_hours(&self) -> u32 {
        (self.total_work_seconds / 3600) as u32
    }

    /// Check if daily goal is reached
    pub fn is_daily_goal_reached(&self, target: u32) -> bool {
        self.today_pomodoros >= target as i32
    }

    /// Get daily goal progress (0.0 to 1.0+)
    pub fn daily_goal_progress(&self, target: u32) -> f32 {
        if target == 0 {
            return 1.0;
        }
        self.today_pomodoros as f32 / target as f32
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Self::empty()
    }
}
