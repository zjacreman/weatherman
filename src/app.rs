use crossterm::event::{KeyCode, KeyModifiers};
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
                    "\u{1F324}\u{FE0F}"
                } else {
                    "\u{1F31A}"
                }
            }
            WmoWeather::PartlyCloudy => "\u{26C5}",
            WmoWeather::Overcast => "\u{2601}\u{FE0F}",
            WmoWeather::Fog => "\u{1F32B}\u{FE0F}",
            WmoWeather::DepositingRimeFog => "\u{1F32B}\u{FE0F}",
            WmoWeather::LightDrizzle => "\u{1F327}\u{FE0F}",
            WmoWeather::ModerateDrizzle => "\u{1F327}\u{FE0F}",
            WmoWeather::DenseDrizzle => "\u{1F327}\u{FE0F}",
            WmoWeather::FreezingDrizzle => "\u{1F9CA}",
            WmoWeather::DenseFreezingDrizzle => "\u{1F9CA}",
            WmoWeather::SlightRain => "\u{1F327}\u{FE0F}",
            WmoWeather::ModerateRain => "\u{1F327}\u{FE0F}",
            WmoWeather::HeavyRain => "\u{1F327}\u{FE0F}",
            WmoWeather::FreezingRain => "\u{1F9CA}",
            WmoWeather::SlightSnow => "\u{2744}\u{FE0F}",
            WmoWeather::ModerateSnow => "\u{1F328}\u{FE0F}",
            WmoWeather::HeavySnow => "\u{1F328}\u{FE0F}",
            WmoWeather::SnowGrains => "\u{1F328}\u{FE0F}",
            WmoWeather::RainShowers => "\u{1F327}\u{FE0F}",
            WmoWeather::SnowShowers => "\u{2744}\u{FE0F}",
            WmoWeather::Thunderstorm => "\u{26C8}\u{FE0F}",
            WmoWeather::ThunderstormWithHail => "\u{26C8}\u{FE0F}",
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
    #[allow(dead_code)]
    pub id: u64,
    pub name: String,
    pub admin1: Option<String>,
    pub country: String,
    #[allow(dead_code)]
    pub country_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    #[allow(dead_code)]
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
    pub pressure: Option<f32>,
    pub uv_index: Option<f32>,
    pub visibility: Option<f32>,
    pub dewpoint: Option<f32>,
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
    pub pressures: Option<Vec<f32>>,
}

impl HourlyForecast {
    /// Find the hourly index whose timestamp is closest to (and not after) the
    /// given current observation time string in `YYYY-MM-DDTHH:MM` form.
    /// Falls back to the last index if none match.
    pub fn pressure_index_for(&self, current_time: &str) -> Option<usize> {
        let pressures = self.pressures.as_ref()?;
        // current.time is "YYYY-MM-DDTHH:MM"; hourly time has the same prefix.
        // Find the last hourly slot whose time <= current_time.
        let mut found: Option<usize> = None;
        for (i, t) in self.times.iter().enumerate() {
            // Compare lexicographically — ISO format sorts chronologically.
            if t.as_str() <= current_time {
                found = Some(i);
            } else {
                break;
            }
        }
        found.or(Some(pressures.len().saturating_sub(1)))
    }

    /// Compare the pressure ~3 hours before `current_time` against the
    /// current pressure, returning (trend, arrow). Returns None if there
    /// isn't enough history.
    pub fn pressure_trend(&self, current_pressure: f32, current_time: &str) -> Option<(&'static str, &'static str)> {
        let idx = self.pressure_index_for(current_time)?;
        let pressures = self.pressures.as_ref()?;
        // Each hourly slot is 1 hour, so 3 slots earlier ~ 3h ago.
        if idx >= 3 {
            let prev = pressures[idx - 3];
            Some(crate::ui::helpers::format_pressure_trend(current_pressure, prev))
        } else {
            None
        }
    }
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

#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    Network(String),
    Geocoding(String),
    Api(String),
    Config(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Network(msg) => write!(f, "Network error: {msg}"),
            AppError::Geocoding(msg) => write!(f, "Geocoding error: {msg}"),
            AppError::Api(msg) => write!(f, "API error: {msg}"),
            AppError::Config(msg) => write!(f, "Config error: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

pub use crossterm::event::KeyEvent;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    #[allow(dead_code)]
    SearchInput(String),
    #[allow(dead_code)]
    SearchSubmit,
    SearchClear,
    SearchResultsReceived(Vec<Location>),
    SearchError(String),
    /// Weather data delivered by a spawned async task. Carries the three
    /// deserialized models so the event loop can store them on `App`. The
    /// hourly/daily vectors are boxed to keep the enum small (clippy:
    /// large_enum_variant) since they can be hundreds of entries each.
    WeatherFetchedBoxed(CurrentWeather, Box<HourlyForecast>, Box<DailyForecast>),
    WeatherError(String),
    /// Result of the startup auto-search for the persisted location name.
    AutoSearchResult { name: String, results: Vec<Location> },
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
    /// True while a geocoding search task is in flight — prevents re-spawning
    /// every loop iteration while `state == LoadingSearch`.
    pub search_pending: bool,
    /// True while a weather fetch is in flight — same purpose.
    pub weather_pending: bool,
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
            search_pending: false,
            weather_pending: false,
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

    #[allow(dead_code)]
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
        match self.temperature_unit {
            TempUnit::Celsius => speed_kmh,
            TempUnit::Fahrenheit => speed_kmh / 1.60934,
        }
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

    #[allow(dead_code)]
    pub fn show_error(&mut self, msg: String) {
        self.error_message = Some(msg);
        self.error_modal_visible = true;
    }

    /// Dispatch a keyboard event. Returns `true` if the key was consumed.
    ///
    /// Moved out of the binary crate so it is reachable from integration tests.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Handle search modal keys
        if self.search_modal_active {
            match key.code {
                KeyCode::Esc => {
                    self.update(Message::SearchModal { active: false });
                    self.update(Message::SearchClear);
                    return false;
                }
                KeyCode::Enter => {
                    if !self.search_results.is_empty()
                        && (self.search_selected_idx as usize) < self.search_results.len()
                    {
                        let selected = self.search_results[self.search_selected_idx as usize].clone();
                        self.search_modal_active = false;
                        self.search_query.clear();
                        self.search_results.clear();
                        self.search_selected_idx = 0;
                        self.location = Some(selected);
                        self.state = AppState::LoadingWeather;
                        return false;
                    }
                    if !self.search_query.is_empty() {
                        self.state = AppState::LoadingSearch;
                        return false;
                    }
                    return false;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.search_selected_idx > 0 {
                        self.search_selected_idx -= 1;
                    }
                    return false;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.search_selected_idx + 1 < self.search_results.len() as u16 {
                        self.search_selected_idx += 1;
                    }
                    return false;
                }
                KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
                    self.search_query.clear();
                    return false;
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    return false;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    return false;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                self.is_quit = true;
                true
            }
            KeyCode::Char('q') => {
                self.is_quit = true;
                true
            }
            KeyCode::Esc => {
                if self.search_modal_active {
                    self.update(Message::SearchModal { active: false });
                    self.update(Message::SearchClear);
                    true
                } else if self.error_modal_visible {
                    self.error_modal_visible = false;
                    self.error_message = None;
                    true
                } else if self.error_message.is_some() {
                    self.error_message = None;
                    true
                } else {
                    false
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.search_query.clear();
                self.search_results.clear();
                self.search_selected_idx = 0;
                self.state = AppState::Idle;
                self.update(Message::SearchModal { active: true });
                true
            }
            KeyCode::Char('u') | KeyCode::Char('U') => {
                self.update(Message::ToggleUnit);
                true
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                if self.location.is_some() {
                    self.state = AppState::Refreshing;
                }
                true
            }
            KeyCode::Tab => {
                self.active_tab = match self.active_tab {
                    0 => 1,
                    _ => 0,
                };
                true
            }
            _ => false,
        }
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
                self.search_selected_idx = 0;
                self.state = AppState::Idle;
            }
            Message::SearchResultsReceived(results) => {
                self.search_pending = false;
                self.search_results = results;
                self.search_modal_active = true;
                self.search_selected_idx = 0;
                self.state = AppState::Idle;
                if self.search_results.is_empty() && self.search_query.len() >= 2 {
                    self.error_message = Some("No locations found".to_string());
                } else {
                    self.error_message = None;
                }
            }
            Message::SearchError(e) => {
                self.search_pending = false;
                self.state = AppState::Idle;
                self.error_message = Some(e);
                self.error_modal_visible = true;
                self.search_results.clear();
                self.search_modal_active = false;
            }
            Message::AutoSearchResult { name, results } => {
                self.search_pending = false;
                if let Some(loc) = results.into_iter().next() {
                    self.location = Some(loc);
                    self.state = AppState::LoadingWeather;
                } else {
                    self.error_message = Some(format!("Location '{name}' not found"));
                    self.error_modal_visible = true;
                    self.state = AppState::Idle;
                }
            }
            Message::WeatherFetchedBoxed(current, hourly, daily) => {
                self.weather_pending = false;
                self.current = Some(current);
                self.hourly = Some(*hourly);
                self.daily = Some(*daily);
                self.state = AppState::Idle;
                self.last_update = Some(chrono::Local::now().format("%H:%M:%S").to_string());
                self.error_message = None;
                self.error_modal_visible = false;
            }
            Message::WeatherError(e) => {
                self.weather_pending = false;
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
                self.search_selected_idx = 0;
            }
            Message::SearchModal { active: false } => {
                self.search_modal_active = false;
                self.search_selected_idx = 0;
            }
            Message::Key(key) => {
                let _ = self.handle_key(key);
            }
        }
    }

    #[allow(dead_code)]
    pub fn weather_display(&self) -> Option<(String, String)> {
        self.current.as_ref().map(|current| {
            let wmo = WmoWeather::from(current.weather_code);
            (wmo.icon(current.is_day).to_string(), wmo.description().to_string())
        })
    }
}
