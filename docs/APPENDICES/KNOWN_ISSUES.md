# Known Issues

## Interactive GUI Viewer (interactive_viewer.rs)

### Issue
The interactive GUI viewer currently has borrow checker issues related to lifetime requirements in `egui_wgpu::Renderer::render()`.

### Error
```
error[E0597]: `encoder` does not live long enough
error[E0505]: cannot move out of `encoder` because it is borrowed
```

The `egui_wgpu::Renderer::render()` method appears to require a `'static` lifetime for the render pass, but the render pass borrows from the locally-scoped encoder, which cannot have a static lifetime.

### Root Cause
This is likely due to version incompatibilities between:
- `egui-wgpu 0.29.1`
- `egui-winit 0.29.1`
- `wgpu 22.1`
- `winit 0.30`

The egui-wgpu crate may have API changes that are not fully compatible with the current setup, or the lifetime annotations in the `render()` method signature may be overly conservative.

### Workarounds

#### Current Working Alternatives:
1. **PNG Frame Export** (`export_frames.rs`) - ✅ Fully Working
   ```bash
   cargo run --example export_frames --release
   ```
   Generates 20 PNG frames in `output/` directory

2. **MP4 Video Export** (`export_video.rs`) - ✅ Fully Working
   ```bash
   cargo run --example export_video --release
   ```
   Generates 300-frame MP4 video with ffmpeg

3. **Text-Based Viewer** (`simple_viewer.rs`) - ✅ Fully Working
   ```bash
   cargo run --example simple_viewer --release
   ```
   Displays real-time statistics in console

### Potential Solutions

1. **Update to Latest Versions**: Try upgrading to the latest egui-wgpu, egui-winit, and wgpu versions that are known to be compatible

2. **Use Different Renderer Pattern**: The egui-wgpu API may have changed to use a different rendering pattern (e.g., separate command buffer submission)

3. **Alternative GUI Framework**: Consider using `eframe` which provides a higher-level API that handles the wgpu integration internally

4. **Wait for Version Updates**: The ecosystem is rapidly evolving; future versions may resolve these compatibility issues

### Status
- Low priority - visualization is fully functional via PNG/MP4 export
- Interactive viewer can be revisited when the dependency ecosystem stabilizes
- Current export tools provide excellent visualization capabilities

### References
- egui-wgpu crate: https://crates.io/crates/egui-wgpu
- Related issue tracker: Consider filing an issue if this persists in latest versions
