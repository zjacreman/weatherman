use crate::app::{App, WmoWeather};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

pub struct CurrentWidget<'a> {
    pub app: &'a App,
}

impl Widget for CurrentWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Current Conditions ")
            .title_alignment(Alignment::Center);
        block.render(area, buf);

        let content_area = area.inner(Margin::new(1, 1));

        if self.app.location.is_none() || self.app.current.is_none() {
            Paragraph::new("No location selected")
                .alignment(Alignment::Center)
                .render(content_area, buf);
            return;
        }

        let current = self.app.current.as_ref().unwrap();
        let location = self.app.location.as_ref().unwrap();
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

        let mut lines: Vec<Line> = vec![
            // Location name (prominent)
            Line::from(vec![Span::styled(
                location.display_name().to_string(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
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
        ];

        // Conditionally add precipitation line
        if let Some(line) = precip_line {
            lines.push(line);
        }

        Paragraph::new(lines).render(content_area, buf);
    }
}
