use serde::Deserialize;
use crate::api::client;
use crate::app::{AppError, CurrentWeather, HourlyForecast, DailyForecast};

// ============================================================
// Public deserialization structs — single source of truth
// ============================================================

/// Weather API response with optional top-level sections.
#[derive(Debug, Deserialize, Clone)]
pub struct WeatherResponse {
    pub current: Option<CurrentData>,
    pub hourly: Option<HourlyData>,
    pub daily: Option<DailyData>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CurrentData {
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
    #[serde(rename = "dew_point_2m")]
    pub dewpoint_temperature: Option<f32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HourlyData {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f32>,
    pub relative_humidity_2m: Vec<u8>,
    pub weather_code: Vec<u8>,
    pub precipitation: Vec<f32>,
    pub wind_speed_10m: Vec<f32>,
    pub is_day: Vec<f64>,
    pub pressure_msl: Option<Vec<f32>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DailyData {
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
// Conversion impl
// ============================================================

impl TryFrom<WeatherResponse> for (CurrentWeather, HourlyForecast, DailyForecast) {
    type Error = AppError;

    fn try_from(resp: WeatherResponse) -> Result<Self, Self::Error> {
        let c = resp.current.ok_or_else(|| AppError::Api("Missing current data".into()))?;
        let h = resp.hourly.ok_or_else(|| AppError::Api("Missing hourly data".into()))?;
        let d = resp.daily.ok_or_else(|| AppError::Api("Missing daily data".into()))?;

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
}

#[derive(Debug, Deserialize)]
struct ApiError {
    pub reason: Option<String>,
}

pub async fn fetch(lat: f64, lon: f64) -> Result<(CurrentWeather, HourlyForecast, DailyForecast), AppError> {
    let url = client::forecast_url(lat, lon);
    let resp = client::HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        if let Ok(api_err) = serde_json::from_str::<ApiError>(&body) {
            if let Some(reason) = api_err.reason {
                return Err(AppError::Api(reason));
            }
        }
        return Err(AppError::Api(format!("HTTP {} — {}", status.as_u16(), body)));
    }

    let parsed: WeatherResponse = resp
        .json()
        .await
        .map_err(|e| AppError::Api(format!("Failed to decode response: {e}")))?;
    let models = parsed.try_into()?;
    Ok(models)
}
