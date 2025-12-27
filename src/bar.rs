use crate::modules::{BatteryModule, NetworkModule};
use crate::niri::NiriIpc;
use crate::config::Config;
use chrono::Local;

pub struct BarRenderer {
    battery: BatteryModule,
    network: NetworkModule,
    niri: Option<NiriIpc>,
    config: Config,
}

impl BarRenderer {
    pub fn new(config: Config) -> Self {
        Self {
            battery: BatteryModule::new(),
            network: NetworkModule::new(),
            niri: NiriIpc::new(),
            config,
        }
    }

    pub fn render(&mut self, canvas: &mut [u8], width: u32, height: u32) {
        // Clear with configured background color
        let bg_color = self.config.parse_color(&self.config.style.background);
        
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&bg_color.to_ne_bytes());
        }

        // Render modules by position
        let mut left_x = self.config.style.padding as i32;
        let mut right_x = width as i32 - self.config.style.padding as i32;
        let y = ((height - 8) / 2) as i32; // Vertically center the text
        
        let fg_color = self.config.parse_color(&self.config.style.foreground);
        let accent_color = self.config.parse_color(&self.config.style.accent);
        
        // Render left modules
        for module_name in &self.config.modules_left {
            let text = self.get_module_text(module_name);
            let color = if module_name == "workspaces" { accent_color } else { fg_color };
            
            self.draw_module(canvas, width, height, &text, left_x, y, color);
            left_x += (text.len() as i32 * 6) + self.config.style.module_spacing as i32;
        }
        
        // Render center modules (centered on screen)
        if !self.config.modules_center.is_empty() {
            let center_text: Vec<String> = self.config.modules_center
                .iter()
                .map(|m| self.get_module_text(m))
                .collect();
            let total_width: i32 = center_text.iter()
                .map(|t| t.len() as i32 * 6)
                .sum::<i32>() + 
                (center_text.len() as i32 - 1) * self.config.style.module_spacing as i32;
            
            let mut center_x = (width as i32 - total_width) / 2;
            
            for (i, _module_name) in self.config.modules_center.iter().enumerate() {
                let text = &center_text[i];
                self.draw_module(canvas, width, height, text, center_x, y, fg_color);
                center_x += (text.len() as i32 * 6) + self.config.style.module_spacing as i32;
            }
        }
        
        // Render right modules (right-aligned)
        let mut right_modules_text = Vec::new();
        for module_name in self.config.modules_right.iter().rev() {
            right_modules_text.push(self.get_module_text(module_name));
        }
        
        for (i, module_name) in self.config.modules_right.iter().enumerate().rev() {
            let text = &right_modules_text[self.config.modules_right.len() - 1 - i];
            let text_width = text.len() as i32 * 6;
            
            // Choose color based on module and state
            let color = self.get_module_color(module_name, text);
            
            self.draw_module(canvas, width, height, text, right_x - text_width, y, color);
            right_x -= text_width + self.config.style.module_spacing as i32;
        }
    }
    
    fn get_module_text(&self, module_name: &str) -> String {
        match module_name {
            "workspaces" => {
                self.niri
                    .as_ref()
                    .map(|n| n.get_workspace_summary())
                    .unwrap_or_else(|| String::from("WS ?"))
            }
            "battery" => self.battery.render(),
            "network" => self.network.render(),
            "clock" => {
                Local::now()
                    .format(&self.config.module_config.clock.format)
                    .to_string()
            }
            _ => String::from("?"),
        }
    }
    
    fn get_module_color(&self, module_name: &str, text: &str) -> u32 {
        match module_name {
            "battery" => {
                // Parse battery percentage from text
                if let Some(pct_str) = text.split_whitespace().find(|s| s.ends_with('%')) {
                    if let Ok(pct) = pct_str.trim_end_matches('%').parse::<u32>() {
                        if pct <= self.config.module_config.battery.critical_threshold {
                            return self.config.parse_color(&self.config.style.critical);
                        } else if pct <= self.config.module_config.battery.warning_threshold {
                            return self.config.parse_color(&self.config.style.warning);
                        }
                    }
                }
                self.config.parse_color(&self.config.style.foreground)
            }
            "workspaces" => self.config.parse_color(&self.config.style.accent),
            _ => self.config.parse_color(&self.config.style.foreground),
        }
    }
    
    fn draw_module(
        &self,
        canvas: &mut [u8],
        width: u32,
        height: u32,
        text: &str,
        x: i32,
        y: i32,
        color: u32,
    ) {
        self.draw_simple_text(canvas, width, height, text, x, y, color);
    }

    fn draw_simple_text(
        &self,
        canvas: &mut [u8],
        width: u32,
        height: u32,
        text: &str,
        x: i32,
        y: i32,
        color: u32,
    ) {
        let char_width = 6;
        
        for (i, ch) in text.chars().enumerate() {
            let char_x = x + (i as i32 * char_width);
            self.draw_char(canvas, width, height, ch, char_x, y, color);
        }
    }
    
    fn draw_char(
        &self,
        canvas: &mut [u8],
        width: u32,
        height: u32,
        ch: char,
        x: i32,
        y: i32,
        color: u32,
    ) {
        // Simple 5x7 bitmap patterns for common characters
        let pattern = match ch {
            '0' => [0x7C, 0xC6, 0xCE, 0xD6, 0xE6, 0xC6, 0x7C],
            '1' => [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E],
            '2' => [0x7C, 0xC6, 0x06, 0x1C, 0x30, 0x60, 0xFE],
            '3' => [0x7C, 0xC6, 0x06, 0x3C, 0x06, 0xC6, 0x7C],
            '4' => [0x0C, 0x1C, 0x3C, 0x6C, 0xFE, 0x0C, 0x0C],
            '5' => [0xFE, 0xC0, 0xFC, 0x06, 0x06, 0xC6, 0x7C],
            '6' => [0x7C, 0xC0, 0xFC, 0xC6, 0xC6, 0xC6, 0x7C],
            '7' => [0xFE, 0x06, 0x0C, 0x18, 0x30, 0x30, 0x30],
            '8' => [0x7C, 0xC6, 0xC6, 0x7C, 0xC6, 0xC6, 0x7C],
            '9' => [0x7C, 0xC6, 0xC6, 0x7E, 0x06, 0x0C, 0x78],
            ':' => [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00],
            '%' => [0xC6, 0xC6, 0x0C, 0x18, 0x30, 0x63, 0x63],
            '[' => [0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C],
            ']' => [0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C],
            '|' => [0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
            ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            '?' => [0x7C, 0xC6, 0x0C, 0x18, 0x18, 0x00, 0x18],
            'A' | 'a' => [0x7C, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6],
            'B' | 'b' => [0xFC, 0xC6, 0xC6, 0xFC, 0xC6, 0xC6, 0xFC],
            'C' | 'c' => [0x7C, 0xC6, 0xC0, 0xC0, 0xC0, 0xC6, 0x7C],
            'D' | 'd' => [0xF8, 0xCC, 0xC6, 0xC6, 0xC6, 0xCC, 0xF8],
            'E' | 'e' => [0xFE, 0xC0, 0xC0, 0xF8, 0xC0, 0xC0, 0xFE],
            'F' | 'f' => [0xFE, 0xC0, 0xC0, 0xF8, 0xC0, 0xC0, 0xC0],
            'G' | 'g' => [0x7C, 0xC6, 0xC0, 0xCE, 0xC6, 0xC6, 0x7C],
            'H' | 'h' => [0xC6, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6],
            'I' => [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x7E],
            'i' => [0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x3C],
            'L' | 'l' => [0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xFE],
            'N' | 'n' => [0xC6, 0xE6, 0xF6, 0xDE, 0xCE, 0xC6, 0xC6],
            'O' | 'o' => [0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C],
            'P' | 'p' => [0xFC, 0xC6, 0xC6, 0xFC, 0xC0, 0xC0, 0xC0],
            'R' | 'r' => [0xFC, 0xC6, 0xC6, 0xFC, 0xCC, 0xC6, 0xC6],
            'S' | 's' => [0x7C, 0xC6, 0xC0, 0x7C, 0x06, 0xC6, 0x7C],
            'T' | 't' => [0xFE, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18],
            'W' | 'w' => [0xC6, 0xC6, 0xC6, 0xD6, 0xFE, 0xEE, 0xC6],
            _ => [0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE],
        };
        
        for (row, byte) in pattern.iter().enumerate() {
            for bit in 0..8 {
                if (byte & (1 << (7 - bit))) != 0 {
                    let px = x + bit;
                    let py = y + row as i32;
                    
                    if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                        let offset = ((py * width as i32 + px) * 4) as usize;
                        if offset + 3 < canvas.len() {
                            canvas[offset..offset + 4].copy_from_slice(&color.to_ne_bytes());
                        }
                    }
                }
            }
        }
    }
}
