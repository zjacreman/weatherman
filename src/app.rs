use serde::Deserialize;
use std::time::Duration;

// == WMO Weather Code ==

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WmoWeather {
    ClearSky,
    MainlyClear,
    PartlyCloudy,
    Overcast,
    Fog,
    DepositingRimeFog,
    LightDrizzle,
    ModerateDrizzle,
    DenseDrizzle,
    FreezingDrizzle,
    DenseFreezingDrizzle,
    SlightRain,
    ModerateRain,
    HeavyRain,
    FreezingRain,
    SlightSnow,
    ModerateSnow,
    HeavySnow,
    SnowGrains,
    RainShowers,
    SnowShowers,
    Thunderstorm,
    ThunderstormWithHail,
    Unknown(u8),
}

impl WmoWeather {
    pub fn from(code: u8) -> Self {
        match code {
            0 => WmoWeather::ClearSky,
            1 => WmoWeather::MainlyClear,
            2 => WmoWeather::PartlyCloudy,
            3 => WmoWeather::Overcast,
            45 => WmoWeather::Fog,
            48 => WmoWeather::DepositingRimeFog,
            51 => WmoWeather::LightDrizzle,
            53 => WmoWeather::ModerateDrizzle,
            55 => WmoWeather::DenseDrizzle,
            56 | 57 => WmoWeather::FreezingDrizzle,
            58 => WmoWeather::DenseFreezingDrizzle,
            61 => WmoWeather::SlightRain,
            63 => WmoWeather::ModerateRain,
            65 => WmoWeather::HeavyRain,
            66 | 67 => WmoWeather::FreezingRain,
            71 => WmoWeather::SlightSnow,
            73 => WmoWeather::ModerateSnow,
            75 => WmoWeather::HeavySnow,
            77 => WmoWeather::SnowGrains,
            80..=82 => WmoWeather::RainShowers,
            85..=87 => WmoWeather::SnowShowers,
            95 => WmoWeather::Thunderstorm,
            96 | 99 => WmoWeather::ThunderstormWithHail,
            _ => WmoWeather::Unknown(code),
        }
    }

    pub fn description(&self) -> String {
        match self {
            WmoWeather::ClearSky => "Clear sky".to_string(),
            WmoWeather::MainlyClear => "Mainly clear".to_string(),
            WmoWeather::PartlyCloudy => "Partly cloudy".to_string(),
            WmoWeather::Overcast => "Overcast".to_string(),
            WmoWeather::Fog => "Fog".to_string(),
            WmoWeather::DepositingRimeFog => "Depositing rime fog".to_string(),
            WmoWeather::LightDrizzle => "Light drizzle".to_string(),
            WmoWeather::ModerateDrizzle => "Moderate drizzle".to_string(),
            WmoWeather::DenseDrizzle => "Dense drizzle".to_string(),
            WmoWeather::FreezingDrizzle => "Freezing drizzle".to_string(),
            WmoWeather::DenseFreezingDrizzle => "Dense freezing drizzle".to_string(),
            WmoWeather::SlightRain => "Slight rain".to_string(),
            WmoWeather::ModerateRain => "Moderate rain".to_string(),
            WmoWeather::HeavyRain => "Heavy rain".to_string(),
            WmoWeather::FreezingRain => "Freezing rain".to_string(),
            WmoWeather::SlightSnow => "Light snowfall".to_string(),
            WmoWeather::ModerateSnow => "Moderate snowfall".to_string(),
            WmoWeather::HeavySnow => "Heavy snowfall".to_string(),
            WmoWeather::SnowGrains => "Snow grains".to_string(),
            WmoWeather::RainShowers => "Rain showers".to_string(),
            WmoWeather::SnowShowers => "Snow showers".to_string(),
            WmoWeather::Thunderstorm => "Thunderstorm".to_string(),
            WmoWeather::ThunderstormWithHail => "Thunderstorm with hail".to_string(),
            WmoWeather::Unknown(c) => format!("Unknown ({c})"),
        }
    }

    pub fn icon(&self, is_day: bool) -> &str {
        match self {
            WmoWeather::ClearSky => {
                if is_day {
                    "\u{1F31E}"
                } else {
                    "\u{1F319}"
                }
            }
            WmoWeather::MainlyClear => {
                if is_day {
                    "\u{1F32E}"
                } else {
                    "\u{1F319}"
                }
            }
            WmoWeather::PartlyCloudy => "\u{26C5}",
            WmoWeather::Overcast => "\u{2601}",
            WmoWeather::Fog => "\u{1F32B}",
            WmoWeather::DepositingRimeFog => "\u{1F32B}",
            WmoWeather::LightDrizzle => "\u{1F328}",
            WmoWeather::ModerateDrizzle => "\u{1F327}",
            WmoWeather::DenseDrizzle => "\u{1F327}",
            WmoWeather::FreezingDrizzle => "\u{1F328}",
            WmoWeather::DenseFreezingDrizzle => "\u{1F328}",
            WmoWeather::SlightRain => "\u{1F327}",
            WmoWeather::ModerateRain => "\u{1F327}",
            WmoWeather::HeavyRain => "\u{1F327}",
            WmoWeather::FreezingRain => "\u{1F32B}",
            WmoWeather::SlightSnow => "\u{2744}",
            WmoWeather::ModerateSnow => "\u{2744}",
            WmoWeather::HeavySnow => "\u{2744}",
            WmoWeather::SnowGrains => "\u{2744}",
            WmoWeather::RainShowers => "\u{1F327}",
            WmoWeather::SnowShowers => "\u{2744}",
            WmoWeather::Thunderstorm => "\u{26C8}",
            WmoWeather::ThunderstormWithHail => "\u{26C8}",
            WmoWeather::Unknown(_) => "\u{2753}",
        }
    }

    pub fn color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            WmoWeather::ClearSky => Color::Rgb(255, 193, 7),
            WmoWeather::MainlyClear => Color::Rgb(255, 235, 59),
            WmoWeather::PartlyCloudy | WmoWeather::Overcast => Color::Rgb(200, 220, 255),
            WmoWeather::Fog | WmoWeather::DepositingRimeFog => Color::Rgb(150, 150, 160),
            WmoWeather::LightDrizzle | WmoWeather::ModerateDrizzle | WmoWeather::DenseDrizzle => {
                Color::Rgb(100, 149, 237)
            }
            WmoWeather::FreezingDrizzle
            | WmoWeather::DenseFreezingDrizzle
            | WmoWeather::FreezingRain => Color::Rgb(135, 206, 235),
            WmoWeather::SlightRain | WmoWeather::ModerateRain | WmoWeather::HeavyRain => {
                Color::Rgb(70, 130, 180)
            }
            WmoWeather::SlightSnow
            | WmoWeather::ModerateSnow
            | WmoWeather::HeavySnow
            | WmoWeather::SnowGrains
            | WmoWeather::SnowShowers => Color::Rgb(240, 248, 255),
            WmoWeather::RainShowers => Color::Rgb(100, 149, 237),
            WmoWeather::Thunderstorm | WmoWeather::ThunderstormWithHail => {
                Color::Rgb(255, 105, 180)
            }
            WmoWeather::Unknown(_) => Color::Gray,
        }
    }
}

// == Data Models ==

#[derive(Debug, Clone, Deserialize)]
pub struct Location {
    pub id: u64,
    pub name: String,
    pub admin1: Option<String>,
    pub country: String,
    pub country_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub population: Option<u64>,
}

impl Location {
    pub fn display_name(&self) -> String {
        if let Some(ref admin) = self.admin1 {
            format!("{}, {}, {}", self.name, admin, self.country)
        } else {
            format!("{}, {}", self.name, self.country)
        }
    }
}

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
}

#[derive(Debug, Clone)]
pub struct HourlyForecast {
    pub times: Vec<String>,
    pub temperatures: Vec<f32>,
    pub humidities: Vec<u8>,
    pub weather_codes: Vec<u8>,
    pub precipitations: Vec<f32>,
    pub wind_speeds: Vec<f32>,
    pub is_day: Vec<bool>,
}

#[derive(Debug, Clone)]
pub struct DailyForecast {
    pub dates: Vec<String>,
    pub temp_high: Vec<f32>,
    pub temp_low: Vec<f32>,
    pub weather_codes: Vec<u8>,
    pub sunrise: Vec<String>,
    pub sunset: Vec<String>,
    pub precip_sum: Vec<f32>,
    pub wind_max: Vec<f32>,
}

// == App State ==

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Idle,
    LoadingWeather,
    LoadingSearch,
    Refreshing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TempUnit {
    Celsius,
    Fahrenheit,
}

impl TempUnit {
    pub fn toggle(&mut self) {
        *self = match self {
            TempUnit::Celsius => TempUnit::Fahrenheit,
            TempUnit::Fahrenheit => TempUnit::Celsius,
        };
    }
    pub fn symbol(&self) -> &str {
        match self {
            TempUnit::Celsius => "C",
            TempUnit::Fahrenheit => "F",
        }
    }
}

pub use crossterm::event::KeyEvent;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    SearchInput(String),
    SearchSubmit,
    SearchClear,
    SearchResultsReceived(Vec<Location>),
    SearchError(String),
    WeatherFetched,
    WeatherError(String),
    ToggleUnit,
    Key(KeyEvent),
    SearchModal { active: bool },
}

// == Main App ==

pub struct App {
    pub state: AppState,
    pub location: Option<Location>,
    pub current: Option<CurrentWeather>,
    pub hourly: Option<HourlyForecast>,
    pub daily: Option<DailyForecast>,
    pub search_query: String,
    pub search_results: Vec<Location>,
    pub search_modal_active: bool,
    pub search_focused: bool,
    pub search_selected_idx: u16,
    pub error_message: Option<String>,
    pub error_modal_visible: bool,
    pub last_update: Option<String>,
    pub temperature_unit: TempUnit,
    pub active_tab: u16,
    pub is_quit: bool,
    pub auto_refresh_interval: Duration,
    pub tick_count: u64,
    pub pending_auto_search: Option<String>,
    pub last_config_path: Option<std::path::PathBuf>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            state: AppState::Idle,
            location: None,
            current: None,
            hourly: None,
            daily: None,
            search_query: String::new(),
            search_results: Vec::new(),
            search_modal_active: false,
            search_focused: false,
            search_selected_idx: 0,
            error_message: None,
            error_modal_visible: false,
            last_update: None,
            temperature_unit: TempUnit::Celsius,
            active_tab: 0,
            is_quit: false,
            auto_refresh_interval: Duration::from_secs(7200),
            tick_count: 0,
            pending_auto_search: None,
            last_config_path: None,
        };

        // Load any previously saved location.
        if let Some((config, path)) = crate::config::load_config() {
            app.pending_auto_search = Some(config.name);
            app.auto_refresh_interval = Duration::from_secs(config.refresh_interval.unwrap_or(7200));
            app.last_config_path = Some(path);
        }

        app
    }

    pub fn is_quitting(&self) -> bool {
        self.is_quit
    }

    pub fn convert_temp(&self, temp_c: f32) -> f32 {
        match self.temperature_unit {
            TempUnit::Celsius => temp_c,
            TempUnit::Fahrenheit => temp_c * 9.0 / 5.0 + 32.0,
        }
    }

    pub fn format_temp(&self, value: f32) -> String {
        crate::ui::helpers::format_temp(value, &self.temperature_unit)
    }

    pub fn convert_wind_speed_kmh(&self, speed_kmh: f32) -> f32 {
        // kmh = mph * 1.60934
        // mph = kmh / 1.60934
        speed_kmh / 1.60934
    }

    pub fn format_wind_speed(&self, speed_kmh: f32) -> String {
        let speed = self.convert_wind_speed_kmh(speed_kmh);
        match self.temperature_unit {
            TempUnit::Celsius => format!("{speed:.0} km/h"),
            TempUnit::Fahrenheit => format!("{speed:.0} mph"),
        }
    }

    pub fn format_wind_direction(&self, degrees: u16) -> String {
        crate::ui::helpers::format_wind_full(degrees)
    }

    pub fn progress_bar(&self, current: u16, max: u16, width: u16) -> String {
        crate::ui::helpers::progress_bar(current, max, width)
    }

    pub fn show_error(&mut self, msg: String) {
        self.error_message = Some(msg);
        self.error_modal_visible = true;
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::SearchInput(s) => {
                self.search_query.clear();
                self.search_query.push_str(&s);
            }
            Message::SearchSubmit => {}
            Message::SearchClear => {
                self.search_query.clear();
                self.search_results.clear();
                self.search_modal_active = false;
                self.search_focused = false;
                self.search_selected_idx = 0;
                self.state = AppState::Idle;
            }
            Message::SearchResultsReceived(results) => {
                self.search_results = results;
                if self.search_results.is_empty() && self.search_query.len() >= 2 {
                    self.error_message = Some("No locations found".to_string());
                } else {
                    self.error_message = None;
                }
            }
            Message::SearchError(e) => {
                self.error_message = Some(e);
                self.error_modal_visible = true;
                self.search_results.clear();
            }
            Message::WeatherFetched => {
                self.state = AppState::Idle;
                self.last_update = Some(chrono::Local::now().format("%H:%M:%S").to_string());
            }
            Message::WeatherError(e) => {
                self.state = AppState::Idle;
                self.error_message = Some(e);
                self.error_modal_visible = true;
            }
            Message::Tick => {
                self.tick_count += 1;
                if self.tick_count >= self.auto_refresh_interval.as_secs() {
                    self.tick_count = 0;
                    if self.location.is_some() {
                        self.state = AppState::Refreshing;
                    }
                }
            }
            Message::ToggleUnit => {
                self.temperature_unit.toggle();
            }
            Message::SearchModal { active: true } => {
                self.search_modal_active = true;
                self.search_focused = true;
                self.search_selected_idx = 0;
            }
            Message::SearchModal { active: false } => {
                self.search_modal_active = false;
                self.search_focused = false;
                self.search_selected_idx = 0;
            }
            Message::Key(_) => {}
        }
    }

    pub fn weather_display(&self) -> Option<(String, String)> {
        self.current.as_ref().map(|current| {
            let wmo = WmoWeather::from(current.weather_code);
            (wmo.icon(current.is_day).to_string(), wmo.description().to_string())
        })
    }
}
