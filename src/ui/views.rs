use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};
use ratatui::Frame;
use chrono::{Datelike, NaiveDate};
use chrono_tz::Tz;
use chrono::Timelike;

use crate::app::{App, AppState, WmoWeather};
use crate::ui;

pub fn draw(area: Rect, app: &App, frame: &mut Frame) {
    let main_chunks = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ],
    )
    .split(area);

    let (main_chunk, status_chunk) = (main_chunks[1], main_chunks[2]);

    // ── TOP BAR ──
    let title_line_vec: Vec<Span> =
        if let (Some(ref loc), Some(ref cur)) = (&app.location, &app.current) {
            let wmo = WmoWeather::from(cur.weather_code);
            vec![
                Span::styled(
                    format!(" {} ", wmo.icon(cur.is_day)),
                    Style::default().fg(wmo.color()),
                ),
                Span::raw("  "),
                Span::styled(
                    loc.display_name(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
            ]
        } else {
            vec![Span::raw(" Weather TUI — Press S to search ")]
        };
    let title_text = Line::from(title_line_vec);
    let title_block = Block::bordered().title_alignment(Alignment::Center);
    frame.render_widget(title_block, main_chunks[0]);
    let inner = main_chunks[0].inner(Margin::new(1, 0));
    frame.render_widget(
        Paragraph::new(title_text).alignment(Alignment::Center),
        inner,
    );

    // ── STATUS BAR ──
    let status_text = if app.state == AppState::LoadingSearch {
        " Searching... ".into()
    } else if app.state == AppState::LoadingWeather || app.state == AppState::Refreshing {
        " Loading weather... ".into()
    } else if app.search_modal_active {
        format!(" Search: {} ", app.search_query)
    } else if app.last_update.is_some() {
        let last_time = app.last_update.clone().unwrap_or_default();
        let remaining_secs = if app.tick_count < app.auto_refresh_interval.as_secs() {
            app.auto_refresh_interval.as_secs() - app.tick_count
        } else {
            0
        };
        let remaining_mins = remaining_secs / 60;
        let remaining_s = remaining_secs % 60;
        let refresh_display = if remaining_secs > 0 {
            format!("{:02}:{:02}", remaining_mins, remaining_s)
        } else {
            "REF".to_string()
        };
        let tab_display = match app.active_tab {
            0 => "daily",
            1 => "hourly",
            _ => "?",
        };
        format!(
            " Last: {} | {} | Ref: {} | Tab=cycles | S=search U=unit R=refresh Esc=clear Q=quit ",
            last_time, tab_display, refresh_display
        )
    } else {
        " Press S to search for a location ".into()
    };
    let status_block = Block::bordered().title_alignment(Alignment::Center);
    frame.render_widget(status_block, status_chunk);
    let inner_status = status_chunk.inner(Margin::new(1, 0));
    frame.render_widget(
        Paragraph::new(status_text).alignment(Alignment::Center),
        inner_status,
    );

    // ── MAIN CONTENT ──
    let active_chunks = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(40), Constraint::Percentage(60)],
    )
    .split(main_chunk);

    render_current_details(app, active_chunks[0], frame);
    render_tabbed_content(app, active_chunks[1], frame);

    // ── SEARCH MODAL ──
    if app.search_modal_active {
        render_search_modal(app, frame.area(), frame);
    }

    // ── ERROR MODAL ──
    if app.error_modal_visible {
        let modal = ui::widgets::error_modal::ErrorModal {
            message: app.error_message.as_deref().unwrap_or(""),
        };
        frame.render_widget(&modal, frame.area());
    }
}

fn render_current_details(app: &App, area: Rect, frame: &mut Frame) {
    use ratatui::widgets::Widget;
    let widget = ui::CurrentWidget { app };
    widget.render(area, frame.buffer_mut());
}

fn render_tabbed_content(app: &App, area: Rect, frame: &mut Frame) {
    let tabs = ["daily", "hourly"];
    let tab_idx = app.active_tab as usize;
    let block_title = format!(" (°{})", app.temperature_unit.symbol());

    let block = Block::bordered().title(format!("{}{}", tab_label(tab_idx), block_title));
    frame.render_widget(block, area);
    let inner_area = area.inner(Margin::new(1, 0));

    let tab_bar_y = inner_area.top();
    let tab_texts: Vec<Span> = tabs
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let active = tab_idx == i;
            if active {
                Span::styled(
                    format!(" {} ", name),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(format!(" {} ", name), Style::default().fg(Color::Gray))
            }
        })
        .collect();

    let tab_line = Line::from(tab_texts);
    frame.render_widget(
        Paragraph::new(tab_line),
        Rect::new(inner_area.left(), tab_bar_y, inner_area.width, 1),
    );

    let content_area = Rect::new(
        inner_area.left(),
        inner_area.top() + 1,
        inner_area.width,
        inner_area.height.saturating_sub(1),
    )
    .inner(Margin::new(3, 0));

    match tab_idx {
        0 => render_daily_tab(app, content_area, frame),
        1 => render_hourly_tab(app, content_area, frame),
        _ => {}
    }
}

fn tab_label(idx: usize) -> &'static str {
    match idx {
        0 => "│ Daily",
        1 => "│ Hourly",
        _ => "│",
    }
}

fn render_hourly_tab(app: &App, area: Rect, frame: &mut Frame) {
    let lines: Vec<Line> = if let Some(ref hourly) = app.hourly {
        // Find the starting index: first future or current hour in the location's timezone
        let start_idx = if let Some(ref loc) = app.location {
            let tz: Tz = loc.timezone.parse().unwrap_or(Tz::UTC);
            let now = chrono::Utc::now().with_timezone(&tz);
            let now_date = now.format("%Y-%m-%d").to_string();
            let now_hour = now.hour();

            hourly
                .times
                .iter()
                .enumerate()
                .find_map(|(i, t)| {
                    if t.len() >= 13 {
                        let date_part = &t[..10];
                        let hour_part = t[11..13].parse::<u32>().ok()?;
                        if date_part > now_date.as_str()
                            || (date_part == now_date.as_str() && hour_part >= now_hour)
                        {
                            Some(i)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or(0)
        } else {
            0
        };

        // Limit to available space (leave 1 line of padding at bottom)
        let max_data_rows = area.height.saturating_sub(3) as usize;
        let available = hourly.times.len().saturating_sub(start_idx);
        let count = std::cmp::min(max_data_rows, available);

        let mut out: Vec<Line> = vec![Line::from(format!("Next {} hours", count))];
        for i in 0..count {
            let idx = start_idx + i;
            let time = &hourly.times[idx];
            let hour_str = if time.len() > 13 {
                time[11..13].to_string()
            } else {
                time.clone()
            };
            let temp = app.format_temp(hourly.temperatures[idx]);
            let wmo = WmoWeather::from(hourly.weather_codes[idx]);
            let is_day = hourly.is_day[idx];
            let prec = format!("{:.1} mm", hourly.precipitations[idx]);
            out.push(Line::from(vec![
                Span::styled(hour_str, Style::default().fg(Color::Yellow)),
                Span::raw(" | "),
                Span::styled(
                    format!("{} {}", wmo.icon(is_day), temp),
                    Style::default().fg(wmo.color()),
                ),
                Span::raw(" | "),
                Span::styled(format!("💧 {prec}"), Style::default().fg(Color::Cyan)),
            ]));
        }
        out
    } else if area.height > 1 && area.width > 0 {
        vec![Line::from(""), Line::from("No hourly data available.")]
    } else {
        Vec::new()
    };

    if lines.is_empty() {
        return;
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_daily_tab(app: &App, area: Rect, frame: &mut Frame) {
    let lines: Vec<Line> = if let Some(ref daily) = app.daily {
        let count = std::cmp::min(daily.dates.len(), area.height.saturating_sub(2) as usize);

        let mut out: Vec<Line> = vec![
            Line::from(format!("{count} day forecast")),
            Line::from(""),
        ];

        for i in 0..count {
            let day_name = day_of_week(&daily.dates[i]);
            let high = daily.temp_high[i];
            let low = daily.temp_low[i];
            let wmo = WmoWeather::from(daily.weather_codes[i]);
            let precip = format!("{:.1} mm", daily.precip_sum[i]);
            let wind = app.format_wind_speed(daily.wind_max[i]);

            out.push(Line::from(vec![
                Span::styled(day_name, Style::default().fg(Color::Yellow)),
                Span::raw(" | "),
                Span::styled(app.format_temp(high), Style::default().fg(Color::Red)),
                Span::raw(" | "),
                Span::styled(app.format_temp(low), Style::default().fg(Color::Blue)),
                Span::raw(" | "),
                Span::styled(
                    format!("{} {}", wmo.icon(true), wmo.description()),
                    Style::default().fg(wmo.color()),
                ),
                Span::raw(" | "),
                Span::styled(format!("💧 {precip}"), Style::default().fg(Color::Cyan)),
                Span::raw(" | "),
                Span::styled(format!("🌬️ {wind}"), Style::default().fg(Color::White)),
            ]));
        }
        out
    } else if area.height > 1 && area.width > 0 {
        vec![Line::from(""), Line::from("No daily data available.")]
    } else {
        Vec::new()
    };

    if lines.is_empty() {
        return;
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_search_modal(app: &App, area: Rect, frame: &mut Frame) {
    let modal_width = 60u16;
    let num_results = app.search_results.len();
    let content_height = if num_results > 0 {
        // input (1) + results (each 2 lines: name + sub) + separator (1) + instructions (1)
        num_results.min(10) * 2 + 3
    } else {
        // input (1) + "No results" (1) + separator (1) + instructions (1)
        4
    };
    let modal_height: u16 = (content_height + 2).try_into().unwrap_or(20);
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    // Clear underlying content and paint an opaque DarkGray background.
    // `Clear` zeroes the cells; `set_style` then applies the modal background
    // across the whole area in one buffer pass — replacing a per-cell loop.
    frame.render_widget(Clear, modal_area);
    let bg_style = Style::default().bg(Color::DarkGray);
    for y in modal_area.top()..modal_area.bottom() {
        for x in modal_area.left()..modal_area.right() {
            frame.buffer_mut()[(x, y)].set_style(bg_style);
        }
    }

    // Block border + title
    let block = Block::bordered()
        .title(" Search locations ")
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(block, modal_area);

    let inner = modal_area.inner(Margin::new(1, 2));

    // Build all content for the inner area as styled lines
    let mut content_lines: Vec<Line> = Vec::new();
    let bg = Style::default().bg(Color::DarkGray);

    // Input row
    content_lines.push(Line::from(Span::styled(
        format!("> {}", app.search_query),
        Style::default().fg(Color::White).bg(Color::DarkGray),
    )));

    if !app.search_results.is_empty() {
        let visible = std::cmp::min(
            (inner.height.saturating_sub(3) as usize) / 2,
            app.search_results.len(),
        );
        for i in 0..visible {
            let loc = &app.search_results[i];
            if i as u16 == app.search_selected_idx {
                content_lines.push(Line::from(Span::styled(
                    format!(" {}", loc.name),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            } else {
                content_lines.push(Line::from(Span::styled(
                    loc.name.clone(),
                    Style::default().fg(Color::White).bg(Color::DarkGray),
                )));
            }
            let sub = if let Some(ref admin) = loc.admin1 {
                format!(" {} · {}", admin, loc.country)
            } else {
                format!(" {}", loc.country)
            };
            content_lines.push(Line::from(Span::styled(
                sub,
                Style::default().fg(Color::Gray).bg(Color::DarkGray),
            )));
        }
        content_lines.push(Line::from(Span::styled(" ", bg)));
        content_lines.push(Line::from(Span::styled(
            " Enter=select · Up/down=nav · Esc=close ",
            Style::default().fg(Color::Gray).bg(Color::DarkGray),
        )));
    } else {
        content_lines.push(Line::from(Span::styled(
            " No results ",
            Style::default().fg(Color::Gray).bg(Color::DarkGray),
        )));
        content_lines.push(Line::from(Span::styled(" ", bg)));
        content_lines.push(Line::from(Span::styled(
            " Type a location name, then press Enter to search ",
            Style::default().fg(Color::Gray).bg(Color::DarkGray),
        )));
        content_lines.push(Line::from(Span::styled(
            " Ctrl+U=clear · Esc=close ",
            Style::default().fg(Color::Gray).bg(Color::DarkGray),
        )));
    }

    // Render all content to the inner area as ONE Paragraph — guarantees zero gaps
    let full_text = content_lines;
    let content_para = Paragraph::new(full_text).style(Style::default().bg(Color::DarkGray));
    frame.render_widget(content_para, inner);
}

fn day_of_week(date_str: &str) -> String {
    if date_str.len() < 10 {
        return date_str.to_string();
    }
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() < 3 {
        return date_str.to_string();
    }
    if let (Ok(y), Ok(m), Ok(d)) = (
        parts[0].parse::<i32>(),
        parts[1].parse::<u32>(),
        parts[2].parse::<u32>(),
    ) {
        if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
            date.weekday().to_string()
        } else {
            date_str.to_string()
        }
    } else {
        date_str.to_string()
    }
}
