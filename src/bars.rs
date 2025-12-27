use chrono::Local;
use sysinfo::System;

// Simple bar renderer that draws to a buffer
pub struct BarRenderer {
    system: System,
}

impl BarRenderer {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    pub fn render(&mut self, canvas: &mut [u8], width: u32, height: u32) {
        // Update system info
        self.system.refresh_all();

        // Clear background (dark gray)
        for pixel in canvas.chunks_exact_mut(4) {
            pixel[0] = 30;  // Blue
            pixel[1] = 30;  // Green
            pixel[2] = 30;  // Red
            pixel[3] = 255; // Alpha (ARGB format)
        }

        // Get status text
        let time = Local::now().format("%H:%M:%S").to_string();
        let cpu_avg = self.system.global_cpu_info().cpu_usage();
        let mem_used = self.system.used_memory() / 1024 / 1024;
        let mem_total = self.system.total_memory() / 1024 / 1024;
        
        let status = format!(
            "  {}  |  CPU: {:.1}%  |  RAM: {}MB / {}MB  ",
            time, cpu_avg, mem_used, mem_total
        );

        // For now, just draw colored rectangles as modules
        // In the next iteration, we'll add proper text rendering
        self.draw_modules(canvas, width, height, &status);
    }

    fn draw_modules(&self, canvas: &mut [u8], width: u32, height: u32, _status: &str) {
        // Draw some colored sections to show it's working
        let section_width = width / 4;
        
        // Time section (cyan)
        self.fill_rect(canvas, width, 10, 5, section_width - 20, height - 10, 100, 200, 200);
        
        // CPU section (green)
        self.fill_rect(canvas, width, section_width + 10, 5, section_width - 20, height - 10, 100, 200, 100);
        
        // RAM section (blue)
        self.fill_rect(canvas, width, section_width * 2 + 10, 5, section_width - 20, height - 10, 100, 100, 200);
        
        // Battery section (yellow)
        self.fill_rect(canvas, width, section_width * 3 + 10, 5, section_width - 20, height - 10, 200, 200, 100);
    }

    fn fill_rect(&self, canvas: &mut [u8], canvas_width: u32, x: u32, y: u32, w: u32, h: u32, r: u8, g: u8, b: u8) {
        for dy in 0..h {
            for dx in 0..w {
                let px = x + dx;
                let py = y + dy;
                if px < canvas_width && py < 30 {
                    let offset = ((py * canvas_width + px) * 4) as usize;
                    if offset + 3 < canvas.len() {
                        canvas[offset] = b;     // Blue
                        canvas[offset + 1] = g; // Green
                        canvas[offset + 2] = r; // Red
                        canvas[offset + 3] = 255; // Alpha
                    }
                }
            }
        }
    }
}
