# AGENTS.md — AI Agent Development Guide

> File for AI agents working on this project. Keep this file in sync with the codebase.

## Key Commands

```bash
# Run the application
cargo run

# Build release binary
cargo build --release

# Run all tests
cargo test

# Type-check only (fastest)
cargo check

# Lint with clippy
cargo clippy -- -D warnings
```

## Architecture Summary

- **Language**: Rust 2021 Edition
- **TUI Framework**: [ratatui 0.30](https://ratatui.rs) — the maintained fork of tui-rs
- **Terminal I/O**: [crossterm 0.29](https://github.com/crossterm-rs/crossterm)
- **Async Runtime**: [tokio 1](https://tokio.rs) (full features)
- **HTTP Client**: [reqwest 0.12](https://docs.rs/reqwest)
- **Serialization**: [serde 1](https://serde.rs) + [serde_json 1](https://docs.rs/serde_json)
- **Date/Time**: [chrono 0.4](https://docs.rs/chrono) with serde feature
- **Config**: [toml 0.8](https://docs.rs/toml) for persisted location config

### Elm Architecture

The app follows a simplified Elm Architecture:

1. **Model** — `App` struct in `app.rs` holds all state (location, weather data, UI state).
2. **Messages** — `Message` enum represents every state transition trigger (key press, API result, tick).
3. **Update** — `App::update(&mut self, msg: Message)` is the pure-ish mutation function.
4. **View** — `draw()` in `main.rs` renders the current `App` state to the terminal.

The event loop in `main.rs` combines keyboard input, async API tasks, and the tick-based auto-refresh.

### Data Flow

```
KeyEvent / API Result / Tick
    →
Message enum variant
    →
App::update(msg)
    →
State mutation (last_update in local HH:MM:SS, search_results, error_message, etc.)
    →
terminal.draw(|frame| draw(frame.area(), &app, frame))
    →
ratatui rendering via widgets
```

## File Structure

```
weatherman/
├── Cargo.toml
├── rust-toolchain.toml
├── src/
│   ├── main.rs              # Binary entry, event loop, key handling, drawing
│   ├── lib.rs                # Library root — re-exports domain and API types
│   ├── app.rs               # Core domain: App, Message, WmoWeather, models
│   ├── config.rs            # Persisted location loading/saving (test-safe dir override)
│   ├── api/
│   │   ├── mod.rs           # Module re-exports
│   │   ├── client.rs        # reqwest HTTP client + URL builders + query encoding
│   │   ├── geocoding.rs     # Open-Meteo geocoding search
│   │   └── weather.rs       # Open-Meteo forecast fetch + deserialization
│   └── ui/
│       ├── mod.rs           # Sub-module re-exports
│       ├── helpers.rs       # format_temp, format_wind_deg, progress_bar
│       └── widgets/         # Ratatui widget components
│           ├── mod.rs
│           ├── current.rs
│           └── error_modal.rs
├── tests/
│   └── unit_tests.rs        # Integration tests for domain logic
└── README.md
```

## API Details

### Geocoding Endpoint

```
GET https://geocoding-api.open-meteo.com/v1/search?name=<query>&count=<n>&language=en
```

- `name`: city/town name (URL-encoded)
- `count`: number of results (1-10)
- `language`: English results

**Response (GeocodingResult):**
```rust
struct GeocodingResult {
    id: u64,
    name: String,
    admin1: Option<String>,     // State/province
    country: String,
    country_code: String,       // ISO 3166-1 alpha-2
    latitude: f64,
    longitude: f64,
    elevation: Option<f64>,
    timezone: String,
    population: Option<u64>,
}
```

### Forecast Endpoint

```
GET https://api.open-meteo.com/v1/forecast?latitude=<lat>&longitude=<lon>&current=<fields>&hourly=<fields>&daily=<fields>&timezone=auto&temperature_unit=celsius
```

**Current fields**: `temperature_2m`, `relative_humidity_2m`, `apparent_temperature`, `precipitation`, `weather_code`, `wind_speed_10m`, `wind_direction_10m`, `wind_gusts_10m`, `is_day`

**Hourly fields**: `temperature_2m`, `relative_humidity_2m`, `apparent_temperature`, `weather_code`, `precipitation`, `wind_speed_10m`, `is_day`

**Daily fields**: `weather_code`, `temperature_2m_max`, `temperature_2m_min`, `sunrise`, `sunset`, `precipitation_sum`, `wind_speed_10m_max`

### Config Persistence

Location persistence uses `src/config.rs` with the `toml` crate:

```rust
pub struct SavedConfig {
    pub name: String,
    pub refresh_interval: Option<u64>,  // seconds; defaults to 7200 (2h) if omitted
}
```

- **Read order**: `~/.config/weatherman/weatherman.toml` (config dir) → `./weatherman.toml` (cwd)
- **Write target**: The path read at startup; defaults to config dir if neither existed
- **Format**:
```toml
name = "New York"
refresh_interval = 7200
```

On startup, `App::new()` loads the saved config and sets `pending_auto_search`. The event loop in `main.rs` performs a geocoding search and auto-fetches weather before the event loop begins. On exit, `run_app()` writes the current location and refresh interval back to the same config file.

### WMO Weather Code Mapping (from `app.rs`)

| Code(s)      | Condition               | Icon                          |
|------------- | ----------------------- | ----------------------------- |
| 0            | Clear Sky              | ☀️ (day) / 🌙 (night)         |
| is_day (current) | f64 (0.0 or 1.0) | N/A | Not a weather code        |
| 1            | Mainly Clear           | —                             |
| 2            | Partly Cloudy          | ⛅                            |
| 3            | Overcast               | ☁️                            |
| 45, 48       | Fog                    | 🌫️                            |
| 51, 53, 55   | Light/Moderate/Dense Drizzle | 🌦️                    |
| 56, 57       | Freezing Drizzle       | 🌦️                            |
| 58           | Dense Freezing Drizzle | 🌦️                            |
| 61, 63, 65   | Light/Moderate/Heavy Rain | 🌧️                         |
| 66, 67       | Freezing Rain          | 🌫️                            |
| 71, 73, 75   | Light/Moderate/Heavy Snow | ❄️                         |
| 77           | Snow Grains            | ❄️                            |
| 80, 81, 82   | Rain Showers           | 🌧️                            |
| 85, 86, 87   | Snow Showers           | ❄️                            |
| 95           | Thunderstorm           | ⛈️                            |
| 96, 99       | Thunderstorm w/ Hail   | ⛈️                            |

## Development Notes

### Ratatui 0.30 Patterns

- Use `Frame::render_widget()` for all widgets.
- Layout via `Layout::new(Direction, Constraints)`.
- Margins: `area.inner(Margin::new(top/bottom, left/right))`.
- Colors: `ratatui::style::Color::Rgb(r, g, b)` or named colors (`Color::Yellow`, `Color::Cyan`, etc.).
- Widgets use `Block::bordered()`, `Paragraph::new()`, `Span::styled()` with `Style`.

### Tick Timeout

The tick timeout uses `saturating_sub` to prevent unsigned integer underflow when a loop iteration takes longer than `TICK_RATE` (1000ms):

```rust
let timeout = std::cmp::min(
    TICK_RATE.saturating_sub(last_tick.elapsed().as_millis() as u64),
    TICK_RATE,
);
```

Without `saturating_sub`, elapsed times exceeding 1000ms (common during slow network requests like geocoding search) would cause a "subtract with overflow" panic in release mode and a debug assertion in debug mode.

---

### Async Event Loop

The main loop in `main.rs` interleaves:

1. **Terminal draw** — `terminal.draw(|frame| ...)` each tick.
2. **Crossterm event poll** — blocks up to 1000ms waiting for a key.
3. **Key handling** — `handle_key()` returns bool for handled/unhandled.
4. **State-based async dispatch** — based on `AppState` enum, launches API tasks via `async move {}`.

**To add a new async operation**: Create a new `AppState` variant, add a `Message` variant, and dispatch the task in the match arm.

### Keyboard Handling

- `handle_key()` in `main.rs` matches `KeyEvent` variants.
- Use `KeyCode::Char('c')` and `KeyModifiers::CONTROL` for modifier combos.

**Outside search modal:**
- `S`/`s` — Opens search modal, clears previous results
- `Tab` — Toggle between Daily and Hourly tabs
- `U`/`u` — Toggle temperature unit
- `R`/`r` — Refresh weather data
- `Esc` — Close modal or clear error
- `q` — Quit the app
- `Ctrl+C` — Quit the app

**Inside search modal:**
- Typing — Appends to `search_query` (no API calls)
- `Enter` — Submits search (if no results yet) or selects location (if results exist)
- `Up`/`k`/`Down`/`j` — Navigate results
- `Ctrl+U` — Clear search query
- `Esc` — Close modal and clear search

### Time Storage Note

The `last_update` field stores time as `HH:MM:SS` in the system's local timezone (using `chrono::Local`), not UTC. This avoids confusion in the status bar.

### Adding a New API Field

1. Add the field to the `#[derive(Deserialize)]` struct in `api/weather.rs` or `api/geocoding.rs`.
2. Update the domain model in `app.rs` (e.g., `CurrentWeather`, `HourlyForecast`) as needed.
3. If the field is added to a fetch struct (with `Option<>`), update `into_models()` in the same file.
4. Add the field to the corresponding `TestXXX` struct (non-Option variant) in `api/weather.rs` if exposed for tests.
5. Update the `From<TestXXX>` impl if converting between types.
6. Update `lib.rs` re-exports in the `weather_api` comment if a new type is exposed.
7. Pass through to the widget/display layer as needed.

### Status Bar / Footer

The bottom status bar displays the `last_update` time in `HH:MM:SS` format in the system's local timezone (no timezone suffix). Time is produced using `chrono::Local`.

**Important**: `is_day` from Open-Meteo is returned as `f64` (0.0 or 1.0), NOT as a boolean. You MUST change the deserialization struct field to `f64` and convert it to bool with `!= 0.0` in the From impl.

## Testing Approach

### Integration Tests

All tests are in `tests/unit_tests.rs`. They exercise:

- **WmoWeather**: `from()`, `description()`, `icon()`, `color()` for every variant.
- **App**: `convert_temp()`, `format_temp()`, `format_wind_direction()`, `progress_bar()`.
- **Geocoding deserialization**: Parse real Open-Meteo JSON responses via serde.
- **Weather deserialization**: Parse current/hourly/daily forecast responses.
- **Message**: Construct all variants, verify `Clone`.
- **TempUnit**: `toggle()` roundtrip, `symbol()` values.
- **App state machine**: `new()` defaults, `WeatherFetched`, `ToggleUnit`.
- **ui::helpers**: Standalone `progress_bar` and `format_wind_full` tests.
- **Search modal**: Modal open/close toggling, search state reset on clear.
- **Tick timeout overflow** — 6 tests verifying `saturating_sub` behavior prevents unsigned integer overflow at normal, zero, exact-equals, and extreme elapsed times.
- **Runtime Option-wrapped weather deserialization** — verifies `api/weather.rs` Option<T> structs parse real API JSON correctly.
- **Geocoding envelope** — verifies the `"results"` (plural) field in the outer geocoding response envelope.
- **WMO weather codes** — tests for all WMO code variants including main clear, rime fog, freezing rain, snow grains, rain/snow showers, freezing drizzle.
- **Tab system** — 2-tab toggle (Daily/Hourly), default tab verification
- **Config persistence** — tests for `load_config()` (config dir then cwd fallback) and `save_config()` (uses last path or defaults to config dir)
- **Config persistence (live)** — 3 tests for save/load roundtrip, parent directory creation, and None-on-missing-file
- **Config refresh_interval** - tests for `SavedConfig` serialization/deserialization with and without `refresh_interval`, and default value verification
- **api/client.rs** - 6 tests for `encode_query()` (ASCII passthrough, space encoding, unicode encoding), `geocoding_url()` (format + encoding), and `forecast_url()` (format verification with required params)
- **App::update** - 7 tests for `SearchClear`, `SearchInput`, `WeatherFetched`, `SearchError`, `WeatherError`, `ToggleUnit`, and `show_error()` state transitions
- **Config persistence (safe)** - tests use `load_config_from_dir()` / `save_config_to_dir()` with temp directories instead of `HOME` env var manipulation

### Adding New Tests

1. Pick the relevant section header in `tests/unit_tests.rs`.
2. Write a `#[test] fn name() { ... }` below it.
3. Run `cargo test` to verify.
4. Follow existing naming conventions: `module_descriptive_name()`.

### Important Test Patterns

- Use `serde_json::from_str::<Type>(json_str)` for API deserialization tests.
- Compare floats with epsilon: `(a - b).abs() < f32::EPSILON`.
- Use `matches!(value, Type::Variant)` for enum variant checks.
- The `weather::` prefix accesses re-exported types from `lib.rs`.

### New Test Patterns

- **`quit_key_sets_is_quit`** — verifies 'q' key sets quit flag
- **`ctrl_c_sets_is_quit`** — verifies Ctrl+C also sets quit flag
- **`search_modal_open_and_close()`** — verifies modal toggling
- **`clear_search_resets_all()`** — verifies search state is fully reset on clear
- **`deserialize_runtime_weather_response_with_options`** — verifies Option-wrapped deserialization from api/weather.rs
- **`deserialize_geocoding_envelope`** — verifies correct "results" (plural) field name
- **WMO weather code coverage** — tests for all untested WMO variants (codes 1, 48, 56, 66, 77, 80, 85)
- **Tab key cycling** — verifies Tab toggles between Daily and Hourly tabs
- **`daily_is_default_tab`** — verifies default `active_tab` is 0 (Daily)
- **Tick timeout overflow tests remain unchanged: `tick_timeout_saturating_sub_*` (6 tests)**
- **format_temp rendering** — 3 tests verifying format_temp produces exactly one ° symbol in positive (single +), negative (no +), and zero cases
- **saturating_sub pattern** — 1 test verifying the pattern prevents subtraction underflow at zero/one/two heights
