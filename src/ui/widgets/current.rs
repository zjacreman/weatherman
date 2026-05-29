use crate::app::{App, WmoWeather};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

/// Simple greedy word-wrap: breaks `text` into lines of at most `width` display columns,
/// never splitting a word in the middle.
fn wrap_text(text: &str, width: u16) -> Vec<String> {
    let width = width as usize;
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    let mut current_line = String::new();
    for word in text.split_whitespace() {
        let extra = if current_line.is_empty() { 0 } else { 1 };
        if current_line.len() + extra + word.len() > width {
            if !current_line.is_empty() {
                lines.push(std::mem::take(&mut current_line));
            }
            if word.len() > width {
                // Word itself exceeds width; split it (rare for normal location names)
                let mut remaining = word;
                while remaining.len() > width {
                    let (chunk, rest) = remaining.split_at(width);
                    lines.push(chunk.to_string());
                    remaining = rest;
                }
                current_line = remaining.to_string();
            } else {
                current_line = word.to_string();
            }
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    if lines.is_empty() {
        lines.push(text.to_string());
    }
    lines
}

fn uv_color(uv: f32) -> Color {
    if uv <= 2.0 {
        Color::Rgb(76, 175, 80)       // green — low
    } else if uv <= 5.0 {
        Color::Rgb(255, 235, 59)      // yellow — moderate
    } else if uv <= 7.0 {
        Color::Rgb(255, 152, 0)       // orange — high
    } else if uv <= 10.0 {
        Color::Rgb(244, 67, 54)       // red — very high
    } else {
        Color::Rgb(156, 39, 176)      // purple — extreme
    }
}

fn visibility_color(vis_m: f32) -> Color {
    if vis_m >= 10000.0 {
        Color::Rgb(76, 175, 80)       // green — excellent
    } else if vis_m >= 5000.0 {
        Color::Rgb(255, 235, 59)      // yellow — moderate
    } else {
        Color::Rgb(244, 67, 54)       // red — poor
    }
}

pub struct CurrentWidget<'a> {
    pub app: &'a App,
}

impl Widget for CurrentWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Current Conditions ")
            .title_alignment(Alignment::Center);
        block.render(area, buf);

        let content_area = area.inner(Margin::new(3, 1));

        if self.app.location.is_none() || self.app.current.is_none() {
            Paragraph::new("No location selected")
                .alignment(Alignment::Center)
                .render(content_area, buf);
            return;
        }

        let current = self.app.current.as_ref().unwrap();
        let location = self.app.location.as_ref().unwrap();
        let hourly_pressures = self.app.hourly.as_ref().and_then(|h| h.pressures.as_ref());
        let wmo = WmoWeather::from(current.weather_code);

        let temp = self.app.format_temp(current.temperature);
        let apparent = self.app.format_temp(current.apparent_temperature);
        let wind = self.app.format_wind_speed(current.wind_speed);
        let wind_dir = self.app.format_wind_direction(current.wind_direction);
        let gusts = self.app.format_wind_speed(current.wind_gusts);
        // Build the content as a vector of styled lines
        let precip_line = if current.precipitation > 0.0 {
            Some(Line::from(vec![
                Span::styled("Precip:", Style::default().fg(Color::Gray)),
                Span::raw("  "),
                Span::styled(
                    format!("{:.1} mm", current.precipitation),
                    Style::default().fg(Color::Rgb(100, 149, 237)),
                ),
            ]))
        } else {
            None
        };

        let humidity_bar = self.app.progress_bar(
            current.humidity as u16,
            100,
            content_area.width.saturating_sub(12),
        );

        let mut lines: Vec<Line> = vec![];
        // Location name (prominent), wrapped cleanly on word boundaries
        for wrapped in wrap_text(&location.display_name(), content_area.width) {
            lines.push(Line::from(vec![Span::styled(
                wrapped,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]));
        }
        lines.extend(vec![
            // Weather icon and condition
            Line::from(vec![
                Span::styled(wmo.icon(current.is_day), Style::default().fg(wmo.color())),
                Span::raw("  "),
                Span::styled(wmo.description(), Style::default().fg(wmo.color())),
            ]),
            // Empty line for spacing
            Line::from(""),
            // Temperature section
            Line::from(vec![
                Span::styled("Temperature:", Style::default().fg(Color::Gray)),
                Span::raw("  "),
                Span::styled(
                    temp,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            // Apparent temperature
            Line::from(vec![
                Span::styled("Apparent:", Style::default().fg(Color::Gray)),
                Span::raw("  "),
                Span::styled(
                    apparent,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            // Humidity with progress bar
            Line::from(vec![
                Span::styled("Humidity:", Style::default().fg(Color::Gray)),
                Span::raw("  "),
                Span::styled(
                    format!("{:>3}%", current.humidity),
                    Style::default().fg(Color::White),
                ),
                Span::raw("  "),
                Span::raw(humidity_bar),
            ]),
            // Empty line for spacing
            Line::from(""),
            // Wind section
            Line::from(vec![
                Span::styled("Wind:", Style::default().fg(Color::Gray)),
                Span::raw("  "),
                Span::raw(wind),
                Span::raw("  "),
                Span::styled(wind_dir, Style::default().fg(Color::Yellow)),
            ]),
            // Wind gusts
            Line::from(vec![
                Span::styled("  Gusts:", Style::default().fg(Color::Gray)),
                Span::raw("  "),
                Span::styled(gusts, Style::default().fg(Color::Yellow)),
            ]),
        ]);

        // Conditionally add precipitation line
        if let Some(line) = precip_line {
            lines.push(line);
        }

        // Pressure with trend
        if let Some(pressure) = current.pressure {
            let trend = if let Some(pressures) = hourly_pressures {
                if pressures.len() >= 4 {
                    let prev = pressures[pressures.len() - 4];
                    crate::ui::helpers::format_pressure_trend(pressure, prev)
                } else {
                    ("—", "")
                }
            } else {
                ("—", "")
            };
            lines.push(Line::from(vec![
                Span::styled("Pressure:", Style::default().fg(Color::Gray)),
                Span::raw("  "),
                Span::styled(
                    format!("{:.0} hPa", pressure),
                    Style::default().fg(Color::White),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{} {}", trend.1, trend.0),
                    Style::default().fg(if trend.0 == "Rising" {
                        Color::Rgb(76, 175, 80)
                    } else if trend.0 == "Falling" {
                        Color::Rgb(244, 67, 54)
                    } else {
                        Color::Gray
                    }),
                ),
            ]));
        }

        // UV Index
        if let Some(uv) = current.uv_index {
            lines.push(Line::from(vec![
                Span::styled("UV Index:", Style::default().fg(Color::Gray)),
                Span::raw("  "),
                Span::styled(
                    format!("{:.1}", uv),
                    Style::default().fg(uv_color(uv)),
                ),
            ]));
        }

        // Visibility
        if let Some(vis) = current.visibility {
            let vis_display = if vis >= 1000.0 {
                format!("{:.1} km", vis / 1000.0)
            } else {
                format!("{:.0} m", vis)
            };
            lines.push(Line::from(vec![
                Span::styled("Visibility:", Style::default().fg(Color::Gray)),
                Span::raw("  "),
                Span::styled(vis_display, Style::default().fg(visibility_color(vis))),
            ]));
        }

        // Dew Point
        if let Some(dew) = current.dewpoint {
            lines.push(Line::from(vec![
                Span::styled("Dew Point:", Style::default().fg(Color::Gray)),
                Span::raw("  "),
                Span::styled(
                    self.app.format_temp(dew),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
        }

        Paragraph::new(lines).render(content_area, buf);
    }
}
