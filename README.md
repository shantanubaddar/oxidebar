<img width="1319" height="768" alt="image" src="https://github.com/user-attachments/assets/cfd291d1-d34c-4492-9d56-cf18dd896e77" />
(Yes, it is super-ugly, but understand that it's in it's barebones form at the moment)


# oxidebar

A lightweight, efficient status bar for Wayland compositors, written in Rust.

## Why oxidebar?

- **Extremely low resource usage**: 30-34MB RAM (vs 40-46MB for waybar)
- **Minimal CPU overhead**: ≤0.1% CPU usage (vs 0.2-0.3% for waybar)  
- **Fast updates**: 200ms refresh rate for snappy workspace switching
- **Simple codebase**: Easy to understand and modify
- **Configurable**: TOML-based configuration similar to waybar

Built specifically for tiling Wayland compositors like [niri](https://github.com/YaLTeR/niri), with a focus on performance and simplicity.

## Features

- ✅ Niri workspace integration
- ✅ Battery status with color-coded warnings
- ✅ Network status  
- ✅ Clock with customizable format
- ✅ Configurable colors and layout
- ✅ Left/center/right module positioning (like waybar)

## Installation

### Quick install (recommended)

```bash
git clone https://github.com/yourusername/oxidebar
cd oxidebar
chmod +x install.sh
./install.sh
```

This will build and install oxidebar to `/usr/local/bin/`, making it available system-wide.

### Manual installation

```bash
git clone https://github.com/yourusername/oxidebar
cd oxidebar
cargo build --release
sudo cp target/release/oxidebar /usr/local/bin/
```

### Verify installation

```bash
which oxidebar
# Should output: /usr/local/bin/oxidebar

oxidebar --version
# Should output: oxidebar 0.1.0
```

### Dependencies

- Rust 1.70+
- Wayland compositor with layer-shell support
- For niri workspace support: niri compositor

## Configuration

oxidebar looks for its configuration at `~/.config/oxidebar/config.toml`. On first run, it will create a default configuration file.

### Example configuration

```toml
height = 30

modules_left = ["workspaces"]
modules_center = []
modules_right = ["network", "battery", "clock"]

[style]
background = "#1e1e2e"
foreground = "#cdd6f4"
accent = "#89b4fa"
warning = "#f9e2af"
critical = "#f38ba8"
padding = 10
module_spacing = 15
font_size = 12

[module_config.workspaces]
format = "{idx}"
show_empty = true

[module_config.battery]
format = "{icon} {percentage}%"
show_icon = true
warning_threshold = 30
critical_threshold = 15

[module_config.network]
format = "{icon} {ifname}"
show_icon = true

[module_config.clock]
format = "%H:%M:%S"
```

### Color themes

The default configuration uses Catppuccin Mocha colors. You can customize any color using hex codes:

```toml
[style]
background = "#282828"  # Gruvbox dark
foreground = "#ebdbb2"
accent = "#83a598"
```

### Clock formats

Use strftime format strings:
- `"%H:%M:%S"` → 14:30:45
- `"%I:%M %p"` → 02:30 PM  
- `"%a %b %d, %H:%M"` → Mon Jan 15, 14:30

## Usage

### Starting oxidebar

```bash
oxidebar
```

### With niri

Add to your niri config (`~/.config/niri/config.kdl`):

```kdl
spawn-at-startup "oxidebar"
```

### Autostart with other compositors

For other Wayland compositors, add oxidebar to your autostart configuration.

## Performance

Benchmarked on a typical system:

| Metric | oxidebar | waybar |
|--------|----------|--------|
| RAM usage | 30-34 MB | 40-46 MB |
| CPU usage | ≤0.1% | 0.2-0.3% |
| Update latency | 200ms | 1000ms |

## Roadmap

- [ ] More modules (CPU, memory, disk usage)
- [ ] Click handlers for modules
- [ ] Better font rendering (TrueType fonts)
- [ ] Custom module separators
- [ ] Icon support
- [ ] Multiple bar instances
- [ ] Tooltip support
- [ ] Module animations

## Contributing

Contributions are welcome! This is a learning project focused on understanding Wayland protocols and efficient system programming in Rust.

## License

MIT

## Acknowledgments

- Inspired by [waybar](https://github.com/Alexays/Waybar)
- Built for [niri](https://github.com/YaLTeR/niri) compositor
- Uses [smithay-client-toolkit](https://github.com/Smithay/client-toolkit) for Wayland layer-shell support
