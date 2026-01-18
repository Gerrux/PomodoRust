//! Utility functions

/// Format seconds as human-readable duration
pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Format seconds as MM:SS
pub fn format_timer(seconds: u64) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", mins, secs)
}

/// Format hours with one decimal place
pub fn format_hours(seconds: i64) -> String {
    let hours = seconds as f64 / 3600.0;
    format!("{:.1}h", hours)
}
