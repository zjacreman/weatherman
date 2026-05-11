use anyhow::Result;
use serde::Deserialize;
use crate::api::client;
use crate::app::{CurrentWeather, HourlyForecast, DailyForecast};

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    pub current: Option<CurrentData>,
    pub hourly: Option<HourlyData>,
    pub daily: Option<DailyData>,
}

#[derive(Debug, Deserialize)]
struct CurrentData {
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

#[derive(Debug, Deserialize)]
struct HourlyData {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f32>,
    pub relative_humidity_2m: Vec<u8>,
    pub weather_code: Vec<u8>,
    pub precipitation: Vec<f32>,
    pub wind_speed_10m: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct DailyData {
    pub time: Vec<String>,
    pub temperature_2m_max: Vec<f32>,
    pub temperature_2m_min: Vec<f32>,
    pub weather_code: Vec<u8>,
    pub sunrise: Vec<String>,
    pub sunset: Vec<String>,
    pub precipitation_sum: Vec<f32>,
    pub wind_speed_10m_max: Vec<f32>,
}

impl From<WeatherResponse> for (CurrentWeather, HourlyForecast, DailyForecast) {
    fn from(r: WeatherResponse) -> Self {
        let c = r.current.expect("Missing current data");
        let h = r.hourly.expect("Missing hourly data");
        let d = r.daily.expect("Missing daily data");

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

pub async fn fetch(lat: f64, lon: f64) -> Result<(CurrentWeather, HourlyForecast, DailyForecast)> {
    let url = client::forecast_url(lat, lon);
    let resp: WeatherResponse = client::HTTP_CLIENT
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(resp.into())
}
