use anyhow::Result;
use serde::Deserialize;
use crate::api::client;
use crate::app::{CurrentWeather, HourlyForecast, DailyForecast};

// ============================================================
// Internal Option-wrapped structs — used only for fetch
// ============================================================

/// Option-wrapped response for parsing API output where fields may be missing.
#[derive(Debug, Deserialize)]
struct FetchWeatherResponse {
    pub current: Option<FetchCurrentData>,
    pub hourly: Option<FetchHourlyData>,
    pub daily: Option<FetchDailyData>,
}

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

#[derive(Debug, Deserialize)]
struct FetchDailyData {
    pub time: Vec<String>,
    pub temperature_2m_max: Vec<f32>,
    pub temperature_2m_min: Vec<f32>,
    pub weather_code: Vec<u8>,
    pub sunrise: Vec<String>,
    pub sunset: Vec<String>,
    pub precipitation_sum: Vec<f32>,
    pub wind_speed_10m_max: Vec<f32>,
}

// ============================================================
// Non-Option test structs — public, used by tests and integration
// ============================================================

/// Non-Option weather response used for testing deserialization.
/// Fields are guaranteed to be present (unlike the Option<> fields in the fetch response).
#[derive(serde::Deserialize, Debug, Clone)]
pub struct TestWeatherResponse {
    pub current: TestCurrentData,
    pub hourly: TestHourlyData,
    pub daily: TestDailyData,
}

/// Re-exported as WeatherResponse for tests
#[allow(dead_code)]
pub type WeatherResponse = TestWeatherResponse;

/// Non-Option data structs for test deserialization
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
}

#[allow(dead_code)]
pub type CurrentData = TestCurrentData;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct TestHourlyData {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f32>,
    pub relative_humidity_2m: Vec<u8>,
    pub weather_code: Vec<u8>,
    pub precipitation: Vec<f32>,
    pub wind_speed_10m: Vec<f32>,
    pub is_day: Vec<f64>,
}

#[allow(dead_code)]
pub type HourlyData = TestHourlyData;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct TestDailyData {
    pub time: Vec<String>,
    pub temperature_2m_max: Vec<f32>,
    pub temperature_2m_min: Vec<f32>,
    pub weather_code: Vec<u8>,
    pub sunrise: Vec<String>,
    pub sunset: Vec<String>,
    pub precipitation_sum: Vec<f32>,
    pub wind_speed_10m_max: Vec<f32>,
}

#[allow(dead_code)]
pub type DailyData = TestDailyData;

// ============================================================
// Conversion impls
// ============================================================

/// Convert an Option-wrapped fetch response into domain models.
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
        },
        HourlyForecast {
            times: h.time,
            temperatures: h.temperature_2m,
            humidities: h.relative_humidity_2m,
            weather_codes: h.weather_code,
            precipitations: h.precipitation,
            wind_speeds: h.wind_speed_10m,
            is_day: h.is_day.iter().map(|v| *v != 0.0).collect(),
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
            },
            crate::app::HourlyForecast {
                times: h.time,
                temperatures: h.temperature_2m,
                humidities: h.relative_humidity_2m,
                weather_codes: h.weather_code,
                precipitations: h.precipitation,
                wind_speeds: h.wind_speed_10m,
                is_day: h.is_day.iter().map(|v| *v != 0.0).collect(),
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

pub async fn fetch(lat: f64, lon: f64) -> Result<(CurrentWeather, HourlyForecast, DailyForecast)> {
    let url = client::forecast_url(lat, lon);
    let resp: FetchWeatherResponse = client::HTTP_CLIENT
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    extract_models(resp)
}
