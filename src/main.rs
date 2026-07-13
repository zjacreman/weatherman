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
use ratatui::Terminal;
use tokio::sync::mpsc;

mod config;
use app::{App, AppError, AppState, Message, TempUnit};

const TICK_RATE: u64 = 1000; // ms — each tick = 1 real second

/// Attempt an API task with retries. Returns the first successful result or the last error.
async fn with_retries<F, Fut, T>(max_attempts: u32, task_factory: F) -> Result<T, AppError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
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
            ui::draw(frame.area(), &app, frame);
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


