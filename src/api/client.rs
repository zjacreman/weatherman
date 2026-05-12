use reqwest::Client;
use std::sync::LazyLock;

pub static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("weather-tui/0.1")
        .build()
        .expect("Failed to build HTTP client")
});

/// Encode query parameter values to be URL-safe
pub fn encode_query(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => vec![c],
            ' ' => "%20".chars().collect::<Vec<_>>(),
            c => {
                let bytes = c.to_string().into_bytes();
                bytes.into_iter()
                    .flat_map(|b| format!("%{:02X}", b).chars().collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            }
        })
        .collect::<String>()
}

#[inline]
pub fn geocoding_url(query: &str, count: u32) -> String {
    let encoded = encode_query(query);
    format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count={}&language=en",
        encoded, count
    )
}

#[inline]
pub fn forecast_url(lat: f64, lon: f64) -> String {
    format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat:.6}&longitude={lon:.6}&current=temperature_2m,relative_humidity_2m,apparent_temperature,precipitation,weather_code,wind_speed_10m,wind_direction_10m,wind_gusts_10m,is_day&hourly=temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,precipitation,wind_speed_10m,is_day&daily=weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,precipitation_sum,wind_speed_10m_max&timezone=auto&temperature_unit=celsius",
    )
}
