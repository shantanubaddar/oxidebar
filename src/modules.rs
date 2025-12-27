use std::fs;

// Module definitions for the bar

pub struct BatteryModule {
    battery_path: String,
}

impl BatteryModule {
    pub fn new() -> Self {
        Self {
            battery_path: Self::find_battery_path(),
        }
    }

    fn find_battery_path() -> String {
        let power_supply_path = "/sys/class/power_supply";
        
        for entry in ["BAT0", "BAT1"] {
            let path = format!("{}/{}", power_supply_path, entry);
            if std::path::Path::new(&path).exists() {
                return path;
            }
        }
        
        format!("{}/BAT0", power_supply_path)
    }

    fn read_sysfs_u64(&self, filename: &str) -> u64 {
        let path = format!("{}/{}", self.battery_path, filename);
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    }

    fn read_sysfs_string(&self, filename: &str) -> String {
        let path = format!("{}/{}", self.battery_path, filename);
        fs::read_to_string(&path)
            .unwrap_or_else(|_| String::from("Unknown"))
            .trim()
            .to_string()
    }

    pub fn get_percentage(&self) -> f64 {
        let charge_now = self.read_sysfs_u64("charge_now");
        let charge_full = self.read_sysfs_u64("charge_full");
        
        if charge_full > 0 {
            (charge_now as f64 / charge_full as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn get_status(&self) -> String {
        self.read_sysfs_string("status")
    }

    pub fn get_icon(&self, percentage: f64, status: &str) -> &'static str {
        if status == "Charging" {
            return "CHG"; // Charging
        }
        
        // Battery level - simple text for now
        if percentage >= 80.0 {
            "BAT"
        } else if percentage >= 60.0 {
            "BAT"
        } else if percentage >= 40.0 {
            "BAT"
        } else if percentage >= 20.0 {
            "BAT"
        } else {
            "LOW"
        }
    }

    pub fn render(&self) -> String {
        let percentage = self.get_percentage();
        let status = self.get_status();
        let icon = self.get_icon(percentage, &status);
        
        format!("{} {:.0}%", icon, percentage)
    }
}

pub struct NetworkModule {
    // Network monitoring
}

impl NetworkModule {
    pub fn new() -> Self {
        Self {}
    }

    fn get_active_interface(&self) -> Option<String> {
        // Check common wireless interface names
        for iface in ["wlan0", "wlp3s0", "wlp2s0", "wlo1"] {
            let path = format!("/sys/class/net/{}/operstate", iface);
            if let Ok(state) = fs::read_to_string(&path) {
                if state.trim() == "up" {
                    return Some(iface.to_string());
                }
            }
        }
        
        // Check ethernet
        for iface in ["eth0", "enp3s0", "enp2s0", "eno1"] {
            let path = format!("/sys/class/net/{}/operstate", iface);
            if let Ok(state) = fs::read_to_string(&path) {
                if state.trim() == "up" {
                    return Some(iface.to_string());
                }
            }
        }
        
        None
    }

    fn is_wireless(&self, iface: &str) -> bool {
        let wireless_path = format!("/sys/class/net/{}/wireless", iface);
        std::path::Path::new(&wireless_path).exists()
    }

    pub fn render(&self) -> String {
        match self.get_active_interface() {
            Some(iface) => {
                if self.is_wireless(&iface) {
                    format!("WiFi {}", iface)
                } else {
                    format!("ETH {}", iface)
                }
            }
            None => String::from("NET Down"),
        }
    }
}
