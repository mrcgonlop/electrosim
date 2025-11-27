//! 2D field viewer example
//!
//! Demonstrates real-time visualization of electromagnetic fields.
//!
//! Run with: cargo run --example viewer_2d --release

use em_physics_sandbox::{
    physics::{EMTheory, MaxwellTheory},
    simulation::VoxelGrid,
    visualization::{FieldComponent, SlicePlane, Viewer2D, ViewerApp},
};
use pollster::FutureExt;
use std::sync::Arc;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();

    // Create event loop and window
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("EM Physics Sandbox - 2D Viewer")
            .with_inner_size(winit::dpi::PhysicalSize::new(1200, 800))
            .build(&event_loop)
            .unwrap(),
    );

    // Initialize wgpu
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
        .block_on()
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
        .block_on()
        .unwrap();

    // Configure surface
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    let size = window.inner_size();
    let mut config = wgpu::SurfaceConfiguration {
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
    let mut egui_ctx = egui::Context::default();
    let mut egui_state = egui_winit::State::new(
        egui_ctx.clone(),
        egui::ViewportId::ROOT,
        &window,
        Some(window.scale_factor() as f32),
        Some(device.limits().max_texture_dimension_2d as usize),
        None,
    );

    let mut egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1, false);

    // Create simulation
    println!("Initializing simulation...");
    let mut grid = VoxelGrid::new(64, 64, 64, 0.01);
    let maxwell = MaxwellTheory::new();

    // Initialize with Gaussian pulse
    maxwell.initialize(&mut grid);

    let mut app = ViewerApp::new(grid);
    app.dt = 1e-12;

    println!("Starting visualization...");
    println!("Controls:");
    println!("  - Use the left panel to adjust visualization settings");
    println!("  - Change slice plane (XY/XZ/YZ)");
    println!("  - Select different field components");
    println!("  - Choose colormap");
    println!("  - Play/Pause simulation");

    event_loop
        .run(move |event, target| {
            target.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { event, .. } => {
                    // Pass event to egui
                    let response = egui_state.on_window_event(&window, &event);

                    if response.repaint {
                        window.request_redraw();
                    }

                    if !response.consumed {
                        match event {
                            WindowEvent::CloseRequested => {
                                target.exit();
                            }
                            WindowEvent::Resized(new_size) => {
                                config.width = new_size.width;
                                config.height = new_size.height;
                                surface.configure(&device, &config);
                            }
                            WindowEvent::RedrawRequested => {
                                // Step simulation if not paused
                                if !app.viewer.paused {
                                    maxwell.update_fields(&mut app.grid, app.dt);
                                    app.time += app.dt;
                                    app.frame += 1;
                                }

                                // Update texture
                                app.update_texture(&device, &queue);

                                // Render egui
                                let raw_input = egui_state.take_egui_input(&window);
                                let full_output = egui_ctx.run(raw_input, |ctx| {
                                    app.ui(ctx);
                                });

                                egui_state.handle_platform_output(&window, full_output.platform_output);

                                let paint_jobs = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

                                // Upload egui textures
                                for (id, image_delta) in &full_output.textures_delta.set {
                                    egui_renderer.update_texture(&device, &queue, *id, image_delta);
                                }

                                // Render
                                let frame = surface.get_current_texture().unwrap();
                                let view = frame
                                    .texture
                                    .create_view(&wgpu::TextureViewDescriptor::default());

                                let mut encoder =
                                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                        label: Some("Render Encoder"),
                                    });

                                {
                                    let mut render_pass =
                                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                            label: Some("Render Pass"),
                                            color_attachments: &[Some(
                                                wgpu::RenderPassColorAttachment {
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
                                                },
                                            )],
                                            depth_stencil_attachment: None,
                                            timestamp_writes: None,
                                            occlusion_query_set: None,
                                        });

                                    // Render egui
                                    egui_renderer.render(
                                        &mut render_pass,
                                        &paint_jobs,
                                        &egui::PixelsPerPoint::new(window.scale_factor() as f32),
                                    );
                                }

                                // Free egui textures
                                for id in &full_output.textures_delta.free {
                                    egui_renderer.free_texture(id);
                                }

                                queue.submit(std::iter::once(encoder.finish()));
                                frame.present();

                                window.request_redraw();
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}
