use crate::app::TempUnit;

/// Format a temperature value with the given unit.
///
/// Used by [App::format_temp] to provide a reusable formatting helper.
#[inline]
pub fn format_temp(value: f32, unit: &TempUnit) -> String {
    let temp = if *unit == TempUnit::Celsius { value }
        else { value * 9.0 / 5.0 + 32.0 };
    if value >= 0.0 {
        format!("+{temp:.0}°")
    } else {
        format!("{temp:.0}°")
    }
}

/// Return the cardinal direction abbreviation (N, NE, E, …) for a given degree value.
///
/// Used by [format_wind_full].
#[inline]
pub fn format_wind_deg(deg: u16) -> &'static str {
    let dirs = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    dirs[((deg as f64 / 45.0) + 0.5) as usize % 8]
}

/// Return a full wind direction string such as `"NE (45°)"`.
///
/// Used by [App::format_wind_direction].
#[inline]
pub fn format_wind_full(deg: u16) -> String {
    let dir = format_wind_deg(deg);
    format!("{dir} ({deg}°)")
}

/// Render a filled/empty block progress bar.
///
/// Used by [App::progress_bar].
#[inline]
pub fn progress_bar(current: u16, max: u16, width: u16) -> String {
    let filled = if max > 0 { (current as f64 / max as f64 * width as f64) as u16 } else { 0 };
    let filled = filled.min(width);
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled as usize), "░".repeat(empty as usize))
}

/// Compute pressure trend from current and 3-hours-ago values.
/// Returns (trend_label, arrow_char).
pub fn format_pressure_trend(current: f32, three_hours_ago: f32) -> (&'static str, &'static str) {
    let diff = current - three_hours_ago;
    if diff > 1.0 {
        ("Rising", "↑")
    } else if diff < -1.0 {
        ("Falling", "↓")
    } else {
        ("Steady", "→")
    }
}
