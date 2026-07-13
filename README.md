# Weatherman TUI

A fast, real-time terminal weather application built with **Rust**, [**ratatui**](https://ratatui.rs), and the **Open-Meteo** free API. Browse forecasts, switch units, and get at-a-glance conditions — all without leaving your terminal.

## Features

- 🌤️ **Real-time weather** — live current conditions from Open-Meteo
- 📅 **Hourly & daily forecasts** — 24+ hours ahead with icons and colors
- 🌡️ **Toggle units** — switch between Celsius/Fahrenheit and km/h/mph instantly
- 🔍 **City search** — fuzzy geocoding with keyboard navigation
- ⌨️ **Keyboard-driven** — no mouse needed; full arrow / vim-style support
- 🎨 **Color-coded weather** — icons and terminal colors adapt to conditions
- ⚡ **Fast startup** — compiled to a tiny native binary; instant renders
- 🪟 **Modal search overlay** — clean search UI right inside the TUI
- 💾 **Persistent location** — remembers your last-used city across sessions

## Screenshots (description)

The terminal displays **three regions** in a compact layout:

1. **Top bar** — weather icon (adaptive ☀️/🌙/⛅/🌧️/snowflake/thunderbolt), city name, current temperature in yellow, and apparent temperature in cyan.
2. **Left panel** — detailed current conditions (temperature, apparent temp, humidity, wind, precipitation, location, timestamp).
3. **Right panel** — tabbed content with two sub-tabs: **Hourly** (next 24 hours) and **Daily** (multi-day forecast). Each row shows day/date, icon, high/low temperatures color-coded (red/green), and precip.
4. **Bottom bar** — status line: last update time (local), active unit, active tab, and key shortcuts. A modal overlay appears for location search.

## Installation

### Cargo (recommended)

```bash
cargo install --path .
```

Or from source:

```bash
git clone https://github.com/zjacreman/weatherman.git
cd weatherman
cargo install --path .
```

### Run directly

```bash
cargo run
```

## Usage

1. **Start** the application — your previously saved location (if any) will be auto-loaded.
2. Press **`S`** — opens the location search modal.
3. Press **Enter** to search for the typed location.
4. Use **↑/↓** (or `k`/`j`) to navigate results; press **Enter** to select.
5. Press **Esc** to close the modal, or **Ctrl+U** to clear the search field.
6. Switch tabs, units, and refresh with the shortcuts below.

### Keyboard Shortcuts

| Key | Action |
|-----|------|
| `Tab` | Toggle between Daily and Hourly tabs |
| `S` | Open search modal |
| `Enter` | Submit search / select location |
| `Esc` | Close search modal and clear search |
| `Ctrl+U` | Clear search field |
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `Esc` | Close search / clear error |
| `U` | Toggle temperature unit (C ↔ F) |
| `R` | Refresh weather data |
| `Ctrl+C` | Quit the application |
| `q` | Quit the application |

## How It Works

### Open-Meteo API

The app consumes the [Open-Meteo API](https://open-meteo.com/) — a free, no-auth, open weather service.

- **Geocoding** — `GET https://geocoding-api.open-meteo.com/v1/search?name=<query>&count=10&language=en`
- **Forecast** — `GET https://api.open-meteo.com/v1/forecast?latitude=<lat>&longitude=<lon>&current=…&hourly=…&daily=…&timezone=auto&temperature_unit=celsius`

All response fields are self-describing in the JSON (see the API docs for full field lists).

- **Note**: `is_day` is returned as an `f64` (0.0 or 1.0), not a boolean
- **Retry logic**: Failed API calls are automatically retried up to 3 times with exponential backoff (500ms, 1000ms, 1500ms delays).

### Data Flow

```
Key press → Message enum → App::update() → internal state change
                                           ↓
                              async task dispatches (tokio)
                                           ↓
                              API fetch (reqwest) → deserialized models
                                           ↓
                              Message::WeatherFetchedBoxed / SearchResultsReceived
                                           ↓
                              Terminal draw cycle (ratatui)
```

1. The `App` struct holds all state. It uses a simplified **Elm Architecture** (`update` / `view`).
2. When the user triggers a search or refresh, a spawned async task calls the Open-Meteo API and reports back over an `mpsc` channel. The UI stays responsive during slow network calls; `q`/`Ctrl+C` always work.
3. Responses are deserialized with `serde` into domain models (`Location`, `CurrentWeather`, `HourlyForecast`, `DailyForecast`).
4. The main loop receives the result via a `Message` and mutates state.
5. `terminal.draw()` calls a pure function that renders the current state using ratatui widgets.

### WMO Weather Code Mapping

| Code | Condition | Icon (day) | Icon (night) | Color |
|------|-----------|------------|--------------|-------|
| 0 | Clear Sky | ☀️ | 🌙 | Yellow |
| 1 | Mainly Clear | 🌤️ (day) / 🌙 (night) | — | Light Yellow |
| 2 | Partly Cloudy | ⛅ | — | Light Blue |
| 3 | Overcast | ☁️ | — | Light Blue |
| 45, 48 | Fog | 🌫️ | — | Gray |
| 51-57 | Drizzle | 🌦️ | — | Blue |
| 61-67 | Rain | 🌧️ | — | Dark Blue |
| 71-77 | Snow | ❄️ | — | White |
| 80-82 | Rain Showers | 🌧️ | — | Blue |
| 85-87 | Snow Showers | ❄️ | — | White |
| 95-99 | Thunderstorm | ⛈️ | — | Pink |

## Architecture

### Module Structure

```
weatherman/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry-point + key handling + rendering
│   ├── lib.rs               # Library root — re-exports domain and API types
│   ├── app.rs               # Core domain: App, Message, AppState, TempUnit, WmoWeather, data models
│   ├── config.rs            # Persisted location loading/saving (test-safe dir override)
│   ├── api/
│   │   ├── mod.rs           # Module layout
│   │   ├── client.rs        # reqwest HTTP client, URL builders, query encoding
│   │   ├── geocoding.rs     # Open-Meteo geocoding search + GeocodingResult
│   │   └── weather.rs       # Open-Meteo forecast fetch + deserialization + WeatherResponse
│   └── ui/
│       ├── mod.rs           # Sub-module re-exports
│       ├── helpers.rs       # format_temp, format_wind_deg, progress_bar
│       └── widgets/         # Ratatui widget components
│           ├── mod.rs
│           ├── current.rs
│           └── error_modal.rs
├── tests/
│   └── unit_tests.rs       # Integration tests for WMO codes, models, conversion, API client
└── README.md
```

### Key Modules

- **`app.rs`** — The heart of the app. Contains the Elm-architecture `update` method, data models (`Location`, `CurrentWeather`, `HourlyForecast`, `DailyForecast`), the `WmoWeather` enum with icon/color/description, and the `TempUnit` enum.
- **`api/`** — Thin wrappers over Open-Meteo endpoints. Handles URL building, HTTP requests, and `serde` deserialization. `weather.rs` provides a single `WeatherResponse` type and a `TryFrom` impl for conversion to domain models. All API types have a single source of truth — no duplicate structs.
- **`config.rs`** — Configuration persistence with test-safe directory override via `load_config_from_dir()` and `save_config_to_dir()` to avoid the user's real config during tests.
- **`ui/`** — Ratatui widget components and layout helpers.

### Temperature & Wind Speed Conversion

- **Temperature**: `°F = °C × 9/5 + 32` — toggled globally by the user.
- **Wind Speed**: `mph = km/h ÷ 1.60934` — switches based on the active unit.

## Configuration

| Config | Default | Description |
|--------|---------|------ |
| Temperature unit | Celsius | Toggle with `U` |
| Refresh interval | 7200s (2h) | Configurable in `weatherman.toml` as `refresh_interval` |
| Tick rate | 1000ms | Terminal redraw rate |
| Search count | 10 | Results per geocoding query |
| Saved location | `<config dir>/weatherman/weatherman.toml` | Last-used city name (checks config dir then cwd). Config dir is platform-aware via the `dirs` crate. |

## Location Persistence

Weatherman remembers your last-used location across sessions using a TOML config file:

1. **Read order**: `<config dir>/weatherman/weatherman.toml` (platform config dir via the `dirs` crate) → `./weatherman.toml` (cwd)
2. **Write target**: The path that was read at startup; defaults to config dir if neither existed
3. **Format**:

```toml
name = "New York"
refresh_interval = 7200
```

- `name` (required): The last-used city name for auto-loading on startup.
- `refresh_interval` (optional): Auto-refresh interval in seconds. Default is 7200 (2 hours) if omitted.

On startup, the app searches for the saved location name and auto-fetches weather. On exit, the current location and refresh interval are written back to the same file. The status bar shows a countdown to the next auto-refresh.

## Development

```bash
# Run in dev mode
cargo run

# Build for release
cargo build --release

# Run tests
cargo test

# Type-check
cargo check

# Run clippy
cargo clippy
```

## Bug Fixes

#### v0.1.7 — Location Persistence
- **Added: Persistent location across sessions** — App now saves the current location to `<config dir>/weatherman/weatherman.toml` (platform config dir via the `dirs` crate; or `./weatherman.toml`) on exit and auto-loads it on startup. Writes to the same path that was read at startup; defaults to config dir if neither existed.
- **Added: Config module** (`src/config.rs`) with `load_config()` and `save_config()` functions using the `toml` crate. Cross-platform config resolution via the `dirs` crate (Windows `%APPDATA%`, macOS `~/Library/Application Support`, Linux `$XDG_CONFIG_HOME`).
- **Added: Auto-load weather on startup** — If a saved location exists, the app spawns a geocoding search task and fetches weather automatically without blocking the UI.

#### v0.2.0 — Sprint 2: Architecture Cleanup & Test Coverage
- **Arch: Unified deserialization types** — Removed duplicate `weather_api` module from `lib.rs`. `WeatherResponse`, `CurrentData`, `HourlyData`, `DailyData`, and `GeocodingResult` are now defined once in the `api/` crate and re-exported. The `GeocodingResult::from()` impl lives alongside the struct in `api/geocoding.rs`. Tests use `api/` types via crate root re-exports.
- **Arch: Removed dead `search_focused` field** — The `search_focused` field was written in 4 places but never read. Removed the field from the `App` struct, its initialization, and all assignments.
- **Fix: Startup geocoding failure now surfaces an error** — Previously a failed geocoding search on startup silently produced a blank screen. Now the error modal displays "Location 'X' not found" or "Failed to search for location: X" to inform the user.
- **Test: Safe config directory tests** — Fixed config persistence tests that used `std::env::set_var("HOME", ...)` which could overwrite the user's real config. All config tests now use temp directories with `load_config_from_dir()` / `save_config_to_dir()`.
- **Test: New `api/client.rs` tests** — 6 tests for `encode_query()`, `geocoding_url()`, and `forecast_url()` covering ASCII passthrough, space encoding, unicode encoding, and full URL format verification.
- **Test: New `App::update` state machine tests** — tests for `SearchClear`, `SearchInput`, `WeatherFetchedBoxed`, `SearchError`, `WeatherError`, `AutoSearchResult`, `ToggleUnit`, and `show_error()` transitions.
- **UX: Improved search modal instructions** — No-results state shows "Type a location name, then press Enter to search" and "Ctrl+U=clear · Esc=close". With-results state shows "Enter=select · Up/down=nav · Esc=close" for clarity.

#### v0.1.5 — Tab System Simplification
- **Fix: Simplified tab system** — Removed the broken 3-tab system (which caused phantom 'Index' state when pressing 3). Now uses exactly 2 tabs: **Daily** (default, shown on launch) and **Hourly** (toggle via `Tab` key). Removed number key shortcuts `1/2/3` and `h`/`l` — only `Tab` cycles tabs.

#### v0.1.6 — Rendering Bug Fixes
- **Fix: Crash on small terminal resize** — `area.height as usize - 2` caused "attempt to subtract with overflow" panic when terminal shrank below 2 lines. Fixed by using `saturating_sub(2)` in both `render_daily_tab` and `render_hourly_tab`, plus early-return guards for tiny areas.
- **Fix: Search modal text overlay** — Modal overlay text was semi-transparent, causing background content to show through the modal. Added a solid DarkGray background fill to cover the entire modal area, preventing content behind from showing through.
- **Fix: Status bar help text** — Removed gibberish `"Tab=Tab/z"` from status bar; replaced with clear help text `"Tab=cycles"`.
- **Fix: Double degree symbols in daily forecast** — `format_temp()` already included the `°` and `+` prefix, causing `++33°°` output. Removed redundant wrapper symbols from the format string.
- **Fix: Search modal background not fully opaque** — Initial modal fill was rendered before the block, allowing content to show through gaps. Added a final background cover paragraph after all modal content to guarantee full opacity.
- **Fix: Daily tab header missing opening paren** — `block_title` contained the full "daily (°C)" including the tab name, which was concatenated with `tab_label()` producing "│ Daily daily (°C)". Simplified to just the unit suffix " (°C)" so the title renders correctly as "│ Daily (°C)".

#### v0.1.4 — UI Polish & Bug Fixes
- **Fix: Tab bar panic when switching to Daily tab** — The tab bar calculation `(tab_idx - 1 + 3) % 3` underflowed when `tab_idx` was 0, causing an "attempt to subtract with overflow" panic. Fixed by using a conditional branch for the Index tab case.
- **Fix: Double degree symbol in top bar** — `format_temp()` already appends °, so wrapping it in another `"{}°"` produced `"+22°°"`. Removed the redundant degree symbol from `format!()` calls.
- **Fix: Tab cycling via Tab** — Added the Tab keyboard shortcut for cycling between Daily and Hourly tabs.
- **UX: Unit symbol now shows ° prefix** — Tab panel headers now display `°C`/`°F` instead of `C`/`F` for clarity.
- **UX: Status bar cleaned up** — Removed redundant "Ctrl+C=quit" from status text (already shown as "Q=quit").

#### v0.1.3 — Critical Fixes
- **Fixed: Weather data displays "error decoding response body"** — `is_day` field in Open-Meteo API is returned as `f64` (0.0/1.0), not as a boolean. Changed deserialization struct from `bool` to `f64` in both `api/weather.rs` and `lib.rs`.
- **Fixed: Error messages lost in status bar** — Added a prominent red error modal overlay for displaying errors. Errors are now also logged to stderr for post-mortem review.
- **Added: Integration tests for real API response format** — Tests now use `is_day: 1.0` (f64) instead of `is_day: true` (boolean) to verify real API deserialization. Added tests for all untested WMO weather codes and geocoding response envelope with correct field renaming.

#### v0.1.2 — Search Modal Fix
- **Fixed: Search typing triggered API calls on every character press** — Search now waits for Enter key before firing. Users type freely (with Ctrl+U to clear), press Enter to search, navigate results with Up/Down, select with Enter, and close with Esc. Search modal stays open to display results and allow location selection.

#### v0.1.1 — Critical Fixes
- **Fixed: Quit doesn't clean up terminal state** — Removed `std::process::exit()` from `run_app()`, allowing the cleanup block (`disable_raw_mode`, `LeaveAlternateScreen`, `DisableMouseCapture`) to run properly on quit. Plain `q` now quits cleanly. Status bar shows "Q=quit Ctrl+C=quit".
- **Fixed: Search never completes** — Both geocoding search and weather fetch fired on every event loop tick because neither had a guard to prevent re-dispatch while already in flight. Added `search_pending` and `weather_pending` one-shot flags to each fire exactly once per session.

#### v0.1.0 — Initial Release
- Initial release with Open-Meteo API integration

---

## Tests

Contributions welcome! Please:

1. Fork the repo and create a feature branch.
2. Add tests for new logic (see `tests/unit_tests.rs`).
3. Ensure `cargo test && cargo clippy` pass.
4. Open a pull request with a descriptive commit message.

## License

MIT
