use anyhow::Result;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_seat,
    delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{Capability, SeatHandler, SeatState},
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_seat, wl_surface},
    Connection, QueueHandle,
};

mod bar;
mod modules;
mod niri;
mod config;
use bar::BarRenderer;
use config::Config;

// Main application state
struct WaybarTui {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    compositor_state: CompositorState,
    shm_state: Shm,
    layer_shell: LayerShell,
    
    // Our bar surface
    layer_surface: Option<LayerSurface>,
    surface: Option<wl_surface::WlSurface>,
    
    // Rendering
    pool: Option<SlotPool>,
    width: u32,
    height: u32,
    
    // Bar renderer
    renderer: Arc<Mutex<BarRenderer>>,
    
    // Configuration
    configured: bool,
    
    // Update tracking
    last_update: Instant,
}

impl WaybarTui {
    fn new(
        registry_state: RegistryState,
        seat_state: SeatState,
        output_state: OutputState,
        compositor_state: CompositorState,
        shm_state: Shm,
        layer_shell: LayerShell,
        config: Config,
        bar_height: u32,
    ) -> Self {
        Self {
            registry_state,
            seat_state,
            output_state,
            compositor_state,
            shm_state,
            layer_shell,
            layer_surface: None,
            surface: None,
            pool: None,
            width: 0,
            height: bar_height,
            renderer: Arc::new(Mutex::new(BarRenderer::new(config))),
            configured: false,
            last_update: Instant::now(),
        }
    }

    fn create_layer_surface(&mut self, qh: &QueueHandle<Self>) {
        // Create the surface
        let surface = self.compositor_state.create_surface(qh);
        
        // Create layer surface
        let layer_surface = self.layer_shell.create_layer_surface(
            qh,
            surface.clone(),
            Layer::Top,
            Some("oxidebar"),
            None,
        );

        // Configure the layer surface
        layer_surface.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
        layer_surface.set_size(0, self.height); // 0 width = full screen width
        layer_surface.set_exclusive_zone(self.height as i32); // Reserve space
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        
        // Commit to apply changes
        surface.commit();
        
        self.layer_surface = Some(layer_surface);
        self.surface = Some(surface);
    }

    fn draw(&mut self, _qh: &QueueHandle<Self>) {
        if !self.configured {
            return;
        }

        let width = self.width;
        let height = self.height;
        
        // Get or create buffer pool
        let pool = self.pool.get_or_insert_with(|| {
            SlotPool::new(
                (width * height * 4) as usize,
                &self.shm_state,
            ).expect("Failed to create pool")
        });

        // Get a buffer - handle errors gracefully
        let (buffer, canvas) = match pool.create_buffer(
            width as i32,
            height as i32,
            width as i32 * 4,
            wayland_client::protocol::wl_shm::Format::Argb8888,
        ) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Failed to create buffer: {}", e);
                return;
            }
        };

        // Render the bar content
        self.renderer.lock().unwrap().render(canvas, width, height);

        // Attach buffer and damage surface
        if let Some(surface) = &self.surface {
            if let Err(e) = buffer.attach_to(surface) {
                eprintln!("Failed to attach buffer: {}", e);
                return;
            }
            surface.damage_buffer(0, 0, width as i32, height as i32);
            
            // Request frame callback to schedule next redraw
            surface.frame(_qh, surface.clone());
            
            surface.commit();
        }
    }
    
    fn should_redraw(&mut self) -> bool {
        // Redraw every 200ms for snappy workspace updates
        if self.last_update.elapsed() >= Duration::from_millis(200) {
            self.last_update = Instant::now();
            true
        } else {
            false
        }
    }
}

// Implement required trait handlers
impl CompositorHandler for WaybarTui {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // Handle scale factor changes if needed
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        // Only redraw if enough time has passed
        if self.should_redraw() {
            self.draw(qh);
        }
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }
}

impl OutputHandler for WaybarTui {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for WaybarTui {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        // Handle surface closed
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        // Get the configured size
        let (w, h) = configure.new_size;
        self.width = w;
        self.height = h.max(self.height);

        self.configured = true;
        
        // Request initial frame callback to start the redraw cycle
        if let Some(surface) = &self.surface {
            surface.frame(qh, surface.clone());
        }
        
        self.draw(qh);
    }
}

impl SeatHandler for WaybarTui {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wl_seat::WlSeat,
        _capability: Capability,
    ) {
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wl_seat::WlSeat,
        _capability: Capability,
    ) {
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl ShmHandler for WaybarTui {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm_state
    }
}

impl ProvidesRegistryState for WaybarTui {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState];
}

// Delegate implementations
delegate_compositor!(WaybarTui);
delegate_output!(WaybarTui);
delegate_shm!(WaybarTui);
delegate_seat!(WaybarTui);
delegate_layer!(WaybarTui);
delegate_registry!(WaybarTui);

fn main() -> Result<()> {
    // Handle --version flag
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("oxidebar {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    
    // Load configuration
    let config = Config::load();
    let bar_height = config.height;
    
    // Connect to Wayland
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    // Initialize required states
    let compositor_state = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm_state = Shm::bind(&globals, &qh)?;
    
    let mut app = WaybarTui::new(
        RegistryState::new(&globals),
        SeatState::new(&globals, &qh),
        OutputState::new(&globals, &qh),
        compositor_state,
        shm_state,
        layer_shell,
        config,
        bar_height,
    );

    // Create the layer surface
    app.create_layer_surface(&qh);

    // Initial roundtrip to get initial configure
    event_queue.roundtrip(&mut app)?;

    // Main event loop with faster refresh rate
    loop {
        // Dispatch events with error handling
        match event_queue.dispatch_pending(&mut app) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Event dispatch error: {}", e);
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
        }
        
        // Force redraw at configured interval (200ms)
        if app.should_redraw() {
            if let Some(surface) = &app.surface {
                surface.frame(&qh, surface.clone());
                surface.commit();
            }
            app.draw(&qh);
        }
        
        // Sleep briefly to avoid spinning
        std::thread::sleep(Duration::from_millis(50));
        
        // Flush the connection with error handling
        if let Err(e) = event_queue.flush() {
            eprintln!("Flush error: {}. Connection may be broken.", e);
            break;
        }
    }
    
    Ok(())
}
