//! Gravitational Lensing in 2+1D Curved Spacetime
//!
//! Demonstrates emergent curved geometry from a spacetime automaton.
//! A massive object curves space, and "light rays" (geodesics) bend around it.
//!
//! This shows that curved spacetime can emerge from:
//! - Dynamical network edge lengths
//! - Local metric responding to mass
//! - Geodesics as shortest paths in modified geometry
//!
//! Run with: cargo run --example curved_spacetime_lensing --release

use em_physics_sandbox::physics::CurvedSpacetime2D;
use glam::Vec2;
use std::fs;
use std::process::Command;

fn main() {
    println!("2+1D Curved Spacetime - Gravitational Lensing");
    println!("==============================================\n");

    println!("Spacetime Automaton Concept:");
    println!("  • Network nodes = spacetime points");
    println!("  • Edge lengths = metric (dynamic!)");
    println!("  • Mass curves space → modifies edge lengths");
    println!("  • Geodesics = shortest paths in curved network\n");

    // Create spacetime lattice
    let nx = 128;
    let ny = 128;
    let spacing = 0.05; // 5cm cells

    println!("Simulation parameters:");
    println!("  Grid: {}×{} nodes", nx, ny);
    println!("  Spacing: {:.2} m", spacing);
    println!("  Domain: {:.1}×{:.1} m\n", nx as f32 * spacing, ny as f32 * spacing);

    let mut spacetime = CurvedSpacetime2D::new(nx, ny, spacing);

    // Add a massive object at center
    let domain_center = Vec2::new(
        nx as f32 * spacing * 0.5,
        ny as f32 * spacing * 0.5,
    );

    let mass = 1e12; // kg (massive object)

    println!("Massive object:");
    println!("  Position: ({:.2}, {:.2}) m", domain_center.x, domain_center.y);
    println!("  Mass: {:.2e} kg\n", mass);

    spacetime.add_point_mass(domain_center, mass);

    // Update geometry
    println!("Computing curved geometry...");
    spacetime.update_metric();
    spacetime.compute_curvature();

    // Find maximum curvature
    let max_curvature = spacetime.nodes.iter()
        .map(|n| n.curvature.abs())
        .fold(0.0_f32, f32::max);

    let min_metric = spacetime.nodes.iter()
        .map(|n| n.metric_scale)
        .fold(f32::INFINITY, f32::min);

    let max_metric = spacetime.nodes.iter()
        .map(|n| n.metric_scale)
        .fold(0.0_f32, f32::max);

    println!("  Metric scale range: [{:.3}, {:.3}]", min_metric, max_metric);
    println!("  Max curvature: {:.3e} m⁻²\n", max_curvature);

    // Trace geodesics ("light rays") from left side
    println!("Tracing geodesics (light rays)...\n");

    let num_rays = 20;
    let mut geodesics = Vec::new();

    let start_x = nx as f32 * spacing * 0.1;
    let end_x = nx as f32 * spacing * 0.9;

    for i in 0..num_rays {
        let y_offset = (i as f32 / (num_rays - 1) as f32 - 0.5) * ny as f32 * spacing * 0.8;
        let start_y = domain_center.y + y_offset;
        let end_y = domain_center.y + y_offset;

        let start = Vec2::new(start_x, start_y);
        let end = Vec2::new(end_x, end_y);

        let geodesic = spacetime.trace_geodesic(start, end, 100);
        geodesics.push(geodesic);
    }

    println!("Traced {} geodesics\n", num_rays);

    // Create output directories
    fs::create_dir_all("output").ok();
    fs::create_dir_all("temp_frames").ok();

    // Export visualization frames (animate metric pulsing)
    let total_frames = 120;
    let fps = 30;

    println!("Generating visualization...");
    println!("  Frames: {}", total_frames);
    println!("  Output: {} FPS\n", fps);

    for frame in 0..total_frames {
        export_spacetime_frame(&spacetime, &geodesics, domain_center, frame, total_frames);

        if frame % 30 == 0 || frame == total_frames - 1 {
            let progress = 100.0 * (frame + 1) as f32 / total_frames as f32;
            println!("  Frame {}/{} ({:.0}%)", frame + 1, total_frames, progress);
        }
    }

    println!("\n✓ Generated {} frames\n", total_frames);

    // Encode video
    println!("Encoding video...");

    let status = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-framerate", &fps.to_string(),
            "-i", "temp_frames/spacetime_%04d.png",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-crf", "18",
            "-preset", "medium",
            "output/curved_spacetime_lensing.mp4",
        ])
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("✓ Video encoding complete!\n");
            fs::remove_dir_all("temp_frames").ok();

            if let Ok(metadata) = fs::metadata("output/curved_spacetime_lensing.mp4") {
                println!("File size: {:.1} MB", metadata.len() as f64 / 1_048_576.0);
            }

            println!("\n✓ SUCCESS!");
            println!("\nVideo: output/curved_spacetime_lensing.mp4");
            println!("\nWhat to see:");
            println!("  • Background: Metric curvature (blue=curved, red=flat)");
            println!("  • Black circle: Massive object");
            println!("  • White lines: Geodesics (light rays)");
            println!("  • Rays bend around mass → gravitational lensing!");
            println!("\nPhysics - Emergent Curved Spacetime:");
            println!("  • Network edge lengths = dynamic metric");
            println!("  • Mass → modifies local metric scale");
            println!("  • Geodesics follow shortest path in curved network");
            println!("  • Einstein's GR emerges from graph dynamics");
            println!("\nKey Insight:");
            println!("  Geometry is NOT background - it's computed from mass!");
            println!("  This is the automaton approach to curved spacetime.");
        }
        Ok(_) => {
            eprintln!("✗ ffmpeg encoding failed");
            eprintln!("Frames saved in temp_frames/");
        }
        Err(e) => {
            eprintln!("✗ ffmpeg error: {}", e);
            eprintln!("Frames saved in temp_frames/");
        }
    }
}

/// Export visualization of curved spacetime and geodesics
fn export_spacetime_frame(
    spacetime: &CurvedSpacetime2D,
    geodesics: &[Vec<Vec2>],
    mass_pos: Vec2,
    frame: usize,
    total_frames: usize,
) {
    let width = 800;
    let height = 800;

    let domain_width = spacetime.nx as f32 * spacetime.spacing;
    let domain_height = spacetime.ny as f32 * spacetime.spacing;

    let mut rgba = vec![0u8; width * height * 4];

    // Helper: world to pixel coordinates
    let world_to_pixel = |pos: Vec2| -> (i32, i32) {
        let x_norm = pos.x / domain_width;
        let y_norm = pos.y / domain_height;

        let px = (x_norm * width as f32) as i32;
        let py = height as i32 - 1 - (y_norm * height as f32) as i32;

        (px, py)
    };

    // Background: dark space
    for i in 0..width * height {
        rgba[i * 4] = 10;
        rgba[i * 4 + 1] = 10;
        rgba[i * 4 + 2] = 15;
        rgba[i * 4 + 3] = 255;
    }

    // Draw curvature as heatmap (more dramatic visualization)
    let max_curvature = spacetime.nodes.iter()
        .map(|n| n.curvature.abs())
        .fold(0.0_f32, f32::max);

    for j in 0..spacetime.ny {
        for i in 0..spacetime.nx {
            let idx_node = spacetime.index(i, j);
            let node = &spacetime.nodes[idx_node];

            let (px, py) = world_to_pixel(node.position);

            if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                let idx_pixel = (py as usize * width + px as usize) * 4;

                // Use curvature for coloring (more visible than metric)
                let curv_intensity = (node.curvature.abs() / max_curvature.max(1e-6)).clamp(0.0, 1.0);

                // Hot colormap: black -> red -> yellow -> white
                let r = (255.0 * curv_intensity.powf(0.4)) as u8;
                let g = (200.0 * (curv_intensity - 0.5).max(0.0) * 2.0) as u8;
                let b = (100.0 * (curv_intensity - 0.8).max(0.0) * 5.0) as u8;

                if idx_pixel + 3 < rgba.len() {
                    rgba[idx_pixel] = r;
                    rgba[idx_pixel + 1] = g;
                    rgba[idx_pixel + 2] = b;
                    rgba[idx_pixel + 3] = 255;
                }
            }
        }
    }

    // Draw metric contour lines for better depth perception
    for j in 1..spacetime.ny - 1 {
        for i in 1..spacetime.nx - 1 {
            let idx = spacetime.index(i, j);
            let g_center = spacetime.nodes[idx].metric_scale;

            // Check if we're at a contour line (metric changes significantly)
            let g_right = spacetime.nodes[spacetime.index(i + 1, j)].metric_scale;
            let g_down = spacetime.nodes[spacetime.index(i, j + 1)].metric_scale;

            // Contour spacing
            let contour_spacing = 0.05; // Every 0.05 metric units
            let level_center = (g_center / contour_spacing).floor();
            let level_right = (g_right / contour_spacing).floor();
            let level_down = (g_down / contour_spacing).floor();

            if level_center != level_right || level_center != level_down {
                let (px, py) = world_to_pixel(spacetime.nodes[idx].position);
                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    let idx_pixel = (py as usize * width + px as usize) * 4;
                    if idx_pixel + 3 < rgba.len() {
                        rgba[idx_pixel] = 100;
                        rgba[idx_pixel + 1] = 100;
                        rgba[idx_pixel + 2] = 150;
                        rgba[idx_pixel + 3] = 255;
                    }
                }
            }
        }
    }

    // Draw geodesics (light rays) with thickness for visibility
    for geodesic in geodesics {
        for window in geodesic.windows(2) {
            let p1 = window[0];
            let p2 = window[1];

            // Bresenham line drawing
            let (x1, y1) = world_to_pixel(p1);
            let (x2, y2) = world_to_pixel(p2);

            // Draw thick lines for better visibility
            for dy in -1..=1 {
                for dx in -1..=1 {
                    draw_line(&mut rgba, width, height, x1 + dx, y1 + dy, x2 + dx, y2 + dy, [200, 255, 255]);
                }
            }
        }
    }

    // Add grid overlay to show warped space
    let grid_spacing = 0.4; // Grid every 0.4m
    let num_grid_lines_x = (domain_width / grid_spacing) as usize;
    let num_grid_lines_y = (domain_height / grid_spacing) as usize;

    // Horizontal grid lines
    for j in 0..=num_grid_lines_y {
        let y_world = j as f32 * grid_spacing;
        for i in 0..100 {
            let x_world = (i as f32 / 99.0) * domain_width;
            let pos = Vec2::new(x_world, y_world);

            let (px, py) = world_to_pixel(pos);
            if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                let idx = (py as usize * width + px as usize) * 4;
                if idx + 3 < rgba.len() {
                    rgba[idx] = (rgba[idx] as u16 + 40).min(255) as u8;
                    rgba[idx + 1] = (rgba[idx + 1] as u16 + 40).min(255) as u8;
                    rgba[idx + 2] = (rgba[idx + 2] as u16 + 40).min(255) as u8;
                }
            }
        }
    }

    // Vertical grid lines
    for i in 0..=num_grid_lines_x {
        let x_world = i as f32 * grid_spacing;
        for j in 0..100 {
            let y_world = (j as f32 / 99.0) * domain_height;
            let pos = Vec2::new(x_world, y_world);

            let (px, py) = world_to_pixel(pos);
            if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                let idx = (py as usize * width + px as usize) * 4;
                if idx + 3 < rgba.len() {
                    rgba[idx] = (rgba[idx] as u16 + 40).min(255) as u8;
                    rgba[idx + 1] = (rgba[idx + 1] as u16 + 40).min(255) as u8;
                    rgba[idx + 2] = (rgba[idx + 2] as u16 + 40).min(255) as u8;
                }
            }
        }
    }

    // Draw massive object as black circle
    let (cx, cy) = world_to_pixel(mass_pos);
    let radius_pixels = 15;

    for dy in -radius_pixels..=radius_pixels {
        for dx in -radius_pixels..=radius_pixels {
            if dx * dx + dy * dy <= radius_pixels * radius_pixels {
                let px = cx + dx;
                let py = cy + dy;

                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    let idx = (py as usize * width + px as usize) * 4;

                    if idx + 3 < rgba.len() {
                        rgba[idx] = 0;
                        rgba[idx + 1] = 0;
                        rgba[idx + 2] = 0;
                        rgba[idx + 3] = 255;
                    }
                }
            }
        }
    }

    image::save_buffer(
        format!("temp_frames/spacetime_{:04}.png", frame),
        &rgba,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();
}

/// Draw a line using Bresenham's algorithm
fn draw_line(rgba: &mut [u8], width: usize, height: usize, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 3]) {
    let mut x = x0;
    let mut y = y0;

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            let idx = (y as usize * width + x as usize) * 4;
            if idx + 3 < rgba.len() {
                rgba[idx] = color[0];
                rgba[idx + 1] = color[1];
                rgba[idx + 2] = color[2];
                rgba[idx + 3] = 255;
            }
        }

        if x == x1 && y == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}
