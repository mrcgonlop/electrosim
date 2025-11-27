//! Interactive 2D field viewer with GUI
//!
//! Cross-platform electromagnetic field visualization with real-time controls.
//! Works on Windows, Mac, and Linux.
//!
//! Run with: cargo run --example interactive_viewer --release

use em_physics_sandbox::{
    physics::{EMTheory, MaxwellTheory},
    simulation::VoxelGrid,
    visualization::viewer::ViewerApp,
};
use pollster::FutureExt;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("EM Physics Sandbox - Interactive Viewer");
    println!("=======================================\n");
    println!("Initializing...");

    // Create event loop
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    // Create and run application
    let mut app = Application::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}

struct Application {
    window: Option<Arc<Window>>,
    state: Option<State>,
}

impl Application {
    fn new() -> Self {
        Self {
            window: None,
            state: None,
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("EM Physics Sandbox - 2D Viewer")
                .with_inner_size(winit::dpi::PhysicalSize::new(1200, 800));

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.window = Some(window.clone());

            // Initialize wgpu state
            let state = State::new(window.clone()).block_on();
            self.state = Some(state);

            println!("Window created! Use the controls to explore the simulation.");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let window = match self.window.as_ref() {
            Some(w) if w.id() == window_id => w,
            _ => return,
        };

        let state = match self.state.as_mut() {
            Some(s) => s,
            None => return,
        };

        // Pass event to egui first
        let response = state.egui_state.on_window_event(window, &event);

        if response.repaint {
            window.request_redraw();
        }

        if !response.consumed {
            match event {
                WindowEvent::CloseRequested => {
                    println!("\nShutting down...");
                    event_loop.exit();
                }
                WindowEvent::Resized(new_size) => {
                    state.resize(new_size);
                }
                WindowEvent::RedrawRequested => {
                    state.render();
                }
                _ => {}
            }
        }
    }
}

struct State {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    app: ViewerApp,
    maxwell: MaxwellTheory,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Main Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Initialize egui
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            Some(winit::window::Theme::Dark),
            Some(2048),
        );

        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1, false);

        // Create simulation
        let mut grid = VoxelGrid::new(64, 64, 64, 0.01);
        let maxwell = MaxwellTheory::new();
        maxwell.initialize(&mut grid);

        let app = ViewerApp::new(grid);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            egui_ctx,
            egui_state,
            egui_renderer,
            app,
            maxwell,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) {
        // Step simulation if not paused
        if !self.app.viewer.paused {
            self.maxwell.update_fields(&mut self.app.grid, self.app.dt);
            self.app.time += self.app.dt;
            self.app.frame += 1;
        }

        // Update texture
        self.app.update_texture(&self.device, &self.queue);

        // Render egui
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            self.app.ui(ctx);
        });

        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);

        let clipped_primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, self.egui_ctx.pixels_per_point());

        // Render frame
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        // Update egui buffers before rendering
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }

        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        // Render egui in its own scope to ensure render_pass is dropped before encoder.finish()
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Render egui (now just draws, buffers already updated)
            self.egui_renderer
                .render(&mut render_pass, &clipped_primitives, &screen_descriptor);

            drop(render_pass); // Explicitly drop render_pass
        }

        // Free egui textures
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        self.window.request_redraw();
    }
}
