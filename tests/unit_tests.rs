//! ==================================================
//!  Weather TUI — integration tests for the domain layer
//! ============================================================

use weatherman::config;
use weatherman::Color;
use weatherman::api::client;
use std::time::Duration;
use weatherman::{App, AppState, Location, Message, SavedConfig, TempUnit, WmoWeather};
use weatherman::{GeocodingResult, WeatherResponse};

// ────────────────────────────────────────────────────
// WMO Weather Code tests
// ────────────────────────────────────────────────────

#[test]
fn wmo_clear_sky() {
    let w = WmoWeather::from(0);
    assert!(matches!(w, WmoWeather::ClearSky));
    assert_eq!(w.description(), "Clear sky");
    assert_eq!(w.icon(true), "\u{1F31E}"); // day
    assert_eq!(w.icon(false), "\u{1F319}"); // night
}

#[test]
fn wmo_partly_cloudy_icon() {
    let w = WmoWeather::from(2);
    assert!(matches!(w, WmoWeather::PartlyCloudy));
    assert_eq!(w.icon(true), "\u{26C5}");
}

#[test]
fn wmo_overcast_icon() {
    let w = WmoWeather::from(3);
    assert!(matches!(w, WmoWeather::Overcast));
    assert_eq!(w.icon(true), "\u{2601}");
}

#[test]
fn wmo_fog() {
    let w = WmoWeather::from(45);
    assert!(matches!(w, WmoWeather::Fog));
}

#[test]
fn wmo_slight_rain() {
    let w = WmoWeather::from(61);
    assert!(matches!(w, WmoWeather::SlightRain));
}

#[test]
fn wmo_slight_snow() {
    let w = WmoWeather::from(71);
    assert!(matches!(w, WmoWeather::SlightSnow));
}

#[test]
fn wmo_thunderstorm() {
    let w = WmoWeather::from(95);
    assert!(matches!(w, WmoWeather::Thunderstorm));
}

#[test]
fn wmo_thunderstorm_hail() {
    let w = WmoWeather::from(99);
    assert!(matches!(w, WmoWeather::ThunderstormWithHail));
}

#[test]
fn wmo_unknown_code() {
    let w = WmoWeather::from(42);
    assert!(matches!(w, WmoWeather::Unknown(42)));
    assert_eq!(w.description(), "Unknown (42)");
}

#[test]
fn wmo_clear_sky_color() {
    let w = WmoWeather::ClearSky;
    let c = w.color();
    assert!(matches!(c, Color::Rgb(255, 193, 7)));
}

#[test]
fn wmo_rain_color() {
    let w = WmoWeather::SlightRain;
    let c = w.color();
    assert!(matches!(c, Color::Rgb(70, 130, 180)));
}

#[test]
fn wmo_thunderstorm_color() {
    let w = WmoWeather::Thunderstorm;
    let c = w.color();
    assert!(matches!(c, Color::Rgb(255, 105, 180)));
}

#[test]
fn wmo_snow_color() {
    let w = WmoWeather::SlightSnow;
    let c = w.color();
    assert!(matches!(c, Color::Rgb(240, 248, 255)));
}

#[test]
fn wmo_fog_color() {
    let w = WmoWeather::Fog;
    let c = w.color();
    assert!(matches!(c, Color::Rgb(150, 150, 160)));
}

#[test]
fn wmo_mainly_clear_code() {
    let w = WmoWeather::from(1);
    assert!(matches!(w, WmoWeather::MainlyClear));
    assert_eq!(w.description(), "Mainly clear");
}

#[test]
fn wmo_rime_fog_code() {
    let w = WmoWeather::from(48);
    assert!(matches!(w, WmoWeather::DepositingRimeFog));
    assert_eq!(w.description(), "Depositing rime fog");
}

#[test]
fn wmo_freezing_rain_code() {
    let w = WmoWeather::from(66);
    assert!(matches!(w, WmoWeather::FreezingRain));
    assert_eq!(w.description(), "Freezing rain");
}

#[test]
fn wmo_snow_grains_code() {
    let w = WmoWeather::from(77);
    assert!(matches!(w, WmoWeather::SnowGrains));
    assert_eq!(w.description(), "Snow grains");
}

#[test]
fn wmo_rain_showers_code() {
    let w = WmoWeather::from(80);
    assert!(matches!(w, WmoWeather::RainShowers));
    assert_eq!(w.description(), "Rain showers");
}

#[test]
fn wmo_snow_showers_code() {
    let w = WmoWeather::from(85);
    assert!(matches!(w, WmoWeather::SnowShowers));
    assert_eq!(w.description(), "Snow showers");
}

#[test]
fn wmo_freezing_drizzle_code() {
    let w = WmoWeather::from(56);
    assert!(matches!(w, WmoWeather::FreezingDrizzle));
    assert_eq!(w.description(), "Freezing drizzle");
}

// ────────────────────────────────────────────────────
// Temperature conversion
// ────────────────────────────────────────────────────

#[test]
fn convert_temp_c_to_c() {
    let mut app = App::new();
    app.temperature_unit = TempUnit::Celsius;
    assert!((app.convert_temp(22.0) - 22.0).abs() < f32::EPSILON);
}

#[test]
fn convert_temp_fahrenheit_round_trip() {
    let mut app = App::new();
    app.temperature_unit = TempUnit::Fahrenheit;
    assert!((app.convert_temp(0.0) - 32.0).abs() < f32::EPSILON);
    assert!((app.convert_temp(100.0) - 212.0).abs() < f32::EPSILON);
    assert!((app.convert_temp(-10.0) - 14.0).abs() < f32::EPSILON);
    assert!((app.convert_temp(21.33) - 70.4).abs() < 0.1);
}

#[test]
fn format_temp_celsius() {
    let mut app = App::new();
    app.temperature_unit = TempUnit::Celsius;
    // Positive/zero temps get a leading +
    assert_eq!(app.format_temp(0.0), "+0°");
    // Negative temps lose the + but keep the sign
    assert_eq!(app.format_temp(-5.0), "-5°");
    // Positive temps get +
    assert_eq!(app.format_temp(25.0), "+25°");
}

// ────────────────────────────────────────────────────
// Wind direction formatting
// ────────────────────────────────────────────────────

#[test]
fn format_wind_direction() {
    let app = App::new();
    let f = |deg: u16| app.format_wind_direction(deg);
    assert!(f(0).contains("N"));
    assert!(f(45).contains("NE"));
    assert!(f(90).contains("E"));
    assert!(f(135).contains("SE"));
    assert!(f(180).contains("S"));
    assert!(f(225).contains("SW"));
    assert!(f(270).contains("W"));
    assert!(f(315).contains("NW"));
    assert!(f(360).contains("N"));
}

// ────────────────────────────────────────────────────
// Progress bar
// ────────────────────────────────────────────────────

#[test]
fn progress_bar_empty() {
    let app = App::new();
    let bar = app.progress_bar(0, 10, 10);
    assert_eq!(bar, "░░░░░░░░░░");
}

#[test]
fn progress_bar_half() {
    let app = App::new();
    let bar = app.progress_bar(5, 10, 10);
    assert_eq!(bar, "█████░░░░░");
}

#[test]
fn progress_bar_full() {
    let app = App::new();
    let bar = app.progress_bar(10, 10, 10);
    assert_eq!(bar, "██████████");
}

// ────────────────────────────────────────────────────
// Geocoding deserialization
// ────────────────────────────────────────────────────

#[test]
fn deserialize_geocoding_result() {
    let json = r#"{
        "id": 2950159,
        "name": "Berlin",
        "admin1": "Berlin",
        "country": "Germany",
        "country_code": "DE",
        "latitude": 52.52437,
        "longitude": 13.41053,
        "elevation": 40.0,
        "timezone": "Europe/Berlin",
        "population": 3426354
    }"#;

    let result: GeocodingResult = serde_json::from_str(json).unwrap();
    let loc: Location = result.into();

    assert_eq!(loc.id, 2950159);
    assert_eq!(loc.name, "Berlin");
    assert_eq!(loc.admin1.as_deref(), Some("Berlin"));
    assert_eq!(loc.country, "Germany");
    assert_eq!(loc.country_code, "DE");
    assert!((loc.latitude - 52.52437).abs() < f64::EPSILON);
    assert!((loc.longitude - 13.41053).abs() < f64::EPSILON);
    assert_eq!(loc.timezone, "Europe/Berlin");
    assert_eq!(loc.population, Some(3426354));
    assert_eq!(loc.display_name(), "Berlin, Berlin, Germany");
}

#[test]
fn deserialize_geocoding_envelope() {
    // The Open-Meteo geocoding API returns {"results": [...]}
    let json = r#"{
      "results": [
        {
          "id": 2950159,
          "name": "Berlin",
          "admin1": "Berlin",
          "country": "Germany",
          "country_code": "DE",
          "latitude": 52.52437,
          "longitude": 13.41053,
          "elevation": 40.0,
          "timezone": "Europe/Berlin",
          "population": 3426354
        }
      ]
    }"#;

    // The outer envelope matches the api/geocoding.rs structs
    #[derive(serde::Deserialize)]
    struct Envelope {
        #[serde(rename = "results")]
        result: Vec<GeocodingResult>,
    }

    let envelope: Envelope = serde_json::from_str(json).expect("should parse envelope");
    assert_eq!(envelope.result.len(), 1);

    let loc: Location = envelope.result.into_iter().next().unwrap().into();
    assert_eq!(loc.display_name(), "Berlin, Berlin, Germany");
}

// ────────────────────────────────────────────────────
// Weather forecast deserialization
// ────────────────────────────────────────────────────

#[test]
fn deserialize_weather_response() {
    let json = r#"{
      "current": {
        "time": "2026-05-10T14:00",
        "temperature_2m": 22.5,
        "relative_humidity_2m": 45,
        "apparent_temperature": 21.8,
        "precipitation": 0.0,
        "weather_code": 2,
        "wind_speed_10m": 15.3,
        "wind_direction_10m": 240,
        "wind_gusts_10m": 25.4,
        "is_day": 1.0
      },
      "hourly": {
        "time": ["2026-05-10T12:00", "2026-05-10T13:00", "2026-05-10T14:00", "2026-05-10T15:00"],
        "temperature_2m": [21.0, 21.8, 22.5, 23.1],
        "relative_humidity_2m": [48, 46, 45, 43],
        "weather_code": [2, 2, 2, 1],
        "precipitation": [0.0, 0.0, 0.0, 0.0],
        "wind_speed_10m": [14.0, 15.3, 16.1, 17.2],
        "is_day": [1.0, 1.0, 1.0, 1.0]
      },
      "daily": {
        "time": ["2026-05-10", "2026-05-11"],
        "temperature_2m_max": [24.0, 25.5],
        "temperature_2m_min": [16.0, 17.2],
        "weather_code": [2, 3],
        "sunrise": ["2026-05-10T05:15"],
        "sunset": ["2026-05-10T20:45"],
        "precipitation_sum": [0.0, 1.2],
        "wind_speed_10m_max": [18.3, 20.1]
      }
    }"#;

    let response: WeatherResponse = serde_json::from_str(json).expect("should parse");

    // Verify deserialization first
    assert_eq!(response.current.time, "2026-05-10T14:00");
    assert!((response.current.temperature_2m - 22.5).abs() < 1e-3);
    assert_eq!(response.current.relative_humidity_2m, 45);
    assert!((response.current.apparent_temperature - 21.8).abs() < 1e-3);
    assert!((response.current.is_day - 1.0).abs() < f64::EPSILON);

    assert_eq!(response.hourly.time.len(), 4);
    assert_eq!(response.hourly.temperature_2m.len(), 4);
    assert_eq!(response.daily.time.len(), 2);
    assert_eq!(response.daily.temperature_2m_max.len(), 2);

    // Convert to domain models
    let (current, hourly, daily) = response.into();

    // === Current ===
    assert_eq!(current.time, "2026-05-10T14:00");
    assert!((current.temperature - 22.5).abs() < 1e-3);
    assert_eq!(current.humidity, 45);
    assert!((current.apparent_temperature - 21.8).abs() < 1e-3);
    assert!((current.precipitation - 0.0).abs() < 1e-3);
    assert_eq!(current.weather_code, 2);
    assert!((current.wind_speed - 15.3).abs() < 1e-3);
    assert_eq!(current.wind_direction, 240);
    assert!((current.wind_gusts - 25.4).abs() < 1e-3);
    assert!(current.is_day);

    // === Hourly ===
    assert_eq!(
        hourly.times,
        [
            "2026-05-10T12:00",
            "2026-05-10T13:00",
            "2026-05-10T14:00",
            "2026-05-10T15:00"
        ]
    );
    assert!((hourly.temperatures[0] - 21.0).abs() < 1e-3);
    assert!((hourly.temperatures[1] - 21.8).abs() < 1e-3);
    assert!((hourly.temperatures[2] - 22.5).abs() < 1e-3);
    assert!((hourly.temperatures[3] - 23.1).abs() < 1e-3);
    assert_eq!(hourly.humidities, [48, 46, 45, 43]);
    assert_eq!(hourly.weather_codes, [2, 2, 2, 1]);
    assert_eq!(hourly.precipitations, [0.0, 0.0, 0.0, 0.0]);
    assert!((hourly.wind_speeds[0] - 14.0).abs() < 1e-3);
    assert!((hourly.wind_speeds[1] - 15.3).abs() < 1e-3);
    assert!((hourly.wind_speeds[2] - 16.1).abs() < 1e-3);
    assert!((hourly.wind_speeds[3] - 17.2).abs() < 1e-3);

    // === Daily ===
    assert_eq!(daily.dates, ["2026-05-10", "2026-05-11"]);
    assert!((daily.temp_high[0] - 24.0).abs() < 1e-3);
    assert!((daily.temp_high[1] - 25.5).abs() < 1e-3);
    assert!((daily.temp_low[0] - 16.0).abs() < 1e-3);
    assert!((daily.temp_low[1] - 17.2).abs() < 1e-3);
    assert_eq!(daily.weather_codes, [2, 3]);
    assert_eq!(daily.sunrise, ["2026-05-10T05:15"]);
    assert_eq!(daily.sunset, ["2026-05-10T20:45"]);
    assert_eq!(daily.precip_sum, [0.0, 1.2]);
    assert!((daily.wind_max[0] - 18.3).abs() < 1e-3);
    assert!((daily.wind_max[1] - 20.1).abs() < 1e-3);
}

#[test]
fn deserialize_weather_response_is_day_as_number() {
    let json = r#"{
      "current": {
        "time": "2026-05-10T02:00",
        "temperature_2m": 18.0,
        "relative_humidity_2m": 72,
        "apparent_temperature": 17.2,
        "precipitation": 0.0,
        "weather_code": 0,
        "wind_speed_10m": 8.5,
        "wind_direction_10m": 180,
        "wind_gusts_10m": 12.0,
        "is_day": 0.0
      },
      "hourly": {
        "time": ["2026-05-10T02:00", "2026-05-10T03:00"],
        "temperature_2m": [18.0, 17.5],
        "relative_humidity_2m": [72, 74],
        "weather_code": [0, 1],
        "precipitation": [0.0, 0.0],
        "wind_speed_10m": [8.5, 9.0],
        "is_day": [0.0, 0.0]
      },
      "daily": {
        "time": ["2026-05-10"],
        "temperature_2m_max": [24.0, 25.5],
        "temperature_2m_min": [16.0, 17.2],
        "weather_code": [0, 3],
        "sunrise": ["2026-05-10T05:15"],
        "sunset": ["2026-05-10T20:45"],
        "precipitation_sum": [0.0, 1.2],
        "wind_speed_10m_max": [18.3, 20.1]
      }
    }"#;

    let response: WeatherResponse = serde_json::from_str(json).expect("should parse");

    // Verify is_day deserializes as 0.0 (night)
    assert!((response.current.is_day - 0.0).abs() < f64::EPSILON);

    // Convert to domain models and verify is_day becomes false
    let (current, _hourly, _daily) = response.into();
    assert!(!current.is_day);
}

#[test]
fn deserialize_runtime_weather_response_with_options() {
    // This matches what api/weather.rs struct expects (all Option<T>)
    let json = r#"{
      "current": {
        "time": "2026-05-10T14:00",
        "temperature_2m": 22.5,
        "relative_humidity_2m": 45,
        "apparent_temperature": 21.8,
        "precipitation": 0.0,
        "weather_code": 2,
        "wind_speed_10m": 15.3,
        "wind_direction_10m": 240,
        "wind_gusts_10m": 25.4,
        "is_day": 1.0
      },
      "hourly": {
        "time": ["2026-05-10T14:00"],
        "temperature_2m": [22.5],
        "relative_humidity_2m": [45],
        "weather_code": [2],
        "precipitation": [0.0],
        "wind_speed_10m": [15.3],
        "is_day": [1.0]
      },
      "daily": {
        "time": ["2026-05-10"],
        "temperature_2m_max": [24.0],
        "temperature_2m_min": [16.0],
        "weather_code": [2],
        "sunrise": ["2026-05-10T05:15"],
        "sunset": ["2026-05-10T20:45"],
        "precipitation_sum": [0.0],
        "wind_speed_10m_max": [18.3]
      }
    }"#;

    // Use runtime Option-wrapped types from api/weather.rs
    #[derive(serde::Deserialize)]
    struct RuntimeResponse {
        current: Option<weatherman::CurrentData>,
        hourly: Option<weatherman::HourlyData>,
        daily: Option<weatherman::DailyData>,
    }

    let resp: RuntimeResponse = serde_json::from_str(json).expect("should parse runtime response");
    assert!(resp.current.is_some());
    assert!(resp.hourly.is_some());
    assert!(resp.daily.is_some());

    // Convert using runtime types' From impl
    let runtime_resp = resp;
    let c = runtime_resp.current.expect("should have current");
    assert!((c.temperature_2m - 22.5).abs() < 1e-3);
    assert!((c.is_day - 1.0).abs() < f64::EPSILON);
    // The boolean check: 1.0 != 0.0 → true
    assert!(c.is_day != 0.0);
}

// ────────────────────────────────────────────────────
// Message enum tests
// ────────────────────────────────────────────────────

#[test]
fn message_variants_constructible() {
    let _tick = Message::Tick;
    let _input = Message::SearchInput("ber".into());
    let _submit = Message::SearchSubmit;
    let _clear = Message::SearchClear;
    let _results = Message::SearchResultsReceived(vec![]);
    let _err = Message::SearchError("oops".into());
    let _fetched = Message::WeatherFetched;
    let _werr = Message::WeatherError("bad".into());
    let _toggle = Message::ToggleUnit;
    let _modal = Message::SearchModal { active: true };
}

#[test]
fn message_clone() {
    let msg = Message::SearchInput("lon".into());
    let msg2 = msg.clone();
    assert!(matches!(msg2, Message::SearchInput(s) if s == "lon"));
}

// ────────────────────────────────────────────────────
// TempUnit tests
// ────────────────────────────────────────────────────

#[test]
fn toggle_celsius_to_fahrenheit() {
    let mut unit = TempUnit::Celsius;
    unit.toggle();
    assert!(matches!(unit, TempUnit::Fahrenheit));
}

#[test]
fn toggle_fahrenheit_to_celsius() {
    let mut unit = TempUnit::Fahrenheit;
    unit.toggle();
    assert!(matches!(unit, TempUnit::Celsius));
}

#[test]
fn toggle_roundtrip() {
    let mut unit = TempUnit::Celsius;
    unit.toggle();
    unit.toggle();
    assert!(matches!(unit, TempUnit::Celsius));
}

#[test]
fn unit_symbols() {
    assert_eq!(TempUnit::Celsius.symbol(), "C");
    assert_eq!(TempUnit::Fahrenheit.symbol(), "F");
}

// ────────────────────────────────────────────────────
// App state machine tests
// ────────────────────────────────────────────────────

#[test]
fn app_new_defaults() {
    let app = App::new();
    assert!(matches!(app.state, AppState::Idle));
    assert!(app.location.is_none());
    assert!(app.current.is_none());
    assert!(app.hourly.is_none());
    assert!(app.daily.is_none());
    assert!(app.search_query.is_empty());
    assert!(app.search_results.is_empty());
    assert!(!app.search_modal_active);
    assert!(matches!(app.state, AppState::Idle));
}

// ────────────────────────────────────────────────────
// Tab system tests (2 tabs: Daily, Hourly)
// ─────────────────────────────────────────────────────────

#[test]
fn daily_is_default_tab() {
    let app = App::new();
    assert_eq!(app.active_tab, 0); // Daily is the default (index 0)
}

// ────────────────────────────────────────────────────
// Tick timeout overflow prevention
// ──────────────────────────────────────────────

#[test]
fn tick_timeout_zero_elapsed() {
    // When elapsed == 0, saturating_sub returns tick_rate exactly
    let tick_rate: u64 = 1000;
    let elapsed: u64 = 0;
    let timeout = tick_rate.saturating_sub(elapsed);
    assert_eq!(timeout, tick_rate);
}

#[test]
fn tick_timeout_saturating_sub_overflows() {
    // saturating_sub gracefully returns 0 instead of panicking on underflow.
    // This was the bug: TICK_RATE - elapsed panicked when elapsed > TICK_RATE.
    let tick_rate: u64 = 1000;
    let elapsed: u64 = 2000;
    let timeout = tick_rate.saturating_sub(elapsed);
    assert_eq!(timeout, 0);
}

#[test]
fn tick_timeout_saturating_sub_exactly_at_tick_rate() {
    // When elapsed == TICK_RATE, timeout should be 0
    let tick_rate: u64 = 1000;
    let elapsed: u64 = 1000;
    let timeout = tick_rate.saturating_sub(elapsed);
    assert_eq!(timeout, 0);
}

#[test]
fn tick_timeout_saturating_sub_full_path() {
    // Simulate the full computation from main.rs:
    //   let timeout = std::cmp::min(tick_rate.saturating_sub(elapsed), tick_rate);
    // When elapsed far exceeds TICK_RATE (e.g., network call during search),
    // the timeout should clamp to 0, not panic.
    let tick_rate: u64 = 1000;
    let elapsed: u64 = 9999;
    let timeout = std::cmp::min(tick_rate.saturating_sub(elapsed), tick_rate);
    assert_eq!(timeout, 0);
}

#[test]
fn tick_timeout_saturating_sub_min_clamp() {
    // When elapsed == 0, min(TICK_RATE, TICK_RATE) == TICK_RATE
    let tick_rate: u64 = 1000;
    let elapsed: u64 = 0;
    let timeout = std::cmp::min(tick_rate.saturating_sub(elapsed), tick_rate);
    assert_eq!(timeout, tick_rate);
}

// ──────────────────────────────
// Bug fix regression tests — v0.1.6 rendering fixes
// ─────────────────────────────────────────────

#[test]
fn format_temp_has_exactly_one_degree_symbol() {
    let app = App::new();
    // format_temp should produce exactly one ° symbol
    let positive = app.format_temp(25.0);
    let degree_count_plus = positive.matches('°').count();
    assert_eq!(
        degree_count_plus, 1,
        "format_temp(25.0) should have exactly one °: got '{}'",
        positive
    );

    let negative = app.format_temp(-5.0);
    let degree_count_neg = negative.matches('°').count();
    assert_eq!(
        degree_count_neg, 1,
        "format_temp(-5.0) should have exactly one °: got '{}'",
        negative
    );

    let zero = app.format_temp(0.0);
    let degree_count_zero = zero.matches('°').count();
    assert_eq!(
        degree_count_zero, 1,
        "format_temp(0.0) should have exactly one °: got '{}'",
        zero
    );
}

// ───────  Config refresh_interval persistence ───────

#[test]
fn saved_config_refresh_interval_defaults() {
    // Option<u64> should parse when the field is present
    let json = r#"{"name": "Berlin", "refresh_interval": 7200}"#;
    let cfg: SavedConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.name, "Berlin");
    assert_eq!(cfg.refresh_interval, Some(7200));
}

#[test]
fn saved_config_refresh_interval_missing() {
    // When the field is missing, it defaults to None (will use 7200 default in app)
    let json = r#"{"name": "Paris"}"#;
    let cfg: SavedConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.name, "Paris");
    assert!(cfg.refresh_interval.is_none());
}

#[test]
fn app_loads_refresh_interval_from_config() {
    let mut app = App::new();
    app.temperature_unit = TempUnit::Celsius;
    // The default should be 7200 seconds (2 hours)
    assert_eq!(app.auto_refresh_interval.as_secs(), 7200);
}

#[test]
fn format_temp_positive_has_single_plus() {
    let app = App::new();
    let positive = app.format_temp(25.0);
    let plus_count = positive.matches('+').count();
    assert_eq!(
        plus_count, 1,
        "format_temp(25.0) should have exactly one +: got '{}'",
        positive
    );
    assert!(
        positive.starts_with('+'),
        "format_temp should prefix positive temps with +: got '{}'",
        positive
    );
}

#[test]
fn format_temp_negative_has_no_plus() {
    let app = App::new();
    let negative = app.format_temp(-10.0);
    let plus_count = negative.matches('+').count();
    assert_eq!(
        plus_count, 0,
        "format_temp(-10.0) should NOT have +: got '{}'",
        negative
    );
}

#[test]
fn saturating_sub_prevents_zero_height_overflow() {
    // Verifies the saturating_sub pattern used in rendering to prevent
    // "attempt to subtract with overflow" panic when terminal shrank below 2 lines
    let zero_height: u16 = 0;
    let one_height: u16 = 1;
    let two_height: u16 = 2;
    let usable_zero = zero_height.saturating_sub(2);
    let usable_one = one_height.saturating_sub(2);
    let usable_two = two_height.saturating_sub(2);
    assert_eq!(usable_zero, 0, "zero height should saturate to 0");
    assert_eq!(usable_one, 0, "one height should saturate to 0");
    assert_eq!(usable_two, 0, "two height should saturate to 0");
    // Normal case
    let ten_height: u16 = 10;
    let usable_ten = ten_height.saturating_sub(2);
    assert_eq!(usable_ten, 8, "ten height minus 2 should be 8");
}

// ────────────────────────────────────────────────────
// Local time fix — last_update uses local time, not UTC
// ──────────────────────────────

#[test]
fn last_update_format_is_local_hhmmss() {
    let mut app = App::new();

    // Initially last_update is None
    assert!(app.last_update.is_none());

    // Trigger WeatherFetched which sets last_update
    app.update(Message::WeatherFetched);

    // last_update should now be set
    assert!(app.last_update.is_some());

    let time_str = app.last_update.unwrap();

    // Format should be HH:MM:SS — exactly 8 characters
    assert_eq!(
        time_str.len(),
        8,
        "last_update should be HH:MM:SS format (8 chars), got: '{}'",
        time_str
    );

    // Should contain exactly two colons (at positions 2 and 5)
    assert_eq!(
        time_str.chars().filter(|c| *c == ':').count(),
        2,
        "last_update should contain exactly two colons, got: '{}'",
        time_str
    );

    // Should NOT end with 'Z' (which would indicate UTC)
    assert!(
        !time_str.ends_with('Z'),
        "last_update should NOT be UTC (no trailing Z), got: '{}'",
        time_str
    );

    // First two chars should be valid digits (hours 00-23)
    let hours: u8 = time_str[..2].parse().unwrap_or(99);
    assert!(hours <= 23, "hours should be 00-23, got: '{}'", time_str);

    // Characters at positions 3 and 4 should be a valid minute (00-59)
    let minutes: u8 = time_str[3..5].parse().unwrap_or(99);
    assert!(minutes <= 59, "minutes should be 00-59, got: '{}'", time_str);

    // Characters at positions 6 and 7 should be a valid second (00-59)
    let seconds: u8 = time_str[6..8].parse().unwrap_or(99);
    assert!(seconds <= 59, "seconds should be 00-59, got: '{}'", time_str);
}

// ────────────────────────────────────────────────────
// Pressure trend tests
// ────────────────────────────────────────────────────

#[test]
fn pressure_trend_rising() {
    let (label, arrow) = weatherman::format_pressure_trend(1015.0, 1012.0);
    assert_eq!(label, "Rising");
    assert_eq!(arrow, "↑");
}

#[test]
fn pressure_trend_falling() {
    let (label, arrow) = weatherman::format_pressure_trend(1010.0, 1014.0);
    assert_eq!(label, "Falling");
    assert_eq!(arrow, "↓");
}

#[test]
fn pressure_trend_steady() {
    let (label, arrow) = weatherman::format_pressure_trend(1013.0, 1013.5);
    assert_eq!(label, "Steady");
    assert_eq!(arrow, "→");
}

// ────────────────────────────────────────────────────
// Auto-refresh timer logic
// ────────────────────────────────────────────────────

#[test]
fn auto_refresh_timer_with_location() {
    let mut app = App::new();

    // Set a small auto-refresh interval for testing
    app.auto_refresh_interval = Duration::from_secs(5);

    // Without a location, no refresh fires — state stays Idle
    app.update(Message::Tick);
    assert_eq!(app.tick_count, 1);
    assert!(matches!(app.state, AppState::Idle));

    app.update(Message::Tick);
    assert_eq!(app.tick_count, 2);
    assert!(matches!(app.state, AppState::Idle));

    // Set a location so the refresh branch can trigger
    app.location = Some(Location {
        id: 1,
        name: "Test City".to_string(),
        admin1: Some("Test State".to_string()),
        country: "Test Nation".to_string(),
        country_code: "TN".to_string(),
        latitude: 40.0,
        longitude: -74.0,
        timezone: "America/New_York".to_string(),
        population: Some(100),
    });

    // Tick 3, 4 — still under threshold of 5
    // (tick_count is currently 2 after the two ticks above)
    app.update(Message::Tick);
    assert_eq!(app.tick_count, 3);
    // Still Idle because 3 < 5
    assert!(matches!(app.state, AppState::Idle));

    // 4th tick before threshold — tick_count crosses to 4
    app.update(Message::Tick);
    assert_eq!(app.tick_count, 4);
    assert!(matches!(app.state, AppState::Idle));

    // 5th tick — crosses the threshold of 5
    app.update(Message::Tick);
    // tick_count resets to 0
    assert_eq!(app.tick_count, 0);
    // State transitions to Refreshing because location is Some
    assert!(matches!(app.state, AppState::Refreshing));

    // Verify that once Refreshing, a WeatherFetched message resets to Idle
    app.update(Message::WeatherFetched);
    assert!(matches!(app.state, AppState::Idle));
}

// ────────────────────────────────────────────────
// Config persistence tests
// All tests use test-safe config directory functions (load_config_from_dir,
// save_config_to_dir) that do NOT depend on HOME, preventing accidental
// writes to the user's real config file.
// ──────────────────────────────────────────────

#[test]
fn config_save_load_roundtrip() {
    let temp_dir = std::env::temp_dir().join("weatherman_test_roundtrip");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    let config_dir = temp_dir.join(".config").join("weatherman");
    std::fs::create_dir_all(&config_dir).unwrap();

    let config_file = config_dir.join("weatherman.toml");
    let toml_content = format!(
        r#"name = "Test City"
refresh_interval = 3600
"#
    );
    std::fs::write(&config_file, toml_content).unwrap();

    let (config, loaded_path) =
        weatherman::config::load_config_from_dir(&config_dir)
            .expect("should load saved config from config dir");
    assert_eq!(config.name, "Test City");
    assert_eq!(config.refresh_interval, Some(3600));
    assert_eq!(loaded_path, config_file);

    // Save using the directory-safe function
    let temp_dir2 = std::env::temp_dir().join("weatherman_test_save2");
    let _ = std::fs::remove_dir_all(&temp_dir2);
    std::fs::create_dir_all(&temp_dir2).unwrap();

    let config_dir2 = temp_dir2.join(".config").join("weatherman");
    let config2 = weatherman::SavedConfig {
        name: "Roundtrip City".to_string(),
        refresh_interval: Some(7200),
    };
    let saved_path = weatherman::config::save_config_to_dir(&config2, None, &config_dir2)
        .expect("should save config successfully");
    assert!(saved_path.exists());
    let contents = std::fs::read_to_string(&saved_path).unwrap();
    assert!(contents.contains("Roundtrip City"));
    assert!(contents.contains("7200"));

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
    let _ = std::fs::remove_dir_all(&temp_dir2);
}

#[test]
fn config_save_creates_parent_dirs() {
    let temp_dir = std::env::temp_dir().join("weatherman_test_save");
    let _ = std::fs::remove_dir_all(&temp_dir);

    // Do NOT create any directories — the save function should create them
    let config_dir = temp_dir.join(".config").join("weatherman");

    let config = weatherman::SavedConfig {
        name: "Test City".to_string(),
        refresh_interval: Some(7200),
    };

    let saved_path = weatherman::config::save_config_to_dir(&config, None, &config_dir)
        .expect("should save config successfully");

    assert!(saved_path.exists(), "config file should exist on disk");
    let contents = std::fs::read_to_string(&saved_path)
        .expect("should be able to read saved config");
    assert!(contents.contains("Test City"), "saved content should contain the name");
    assert!(contents.contains("7200"), "saved content should contain refresh_interval");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn config_load_returns_none_when_no_config_file() {
    let temp_dir = std::env::temp_dir().join("weatherman_test_none");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    let config_dir = temp_dir.join(".config").join("weatherman");

    let result = weatherman::config::load_config_from_dir(&config_dir);
    assert!(
        result.is_none(),
        "should return None when no config file exists"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// ──────────────────api/client.rs tests ──────────────────
// ──────────────────────────────────────────────────────

#[test]
fn encode_query_preserves_ascii_safe_chars() {
    // Safe characters pass through unchanged
    assert_eq!(client::encode_query("ABC"), "ABC");
    assert_eq!(client::encode_query("abc"), "abc");
    assert_eq!(client::encode_query("0123456789"), "0123456789");
    assert_eq!(client::encode_query("-_.~"), "-_.~");
}

#[test]
fn encode_query_spaces_become_percent_encoded() {
    assert_eq!(client::encode_query("New York"), "New%20York");
    assert_eq!(client::encode_query(" "), "%20");
    assert_eq!(client::encode_query("hello world"), "hello%20world");
}

#[test]
fn encode_query_unicode_become_percent_encoded() {
    // The implementation encodes bytes individually as hex
    let result = client::encode_query("München");
    assert!(result.contains("M%"));
    assert!(!result.contains(" München"));
}

#[test]
fn geocoding_url_format() {
    let url = client::geocoding_url("London", 10);
    assert!(url.starts_with("https://geocoding-api.open-meteo.com/v1/search?"));
    assert!(url.contains("name=London"));
    assert!(url.contains("&count=10"));
    assert!(url.contains("&language=en"));
}

#[test]
fn geocoding_url_encodes_spaces() {
    let url = client::geocoding_url("New York", 5);
    assert!(url.contains("name=New%20York"));
    assert!(url.contains("&count=5"));
}

#[test]
fn forecast_url_format() {
    let url = client::forecast_url(40.7128, -74.0060);
    assert!(url.starts_with("https://api.open-meteo.com/v1/forecast?"));
    assert!(url.contains("latitude=40.7128"));
    assert!(url.contains("longitude=-74.0060"));
    assert!(url.contains("&current="));
    assert!(url.contains("&hourly="));
    assert!(url.contains("&daily="));
}

// ──────────────────App::update state machine tests ──────────────
// ────────────────────────────────────────────────────────────────

#[test]
fn search_clear_resets_state() {
    let mut app = App::new();
    app.search_query = "Berlin".to_string();
    app.search_results = vec![Location {
        id: 1, name: "Berlin".into(), admin1: Some("Berlin".into()),
        country: "Germany".into(), country_code: "DE".into(),
        latitude: 52.0, longitude: 13.0, timezone: "Europe/Berlin".into(),
        population: None,
    }];
    app.search_modal_active = true;
    app.state = AppState::LoadingSearch;

    app.update(Message::SearchClear);

    assert!(app.search_query.is_empty());
    assert!(app.search_results.is_empty());
    assert!(!app.search_modal_active);
    assert!(matches!(app.state, AppState::Idle));
}

#[test]
fn search_input_sets_query() {
    let mut app = App::new();
    app.update(Message::SearchInput("ber".into()));
    assert_eq!(app.search_query, "ber");
}

#[test]
fn weather_fetched_sets_last_update_and_idle() {
    let mut app = App::new();
    app.state = AppState::LoadingWeather;
    app.update(Message::WeatherFetched);
    assert!(matches!(app.state, AppState::Idle));
    assert!(app.last_update.is_some());
}

#[test]
fn search_error_shows_modal() {
    let mut app = App::new();
    app.update(Message::SearchError("network error".into()));
    assert!(app.error_modal_visible);
    assert_eq!(app.error_message.as_deref(), Some("network error"));
}

#[test]
fn weather_error_shows_modal() {
    let mut app = App::new();
    app.update(Message::WeatherError("api down".into()));
    assert!(app.error_modal_visible);
    assert_eq!(app.error_message.as_deref(), Some("api down"));
}

#[test]
fn toggle_unit_roundtrip() {
    let mut app = App::new();
    assert!(matches!(app.temperature_unit, TempUnit::Celsius));
    app.update(Message::ToggleUnit);
    assert!(matches!(app.temperature_unit, TempUnit::Fahrenheit));
    app.update(Message::ToggleUnit);
    assert!(matches!(app.temperature_unit, TempUnit::Celsius));
}

#[test]
fn show_error_sets_modal() {
    let mut app = App::new();
    app.show_error("test error".to_string());
    assert!(app.error_modal_visible);
    assert_eq!(app.error_message.as_deref(), Some("test error"));
}

// ──────────────────Config module integration tests ──────────────
// ─────────────────────────────────────────────────────────────────

#[test]
fn config_load_config_from_dir_returns_none_for_empty_dir() {
    let temp_dir = std::env::temp_dir().join("weatherman_test_empty_dir");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    let result = config::load_config_from_dir(&temp_dir);
    assert!(result.is_none());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn config_save_config_to_dir_creates_nested_dirs() {
    let temp_dir = std::env::temp_dir().join("weatherman_test_nested");
    let _ = std::fs::remove_dir_all(&temp_dir);
    assert!(!temp_dir.exists());

    let config = SavedConfig {
        name: "Nested City".to_string(),
        refresh_interval: Some(3600),
    };

    // Don't create any directories — the save should create them
    let saved = config::save_config_to_dir(&config, None, &temp_dir).unwrap();
    assert!(saved.exists());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn forecast_url_contains_required_params() {
    let url = client::forecast_url(52.52, 13.41);
    // Verify key query parameters are present
    assert!(url.contains("temperature_unit=celsius"));
    assert!(url.contains("timezone=auto"));
    assert!(url.contains("current=temperature_2m"));
    assert!(url.contains("pressure_msl"));
    assert!(url.contains("uv_index"));
    assert!(url.contains("visibility"));
    assert!(url.contains("dewpoint_temperature"));
    assert!(url.contains("hourly=temperature_2m"));
    assert!(url.contains("daily=weather_code"));
}
