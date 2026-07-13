mod api;
mod app;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};
use ratatui::Terminal;
use tokio::sync::mpsc;

mod config;
use app::{App, AppState, Message, TempUnit, WmoWeather};
use chrono::Timelike;
use chrono_tz::Tz;

const TICK_RATE: u64 = 1000; // ms — each tick = 1 real second

/// Attempt an API task with retries. Returns the first successful result or the last error.
async fn with_retries<F, Fut, T, E>(max_attempts: u32, task_factory: F) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut last_err = None;
    for attempt in 1..=max_attempts {
        match task_factory().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                if attempt < max_attempts {
                    let delay = std::time::Duration::from_millis(500 * attempt as u64);
                    tokio::time::sleep(delay).await;
                }
                last_err = Some(e);
            }
        }
    }
    Err(last_err.expect("at least one attempt was made"))
}

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let result = run_app(&mut terminal).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

#[allow(clippy::too_many_lines)]
async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();

    // mpsc channel carrying every state transition (keys, ticks, async results).
    // Unbounded so senders in spawned tasks never have to await (they run to
    // completion even if the main loop is busy drawing).
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // ── Keyboard listener: crossterm is a blocking sync API, so it runs on a
    //    dedicated blocking thread and forwards KeyEvents as Messages.
    {
        let tx = tx.clone();
        tokio::task::spawn_blocking(move || loop {
            // poll for up to 100ms so the task can exit promptly if the channel closes
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(CrosstermEvent::Key(key)) = event::read() {
                    if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat)
                        && tx.send(Message::Key(key)).is_err()
                    {
                        return;
                    }
                }
            }
        });
    }

    // ── Tick timer: one Message::Tick per second drives auto-refresh + redraws.
    {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(TICK_RATE));
            loop {
                interval.tick().await;
                if tx.send(Message::Tick).is_err() {
                    return;
                }
            }
        });
    }

    // ── Startup auto-load: if a location was persisted, kick off the geocoding
    //    search as a spawned task so the UI never blocks. The result returns as
    //    Message::AutoSearchResult, which then triggers the weather fetch.
    if let Some(name) = app.pending_auto_search.take() {
        app.search_pending = true;
        let tx = tx.clone();
        let query = name.clone();
        tokio::spawn(async move {
            match with_retries(3, || async { api::geocoding::search(&query).await }).await {
                Ok(results) => {
                    let _ = tx.send(Message::AutoSearchResult { name, results });
                }
                Err(e) => {
                    let _ = tx.send(Message::SearchError(e.to_string()));
                }
            }
        });
    }

    // The forecast statuses contain variable-presentation emoji (e.g. 🌤️ =
    // U+1F324 U+FE0F) whose terminal display width is ambiguous. ratatui's
    // incremental diff emits them in a different print context after a partial
    // repaint, which desyncs column alignment (the "Mainly clear" → "Mainlyclear"
    // bug). Forcing a full clear+repaint whenever the visible content changes
    // makes every frame render like the (correct) initial frame. We detect those
    // changes with a cheap signature of the state that affects what is drawn:
    // active tab, unit, either modal's visibility, and the last-update stamp
    // (which changes on refresh / new location data).
    let mut displayed_view = view_signature(&app);

    loop {
        if view_signature(&app) != displayed_view {
            terminal.clear()?;
            displayed_view = view_signature(&app);
        }
        terminal.draw(|frame| {
            draw(frame.area(), &app, frame);
        })?;

        // Block on the next Message — keys, ticks, and async results all arrive
        // here. The UI stays responsive because no HTTP call lives on this task.
        let Some(msg) = rx.recv().await else { break };
        app.update(msg);

        if app.is_quitting() {
            break;
        }

        // After each state transition, launch any async work the new state implies.
        // In-flight guards prevent spawning duplicate tasks.
        match app.state {
            AppState::LoadingSearch if !app.search_pending => {
                if app.search_query.is_empty() {
                    app.state = AppState::Idle;
                } else {
                    app.search_pending = true;
                    let tx = tx.clone();
                    let query = app.search_query.clone();
                    tokio::spawn(async move {
                        match with_retries(3, || async { api::geocoding::search(&query).await }).await
                        {
                            Ok(results) => {
                                let _ = tx.send(Message::SearchResultsReceived(results));
                            }
                            Err(e) => {
                                let _ = tx.send(Message::SearchError(e.to_string()));
                            }
                        }
                    });
                }
            }
            AppState::LoadingWeather | AppState::Refreshing if !app.weather_pending => {
                if let Some(loc) = app.location.as_ref() {
                    app.weather_pending = true;
                    let tx = tx.clone();
                    let lat = loc.latitude;
                    let lon = loc.longitude;
                    tokio::spawn(async move {
                        match with_retries(3, || async { api::weather::fetch(lat, lon).await }).await
                        {
                            Ok((current, hourly, daily)) => {
                                let _ = tx.send(Message::WeatherFetchedBoxed(
                                    current,
                                    Box::new(hourly),
                                    Box::new(daily),
                                ));
                            }
                            Err(e) => {
                                let _ = tx.send(Message::WeatherError(e.to_string()));
                            }
                        }
                    });
                } else {
                    app.state = AppState::Idle;
                    app.weather_pending = false;
                }
            }
            _ => {}
        }
    }

    // Persist the current location on exit.
    if let Some(ref loc) = app.location {
        let cfg = config::SavedConfig {
            name: loc.name.clone(),
            refresh_interval: Some(app.auto_refresh_interval.as_secs()),
        };
        let base_dir = config::config_path();
        if let Err(e) = config::save_config(&cfg, app.last_config_path.as_deref(), &base_dir) {
            eprintln!("Failed to save config: {e}");
        }
    }

    Ok(())
}

/// Cheap fingerprint of the state that determines what is painted, used to
/// decide when a full repaint is needed instead of ratatui's incremental diff
/// (see the note in the event loop about ambiguous-width emoji).
fn view_signature(app: &App) -> (u16, TempUnit, bool, bool, Option<String>) {
    (
        app.active_tab,
        app.temperature_unit,
        app.search_modal_active,
        app.error_modal_visible,
        app.last_update.clone(),
    )
}

fn draw(area: Rect, app: &App, frame: &mut ratatui::Frame) {
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

fn render_current_details(app: &App, area: Rect, frame: &mut ratatui::Frame) {
    use ratatui::widgets::Widget;
    let widget = ui::CurrentWidget { app };
    widget.render(area, frame.buffer_mut());
}

fn render_tabbed_content(app: &App, area: Rect, frame: &mut ratatui::Frame) {
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

fn render_hourly_tab(app: &App, area: Rect, frame: &mut ratatui::Frame) {
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

fn render_daily_tab(app: &App, area: Rect, frame: &mut ratatui::Frame) {
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

fn render_search_modal(app: &App, area: Rect, frame: &mut ratatui::Frame) {
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
        use chrono::{Datelike, NaiveDate};
        if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
            date.weekday().to_string()
        } else {
            date_str.to_string()
        }
    } else {
        date_str.to_string()
    }
}
