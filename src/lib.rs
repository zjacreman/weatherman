//! Weather TUI — Rust + ratatui terminal weather app consuming the Open-Meteo API.
//!
//! This library exposes the core domain types (WMO weather codes, data models,
//! app state, temperature unit, messages) and the formatting helpers so that
//! downstream code and integration tests can exercise the business logic.
//!
//! See `api::weather::WeatherResponse` and `api::geocoding::GeocodingResult` for
//! the deserializable response types and conversion helpers.

pub mod api;
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

// Re-export weather test structs from api/weather.rs
pub use api::weather::{CurrentData, DailyData, HourlyData, TestWeatherResponse, WeatherResponse};

// Re-export geocoding types from api/geocoding.rs
pub use api::geocoding::GeocodingResult;
