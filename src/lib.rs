//! Weather TUI — Rust + ratatui terminal weather app consuming the Open-Meteo API.
//!
//! This library exposes the core domain types (WMO weather codes, data models,
//! app state, temperature unit, messages) and the formatting helpers so that
//! downstream code and integration tests can exercise the business logic.
//!
//! See `weather_api::*` for the deserializable response types and conversion helpers.

pub mod app;
pub mod config;
pub mod ui;

// Re-export key types from sub-modules for convenient downstream access.
pub use app::{
    App, AppState, CurrentWeather, DailyForecast, HourlyForecast, Location, Message, TempUnit,
    WmoWeather,
};
pub use config::SavedConfig;
pub use ratatui::style::Color;

// Re-export ui helpers for testing
pub use ui::helpers::{format_wind_full, progress_bar};

// Re-export CurrentWidget so crate consumers can use it
pub use ui::CurrentWidget;

// --- Weather-API deserialization / conversion types ---

mod weather_api {
    pub use super::app::CurrentWeather;
    pub use super::app::DailyForecast;
    pub use super::app::HourlyForecast;

    /// Deserializable weather payload matching the Open-Meteo forecast API.
    #[derive(serde::Deserialize, Debug, Clone)]
    pub struct WeatherResponse {
        pub current: CurrentData,
        pub hourly: HourlyData,
        pub daily: DailyData,
    }

    #[derive(serde::Deserialize, Debug, Clone)]
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
    }

    #[derive(serde::Deserialize, Debug, Clone)]
    pub struct HourlyData {
        pub time: Vec<String>,
        pub temperature_2m: Vec<f32>,
        pub relative_humidity_2m: Vec<u8>,
        pub weather_code: Vec<u8>,
        pub precipitation: Vec<f32>,
        pub wind_speed_10m: Vec<f32>,
        pub is_day: Vec<f64>,
    }

    #[derive(serde::Deserialize, Debug, Clone)]
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

    /// A geocoding result from Open-Meteo's search endpoint.
    #[derive(serde::Deserialize, Debug, Clone)]
    pub struct GeocodingResult {
        pub id: u64,
        pub name: String,
        pub admin1: Option<String>,
        pub country: String,
        pub country_code: String,
        pub latitude: f64,
        pub longitude: f64,
        pub elevation: Option<f64>,
        pub timezone: String,
        pub population: Option<u64>,
    }

    /// Convert a deserialized geocoding result into the domain Location model.
    impl From<GeocodingResult> for super::Location {
        fn from(r: GeocodingResult) -> Self {
            super::Location {
                id: r.id,
                name: r.name,
                admin1: r.admin1,
                country: r.country,
                country_code: r.country_code,
                latitude: r.latitude,
                longitude: r.longitude,
                timezone: r.timezone,
                population: r.population,
            }
        }
    }

    /// Convert the full weather response into the three domain models:
    /// (CurrentWeather, HourlyForecast, DailyForecast).
    impl From<WeatherResponse> for (CurrentWeather, HourlyForecast, DailyForecast) {
        fn from(r: WeatherResponse) -> Self {
            let c = r.current;
            let h = r.hourly;
            let d = r.daily;

            (
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
            )
        }
    }
}

// Expose the WeatherResponse struct and geocoding type for integration tests.
pub use weather_api::{CurrentData, DailyData, GeocodingResult, HourlyData, WeatherResponse};
