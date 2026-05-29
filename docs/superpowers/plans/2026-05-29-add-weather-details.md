# Add Weather Details: Pressure, UV Index, Visibility, Dew Point

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add four new data points to the Current Conditions sidebar: barometric pressure with trend (rising/falling/steady), UV index, visibility, and dew point.

**Architecture:** Extend the existing data pipeline end-to-end: API URL → fetch structs → domain models → widget rendering. Pressure trend requires hourly pressure data to compare the current reading against 3 hours prior. All new API fields use `Option<T>` in fetch structs (matching existing pattern) to handle missing data gracefully.

**Tech Stack:** Open-Meteo API fields (`pressure_msl`, `uv_index`, `visibility`, `dewpoint_temperature`), serde `Option<T>` deserialization, ratatui widget rendering.

---

## File Map

| File | Change |
|------|--------|
| `src/api/client.rs` | Add 4 new fields to `current=` param, add `pressure_msl` to `hourly=` param |
| `src/api/weather.rs` | Add fields to `FetchCurrentData`, `FetchHourlyData`, `TestCurrentData`, `TestHourlyData`; update `extract_models()` and `From<TestWeatherResponse>` |
| `src/app.rs` | Add fields to `CurrentWeather` and `HourlyForecast` structs |
| `src/ui/widgets/current.rs` | Add 4 new display lines with formatting |
| `src/ui/helpers.rs` | Add `format_pressure_trend()` helper |
| `tests/unit_tests.rs` | Update test JSON fixtures, add new field assertions |

---

### Task 1: Add new fields to API URL

**Files:**
- Modify: `src/api/client.rs:38-42`
- Test: `tests/unit_tests.rs` (forecast_url tests)

- [ ] **Step 1: Update `forecast_url` to request new current fields**

In `src/api/client.rs`, change the `current=` parameter to include `pressure_msl,uv_index,visibility,dewpoint_temperature`:

```rust
pub fn forecast_url(lat: f64, lon: f64) -> String {
    format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat:.6}&longitude={lon:.6}&current=temperature_2m,relative_humidity_2m,apparent_temperature,precipitation,weather_code,wind_speed_10m,wind_direction_10m,wind_gusts_10m,is_day,pressure_msl,uv_index,visibility,dewpoint_temperature&hourly=temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,precipitation,wind_speed_10m,is_day,pressure_msl&daily=weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,precipitation_sum,wind_speed_10m_max&timezone=auto&temperature_unit=celsius",
    )
}
```

- [ ] **Step 2: Update forecast_url test to verify new fields**

In `tests/unit_tests.rs`, update `forecast_url_contains_required_params` to assert the new fields:

```rust
#[test]
fn forecast_url_contains_required_params() {
    let url = client::forecast_url(52.52, 13.41);
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
```

- [ ] **Step 3: Run tests to verify**

Run: `nix-shell --run "cargo test forecast_url"`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/api/client.rs tests/unit_tests.rs
git commit -m "feat: add pressure, uv, visibility, dewpoint to API request"
```

---

### Task 2: Add fields to fetch structs

**Files:**
- Modify: `src/api/weather.rs:18-30` (FetchCurrentData)
- Modify: `src/api/weather.rs:32-41` (FetchHourlyData)

- [ ] **Step 1: Add fields to `FetchCurrentData`**

```rust
#[derive(Debug, Deserialize)]
struct FetchCurrentData {
    pub time: String,
    pub temperature_2m: f32,
    pub relative_humidity_2m: u8,
    pub apparent_temperature: f32,
    pub precipitation: f32,
    pub weather_code: u8,
    pub wind_speed_10m: f32,
    pub wind_direction_10m: u16,
    pub wind_gusts_10m: f32,
    pub is_day: f64,
    pub pressure_msl: Option<f32>,
    pub uv_index: Option<f32>,
    pub visibility: Option<f32>,
    pub dewpoint_temperature: Option<f32>,
}
```

- [ ] **Step 2: Add `pressure_msl` to `FetchHourlyData`**

```rust
#[derive(Debug, Deserialize)]
struct FetchHourlyData {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f32>,
    pub relative_humidity_2m: Vec<u8>,
    pub weather_code: Vec<u8>,
    pub precipitation: Vec<f32>,
    pub wind_speed_10m: Vec<f32>,
    pub is_day: Vec<f64>,
    pub pressure_msl: Option<Vec<f32>>,
}
```

- [ ] **Step 3: Verify compilation**

Run: `nix-shell --run "cargo check"`
Expected: PASS (new fields are Option, so existing code still works)

- [ ] **Step 4: Commit**

```bash
git add src/api/weather.rs
git commit -m "feat: add Option fields to fetch structs for pressure, uv, visibility, dewpoint"
```

---

### Task 3: Add fields to test structs

**Files:**
- Modify: `src/api/weather.rs:73-85` (TestCurrentData)
- Modify: `src/api/weather.rs:90-99` (TestHourlyData)

- [ ] **Step 1: Add fields to `TestCurrentData`**

```rust
#[derive(serde::Deserialize, Debug, Clone)]
pub struct TestCurrentData {
    pub time: String,
    pub temperature_2m: f32,
    pub relative_humidity_2m: u8,
    pub apparent_temperature: f32,
    pub precipitation: f32,
    pub weather_code: u8,
    pub wind_speed_10m: f32,
    pub wind_direction_10m: u16,
    pub wind_gusts_10m: f32,
    pub is_day: f64,
    pub pressure_msl: Option<f32>,
    pub uv_index: Option<f32>,
    pub visibility: Option<f32>,
    pub dewpoint_temperature: Option<f32>,
}
```

- [ ] **Step 2: Add `pressure_msl` to `TestHourlyData`**

```rust
#[derive(serde::Deserialize, Debug, Clone)]
pub struct TestHourlyData {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f32>,
    pub relative_humidity_2m: Vec<u8>,
    pub weather_code: Vec<u8>,
    pub precipitation: Vec<f32>,
    pub wind_speed_10m: Vec<f32>,
    pub is_day: Vec<f64>,
    pub pressure_msl: Option<Vec<f32>>,
}
```

- [ ] **Step 3: Verify compilation**

Run: `nix-shell --run "cargo check"`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/api/weather.rs
git commit -m "feat: add pressure, uv, visibility, dewpoint to test structs"
```

---

### Task 4: Add fields to domain models

**Files:**
- Modify: `src/app.rs:192-204` (CurrentWeather)
- Modify: `src/app.rs:206-215` (HourlyForecast)

- [ ] **Step 1: Add fields to `CurrentWeather`**

```rust
#[derive(Debug, Clone)]
pub struct CurrentWeather {
    pub time: String,
    pub temperature: f32,
    pub apparent_temperature: f32,
    pub humidity: u8,
    pub wind_speed: f32,
    pub wind_direction: u16,
    pub wind_gusts: f32,
    pub precipitation: f32,
    pub weather_code: u8,
    pub is_day: bool,
    pub pressure: Option<f32>,
    pub uv_index: Option<f32>,
    pub visibility: Option<f32>,
    pub dewpoint: Option<f32>,
}
```

- [ ] **Step 2: Add `pressures` to `HourlyForecast`**

```rust
#[derive(Debug, Clone)]
pub struct HourlyForecast {
    pub times: Vec<String>,
    pub temperatures: Vec<f32>,
    pub humidities: Vec<u8>,
    pub weather_codes: Vec<u8>,
    pub precipitations: Vec<f32>,
    pub wind_speeds: Vec<f32>,
    pub is_day: Vec<bool>,
    pub pressures: Option<Vec<f32>>,
}
```

- [ ] **Step 3: Verify compilation**

Run: `nix-shell --run "cargo check"`
Expected: FAIL — `extract_models` and `From` impl don't populate new fields yet.

- [ ] **Step 4: Commit**

```bash
git add src/app.rs
git commit -m "feat: add pressure, uv, visibility, dewpoint to domain models"
```

---

### Task 5: Wire conversion impls

**Files:**
- Modify: `src/api/weather.rs:124-162` (extract_models)
- Modify: `src/api/weather.rs:164-207` (From<TestWeatherResponse>)

- [ ] **Step 1: Update `extract_models` to map new fields**

```rust
fn extract_models(resp: FetchWeatherResponse) -> anyhow::Result<(CurrentWeather, HourlyForecast, DailyForecast)> {
    let c = resp.current.ok_or_else(|| anyhow::anyhow!("Missing current data"))?;
    let h = resp.hourly.ok_or_else(|| anyhow::anyhow!("Missing hourly data"))?;
    let d = resp.daily.ok_or_else(|| anyhow::anyhow!("Missing daily data"))?;

    Ok((
        CurrentWeather {
            time: c.time,
            temperature: c.temperature_2m,
            apparent_temperature: c.apparent_temperature,
            humidity: c.relative_humidity_2m,
            wind_speed: c.wind_speed_10m,
            wind_direction: c.wind_direction_10m,
            wind_gusts: c.wind_gusts_10m,
            precipitation: c.precipitation,
            weather_code: c.weather_code,
            is_day: c.is_day != 0.0,
            pressure: c.pressure_msl,
            uv_index: c.uv_index,
            visibility: c.visibility,
            dewpoint: c.dewpoint_temperature,
        },
        HourlyForecast {
            times: h.time,
            temperatures: h.temperature_2m,
            humidities: h.relative_humidity_2m,
            weather_codes: h.weather_code,
            precipitations: h.precipitation,
            wind_speeds: h.wind_speed_10m,
            is_day: h.is_day.iter().map(|v| *v != 0.0).collect(),
            pressures: h.pressure_msl,
        },
        DailyForecast {
            dates: d.time,
            temp_high: d.temperature_2m_max,
            temp_low: d.temperature_2m_min,
            weather_codes: d.weather_code,
            sunrise: d.sunrise,
            sunset: d.sunset,
            precip_sum: d.precipitation_sum,
            wind_max: d.wind_speed_10m_max,
        },
    ))
}
```

- [ ] **Step 2: Update `From<TestWeatherResponse>` impl**

```rust
impl From<TestWeatherResponse> for (
    crate::app::CurrentWeather,
    crate::app::HourlyForecast,
    crate::app::DailyForecast,
) {
    fn from(r: TestWeatherResponse) -> Self {
        let c = r.current;
        let h = r.hourly;
        let d = r.daily;
        (
            crate::app::CurrentWeather {
                time: c.time,
                temperature: c.temperature_2m,
                apparent_temperature: c.apparent_temperature,
                humidity: c.relative_humidity_2m,
                wind_speed: c.wind_speed_10m,
                wind_direction: c.wind_direction_10m,
                wind_gusts: c.wind_gusts_10m,
                precipitation: c.precipitation,
                weather_code: c.weather_code,
                is_day: c.is_day != 0.0,
                pressure: c.pressure_msl,
                uv_index: c.uv_index,
                visibility: c.visibility,
                dewpoint: c.dewpoint_temperature,
            },
            crate::app::HourlyForecast {
                times: h.time,
                temperatures: h.temperature_2m,
                humidities: h.relative_humidity_2m,
                weather_codes: h.weather_code,
                precipitations: h.precipitation,
                wind_speeds: h.wind_speed_10m,
                is_day: h.is_day.iter().map(|v| *v != 0.0).collect(),
                pressures: h.pressure_msl,
            },
            crate::app::DailyForecast {
                dates: d.time,
                temp_high: d.temperature_2m_max,
                temp_low: d.temperature_2m_min,
                weather_codes: d.weather_code,
                sunrise: d.sunrise,
                sunset: d.sunset,
                precip_sum: d.precipitation_sum,
                wind_max: d.wind_speed_10m_max,
            },
        )
    }
}
```

- [ ] **Step 3: Verify compilation**

Run: `nix-shell --run "cargo check"`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/api/weather.rs
git commit -m "feat: wire new fields through conversion impls"
```

---

### Task 6: Add pressure trend helper

**Files:**
- Modify: `src/ui/helpers.rs`
- Test: `tests/unit_tests.rs`

- [ ] **Step 1: Add `format_pressure_trend` to helpers**

```rust
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
```

- [ ] **Step 2: Add tests for pressure trend**

```rust
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
```

- [ ] **Step 3: Add re-export to `src/lib.rs`**

Add to the `pub use ui::helpers::` line:

```rust
pub use ui::helpers::{format_wind_full, progress_bar, format_pressure_trend};
```

- [ ] **Step 4: Run tests**

Run: `nix-shell --run "cargo test"`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/ui/helpers.rs src/lib.rs tests/unit_tests.rs
git commit -m "feat: add pressure trend helper with tests"
```

---

### Task 7: Update current widget to display new fields

**Files:**
- Modify: `src/ui/widgets/current.rs`

- [ ] **Step 1: Add UV index formatting helper**

Add this private helper function at the top of `current.rs` (after imports):

```rust
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
```

- [ ] **Step 2: Add new display lines to the widget**

After the existing wind/gusts lines and before the `precip_line` push, add four new blocks. Replace the `lines` construction and the conditional precip push at the end of the render method:

```rust
        // Conditionally add precipitation line
        if let Some(line) = precip_line {
            lines.push(line);
        }

        // Pressure with trend
        if let Some(pressure) = current.pressure {
            let trend = if let Some(ref pressures) = hourly_pressures {
                // Find pressure ~3 hours ago by looking back 3 entries
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
```

- [ ] **Step 3: Pass hourly pressures into the widget**

The widget needs access to hourly pressure data for trend calculation. Change the render method to pull it from `self.app`:

At the top of the render method, after getting `current` and `location`, add:

```rust
        let hourly_pressures = self.app.hourly.as_ref().and_then(|h| h.pressures.as_ref());
```

Then use `hourly_pressures` in the pressure trend block above.

- [ ] **Step 4: Verify compilation**

Run: `nix-shell --run "cargo check"`
Expected: PASS

- [ ] **Step 5: Run all tests**

Run: `nix-shell --run "cargo test"`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/ui/widgets/current.rs
git commit -m "feat: display pressure, uv, visibility, dewpoint in current widget"
```

---

### Task 8: Update test fixtures with new fields

**Files:**
- Modify: `tests/unit_tests.rs`

- [ ] **Step 1: Update `deserialize_weather_response` fixture JSON**

Add the new fields to the `"current"` block in the test JSON:

```rust
"pressure_msl": 1013.2,
"uv_index": 5.2,
"visibility": 15000.0,
"dewpoint_temperature": 12.1
```

Add `pressure_msl` to the `"hourly"` block:

```rust
"pressure_msl": [1012.5, 1013.0, 1013.2, 1013.5]
```

- [ ] **Step 2: Add assertions for new fields**

After the existing current field assertions, add:

```rust
assert!((current.pressure.unwrap() - 1013.2).abs() < 1e-3);
assert!((current.uv_index.unwrap() - 5.2).abs() < 1e-3);
assert!((current.visibility.unwrap() - 15000.0).abs() < 1e-3);
assert!((current.dewpoint.unwrap() - 12.1).abs() < 1e-3);
```

After the existing hourly assertions, add:

```rust
let pressures = hourly.pressures.as_ref().unwrap();
assert_eq!(pressures.len(), 4);
assert!((pressures[0] - 1012.5).abs() < 1e-3);
```

- [ ] **Step 3: Update `deserialize_weather_response_is_day_as_number` fixture**

Add the new current fields (as `null` to test Option deserialization):

```rust
"pressure_msl": null,
"uv_index": null,
"visibility": null,
"dewpoint_temperature": null
```

Add hourly pressure (as `null`):

```rust
"pressure_msl": null
```

- [ ] **Step 4: Update `deserialize_runtime_weather_response_with_options` fixture**

Add the new fields (matching the Option pattern):

```rust
"pressure_msl": 1015.0,
"uv_index": 3.0,
"visibility": 20000.0,
"dewpoint_temperature": 10.5
```

And hourly:

```rust
"pressure_msl": [1015.0]
```

- [ ] **Step 5: Run all tests**

Run: `nix-shell --run "cargo test"`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add tests/unit_tests.rs
git commit -m "test: update fixtures for pressure, uv, visibility, dewpoint"
```

---

### Task 9: Verify and lint

- [ ] **Step 1: Run full test suite**

Run: `nix-shell --run "cargo test"`
Expected: All tests PASS

- [ ] **Step 2: Run clippy**

Run: `nix-shell --run "cargo clippy -- -D warnings"`
Expected: No warnings

- [ ] **Step 3: Run cargo check**

Run: `nix-shell --run "cargo check"`
Expected: PASS
