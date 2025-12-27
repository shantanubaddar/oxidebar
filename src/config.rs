use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default = "default_height")]
    pub height: u32,
    
    #[serde(default)]
    pub style: Style,
    
    #[serde(default)]
    pub modules_left: Vec<String>,
    
    #[serde(default)]
    pub modules_center: Vec<String>,
    
    #[serde(default)]
    pub modules_right: Vec<String>,
    
    #[serde(default)]
    pub module_config: ModuleConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Style {
    #[serde(default = "default_bg_color")]
    pub background: String,
    
    #[serde(default = "default_fg_color")]
    pub foreground: String,
    
    #[serde(default = "default_accent_color")]
    pub accent: String,
    
    #[serde(default = "default_warning_color")]
    pub warning: String,
    
    #[serde(default = "default_critical_color")]
    pub critical: String,
    
    #[serde(default = "default_padding")]
    pub padding: u32,
    
    #[serde(default = "default_spacing")]
    pub module_spacing: u32,
    
    #[serde(default = "default_font_size")]
    pub font_size: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModuleConfig {
    #[serde(default)]
    pub workspaces: WorkspacesConfig,
    
    #[serde(default)]
    pub battery: BatteryConfig,
    
    #[serde(default)]
    pub network: NetworkConfig,
    
    #[serde(default)]
    pub clock: ClockConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkspacesConfig {
    #[serde(default = "default_ws_format")]
    pub format: String,
    
    #[serde(default = "default_true")]
    pub show_empty: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BatteryConfig {
    #[serde(default = "default_battery_format")]
    pub format: String,
    
    #[serde(default = "default_true")]
    pub show_icon: bool,
    
    #[serde(default = "default_battery_warning")]
    pub warning_threshold: u32,
    
    #[serde(default = "default_battery_critical")]
    pub critical_threshold: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NetworkConfig {
    #[serde(default = "default_network_format")]
    pub format: String,
    
    #[serde(default = "default_true")]
    pub show_icon: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClockConfig {
    #[serde(default = "default_clock_format")]
    pub format: String,
}

// Default functions
fn default_height() -> u32 { 30 }
fn default_bg_color() -> String { "#1e1e2e".to_string() }
fn default_fg_color() -> String { "#cdd6f4".to_string() }
fn default_accent_color() -> String { "#89b4fa".to_string() }
fn default_warning_color() -> String { "#f9e2af".to_string() }
fn default_critical_color() -> String { "#f38ba8".to_string() }
fn default_padding() -> u32 { 10 }
fn default_spacing() -> u32 { 15 }
fn default_font_size() -> u32 { 12 }

fn default_ws_format() -> String { "{idx}".to_string() }
fn default_battery_format() -> String { "{icon} {percentage}%".to_string() }
fn default_network_format() -> String { "{icon} {ifname}".to_string() }
fn default_clock_format() -> String { "%H:%M:%S".to_string() }

fn default_battery_warning() -> u32 { 30 }
fn default_battery_critical() -> u32 { 15 }
fn default_true() -> bool { true }

impl Default for Style {
    fn default() -> Self {
        Self {
            background: default_bg_color(),
            foreground: default_fg_color(),
            accent: default_accent_color(),
            warning: default_warning_color(),
            critical: default_critical_color(),
            padding: default_padding(),
            module_spacing: default_spacing(),
            font_size: default_font_size(),
        }
    }
}

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            format: default_ws_format(),
            show_empty: true,
        }
    }
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            format: default_battery_format(),
            show_icon: true,
            warning_threshold: default_battery_warning(),
            critical_threshold: default_battery_critical(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            format: default_network_format(),
            show_icon: true,
        }
    }
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: default_clock_format(),
        }
    }
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            workspaces: WorkspacesConfig::default(),
            battery: BatteryConfig::default(),
            network: NetworkConfig::default(),
            clock: ClockConfig::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            height: default_height(),
            style: Style::default(),
            modules_left: vec!["workspaces".to_string()],
            modules_center: vec![],
            modules_right: vec!["network".to_string(), "battery".to_string(), "clock".to_string()],
            module_config: ModuleConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        
        if let Ok(contents) = fs::read_to_string(&config_path) {
            match toml::from_str(&contents) {
                Ok(config) => {
                    eprintln!("Loaded config from: {}", config_path.display());
                    return config;
                }
                Err(e) => {
                    eprintln!("Failed to parse config: {}", e);
                    eprintln!("Using default config");
                }
            }
        } else {
            eprintln!("No config found at: {}", config_path.display());
            eprintln!("Creating default config...");
            
            let default = Self::default();
            if let Ok(toml_string) = toml::to_string_pretty(&default) {
                if let Some(parent) = config_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&config_path, toml_string);
                eprintln!("Created default config at: {}", config_path.display());
            }
            return default;
        }
        
        Self::default()
    }
    
    fn get_config_path() -> PathBuf {
        if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(config_home).join("oxidebar").join("config.toml")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("oxidebar").join("config.toml")
        } else {
            PathBuf::from("config.toml")
        }
    }
    
    pub fn parse_color(&self, color_str: &str) -> u32 {
        // Parse hex color like "#1e1e2e" to ARGB
        let color_str = color_str.trim_start_matches('#');
        
        if color_str.len() == 6 {
            // RGB format - add full opacity
            if let Ok(rgb) = u32::from_str_radix(color_str, 16) {
                return 0xFF000000 | rgb;
            }
        } else if color_str.len() == 8 {
            // ARGB format
            if let Ok(argb) = u32::from_str_radix(color_str, 16) {
                return argb;
            }
        }
        
        // Default to white on parse error
        0xFFFFFFFF
    }
}
