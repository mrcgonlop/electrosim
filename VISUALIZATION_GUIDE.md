# Visualization Guide

This guide shows you how to visualize all the electromagnetic simulations in the sandbox.

**Status**: ✅ All visualization methods working! PNG export, MP4 video generation, and text-based viewer are fully functional. Interactive GUI viewer is temporarily disabled due to API compatibility issues (see [KNOWN_ISSUES.md](KNOWN_ISSUES.md)).

## 🎬 Quick Start - Watch All Demos as Videos

### Option 1: Export All Demos as PNG Frames (Fastest)

```bash
cargo run --example export_frames --release
```

**Output:** `output/*.png` - 20 frames showing:
- 2D wave propagation (circular waves)
- 3D Maxwell EM field evolution

**Time:** ~5 seconds

---

### Option 2: Create High-Quality MP4 Videos

```bash
# First time: Install ffmpeg (already done!)
# Just restart your terminal for PATH update

cargo run --example export_video --release
```

**Output:** `output/wave_simulation.mp4` - 10-second video at 30fps

**Time:** ~30 seconds (300 frames + encoding)

---

### Option 3: Live Statistics (No Visual, Just Numbers)

```bash
cargo run --example simple_viewer --release
```

**Output:** Real-time console stats showing:
- Frame count
- Simulation time
- Energy levels
- Field magnitudes
- Performance (FPS)

---

## 📊 How to Visualize Each Demo

### 1. **2D Wave Equation** (`run_2d_wave_demo`)

**What it shows:** Circular wave propagating outward from center

**Method A: PNG Frames**
```rust
use em_physics_sandbox::{
    physics::{Wave2D, ScalarWaveTheory},
    visualization::slice2d::colormaps,
};

let wave = Wave2D::new(1.0);
let mut field = vec![0.0; 128 * 128];
let mut velocity = vec![0.0; 128 * 128];

wave.set_gaussian_pulse(&mut field, (64, 64), 1.0, 8.0, 128, 128);

for frame in 0..50 {
    // Simulate
    wave.update_scalar_field(&mut field, &mut velocity, 128, 128, 0.1, 0.01);

    // Visualize every 5 frames
    if frame % 5 == 0 {
        export_2d_frame(&field, 128, 128, frame);
    }
}

fn export_2d_frame(field: &[f32], width: usize, height: usize, frame: usize) {
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;
    for &val in field {
        min_val = min_val.min(val);
        max_val = max_val.max(val);
    }

    let range = (max_val - min_val).max(1e-10);
    let mut rgba = Vec::new();

    for &val in field {
        let t = ((val - min_val) / range).clamp(0.0, 1.0);
        let rgb = colormaps::viridis(t);
        rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }

    image::save_buffer(
        format!("output/wave_{:04}.png", frame),
        &rgba,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    ).unwrap();
}
```

**Method B: Use Built-in Example**
```bash
# Modify export_video.rs to call record_2d_wave_video()
cargo run --example export_video --release
```

---

### 2. **3D Maxwell FDTD** (`run_maxwell_demo`)

**What it shows:** 3D electromagnetic field slice (XY plane through middle)

**Method A: Extract Slice and Export**
```rust
use em_physics_sandbox::{
    physics::{MaxwellTheory, EMTheory},
    simulation::VoxelGrid,
    visualization::{FieldSlice, SlicePlane, FieldComponent, slice2d::colormaps},
};

let maxwell = MaxwellTheory::new();
let mut grid = VoxelGrid::new(64, 64, 64, 0.01);
maxwell.initialize(&mut grid);

for frame in 0..50 {
    // Simulate
    for _ in 0..10 {
        maxwell.update_fields(&mut grid, 1e-12);
    }

    // Visualize
    if frame % 5 == 0 {
        let slice = FieldSlice::from_grid(
            &grid,
            SlicePlane::XY,  // Top-down view
            0.5,             // Middle of Z axis
            FieldComponent::EMagnitude,
        );

        let rgba = slice.to_rgba8(colormaps::hot);

        image::save_buffer(
            format!("output/maxwell_{:04}.png", frame),
            &rgba,
            slice.width as u32,
            slice.height as u32,
            image::ColorType::Rgba8,
        ).unwrap();
    }
}
```

**Method B: Use Built-in Example**
```bash
cargo run --example export_frames --release
# Check output/maxwell3d_frame_*.png
```

---

### 3. **Faraday Cage Experiment** (`run_faraday_cage_demo`)

**What it shows:** Electric field being blocked by conducting cage

**Visualization Code:**
```rust
use em_physics_sandbox::{
    experiments::{Experiment, FaradayCage},
    physics::{MaxwellTheory, EMTheory},
    simulation::VoxelGrid,
    visualization::{FieldSlice, SlicePlane, FieldComponent, slice2d::colormaps},
};

let maxwell = MaxwellTheory::new();
let cage = FaradayCage::new(0.7, 100.0);
let mut grid = VoxelGrid::new(32, 32, 32, 0.01);

cage.setup(&mut grid);

for frame in 0..100 {
    maxwell.update_fields(&mut grid, 1e-12);

    if frame % 10 == 0 {
        // Show E field magnitude - you'll see the cage blocking it!
        let slice = FieldSlice::from_grid(
            &grid,
            SlicePlane::XY,
            0.5,
            FieldComponent::EMagnitude,
        );

        let rgba = slice.to_rgba8(colormaps::viridis);

        image::save_buffer(
            format!("output/faraday_{:04}.png", frame),
            &rgba,
            slice.width as u32,
            slice.height as u32,
            image::ColorType::Rgba8,
        ).unwrap();
    }
}
```

---

## 🎨 Visualization Options for All Demos

### **Slice Planes** (for 3D simulations)

```rust
SlicePlane::XY  // Top-down view (looking along Z axis)
SlicePlane::XZ  // Side view (looking along Y axis)
SlicePlane::YZ  // Front view (looking along X axis)
```

### **Field Components**

```rust
FieldComponent::EMagnitude           // |E| - Electric field strength
FieldComponent::BMagnitude           // |B| - Magnetic field strength
FieldComponent::Ex                   // E_x component
FieldComponent::Ey                   // E_y component
FieldComponent::Ez                   // E_z component
FieldComponent::Bx                   // B_x component
FieldComponent::By                   // B_y component
FieldComponent::Bz                   // B_z component
FieldComponent::Energy               // Energy density
FieldComponent::PoyntingMagnitude    // |S| - Energy flow
```

### **Colormaps**

```rust
colormaps::viridis   // Scientific standard (purple -> green -> yellow)
colormaps::hot       // Heat map (black -> red -> yellow -> white)
colormaps::cool      // Cool colors (cyan -> magenta)
colormaps::gray      // Grayscale
colormaps::red_blue  // Diverging (good for +/- values)
```

---

## 📹 Creating Videos from Any Demo

### Step 1: Generate Frames

```rust
// In your demo, add this after each simulation step:
if frame % export_interval == 0 {
    let slice = FieldSlice::from_grid(&grid, SlicePlane::XY, 0.5, FieldComponent::EMagnitude);
    let rgba = slice.to_rgba8(colormaps::viridis);

    image::save_buffer(
        format!("temp_frames/frame_{:04}.png", frame / export_interval),
        &rgba,
        slice.width as u32,
        slice.height as u32,
        image::ColorType::Rgba8,
    ).unwrap();
}
```

### Step 2: Encode to MP4

```bash
# After restarting terminal (for ffmpeg PATH):
ffmpeg -framerate 30 -i temp_frames/frame_%04d.png \
       -c:v libx264 -pix_fmt yuv420p -crf 23 \
       output/my_simulation.mp4
```

Or use the built-in video export example and modify for your demo.

---

## 🖼️ Understanding the Visualizations

### **2D Wave Colors:**
- **Dark purple/blue** = Wave trough (negative/low)
- **Green** = Zero/equilibrium
- **Yellow** = Wave peak (positive/high)

### **3D Maxwell Colors (Hot colormap):**
- **Black** = No field
- **Red** = Low field strength
- **Yellow** = Medium field strength
- **White** = High field strength

### **Faraday Cage:**
- **Inside cage** = Should be dark (shielded)
- **Outside cage** = Bright (field present)
- **Cage walls** = Zero (conductor)

---

## 💡 Tips for Best Visualizations

1. **Frame Rate**: Export every 5-10 steps for smooth videos
2. **Resolution**: Use 128×128 or 256×256 for good quality
3. **Duration**: 200-300 frames = 7-10 seconds at 30fps
4. **Colormap**:
   - Use `viridis` for scientific accuracy
   - Use `hot` for dramatic effect
   - Use `red_blue` for showing +/- fields

5. **Field Component**:
   - Start with `EMagnitude` or `BMagnitude` for overview
   - Use `Energy` to see where energy concentrates
   - Use `PoyntingMagnitude` to see energy flow direction

---

## 🚀 Quick Command Reference

```bash
# PNG frames (fast preview)
cargo run --example export_frames --release

# MP4 video (after terminal restart for ffmpeg)
cargo run --example export_video --release

# Live statistics
cargo run --example simple_viewer --release

# Run main demos (no visualization)
cargo run --release

# Run tests
cargo test --lib visualization
```

---

## 📂 Output Locations

- **PNG frames**: `output/*.png`
- **Videos**: `output/*.mp4`
- **Temporary frames**: `temp_frames/*.png` (auto-cleaned after video encoding)

---

## 🎯 Example: Visualizing Your Own Simulation

```rust
use em_physics_sandbox::{
    physics::{MaxwellTheory, EMTheory},
    simulation::VoxelGrid,
    visualization::{FieldSlice, SlicePlane, FieldComponent, slice2d::colormaps},
};

fn main() {
    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(64, 64, 64, 0.01);

    // Your custom setup
    maxwell.initialize(&mut grid);

    // Simulate and visualize
    for frame in 0..100 {
        // Physics
        maxwell.update_fields(&mut grid, 1e-12);

        // Visualization (every 5 frames)
        if frame % 5 == 0 {
            let slice = FieldSlice::from_grid(
                &grid,
                SlicePlane::XY,
                0.5,
                FieldComponent::EMagnitude,
            );

            let rgba = slice.to_rgba8(colormaps::viridis);

            std::fs::create_dir_all("output").ok();
            image::save_buffer(
                format!("output/my_sim_{:04}.png", frame / 5),
                &rgba,
                slice.width as u32,
                slice.height as u32,
                image::ColorType::Rgba8,
            ).unwrap();
        }
    }

    println!("Frames saved to output/ directory!");
}
```

Then create video:
```bash
ffmpeg -framerate 30 -i output/my_sim_%04d.png -c:v libx264 -pix_fmt yuv420p output/my_simulation.mp4
```

---

**All demos can be visualized!** Choose PNG for quick previews or MP4 for polished results. 🎉
