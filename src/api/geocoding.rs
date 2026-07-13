use serde::Deserialize;
use crate::api::client;
use crate::app::{AppError, Location};

#[derive(Debug, Deserialize)]
pub struct GeocodingResponse {
    #[serde(rename = "results")]
    pub result: Vec<GeocodingResult>,
}

#[derive(Debug, Deserialize)]
pub struct GeocodingResult {
    pub id: u64,
    pub name: String,
    pub admin1: Option<String>,
    pub country: String,
    pub country_code: String,
    pub latitude: f64,
    pub longitude: f64,
    #[allow(dead_code)]
    pub elevation: Option<f64>,
    pub timezone: String,
    pub population: Option<u64>,
}

impl From<GeocodingResult> for Location {
    fn from(r: GeocodingResult) -> Self {
        Location {
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

pub async fn search(name: &str) -> Result<Vec<Location>, AppError> {
    let url = client::geocoding_url(name, 10);
    let resp: GeocodingResponse = client::HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| AppError::Geocoding(e.to_string()))?
        .json()
        .await
        .map_err(|e| AppError::Geocoding(format!("Failed to decode geocoding response: {e}")))?;
    Ok(resp.result.into_iter().map(Location::from).collect())
}
