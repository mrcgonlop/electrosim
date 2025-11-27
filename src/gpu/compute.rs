//! GPU compute pipeline setup for field updates

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;

/// GPU compute pipeline for electromagnetic field updates
///
/// This uses wgpu compute shaders to accelerate field calculations.
///
/// TODO: Implement full GPU pipeline with WGSL shaders
pub struct ComputePipeline {
    enabled: bool,
}

impl ComputePipeline {
    /// Create a new compute pipeline
    ///
    /// # Example
    ///
    /// ```no_run
    /// use em_physics_sandbox::gpu::ComputePipeline;
    ///
    /// // Note: This requires an async runtime
    /// // let pipeline = ComputePipeline::new().await.unwrap();
    /// ```
    pub async fn new() -> Result<Self> {
        // TODO: Initialize wgpu device and queue
        log::info!("GPU compute pipeline not yet implemented, using CPU fallback");

        Ok(Self {
            enabled: false,
        })
    }

    /// Create a disabled compute pipeline (CPU fallback)
    pub fn disabled() -> Self {
        Self {
            enabled: false,
        }
    }

    /// Update fields using GPU compute shader
    ///
    /// # Arguments
    ///
    /// * `e_field` - Electric field buffer
    /// * `b_field` - Magnetic field buffer
    /// * `dt` - Time step
    /// * `dx` - Grid spacing
    pub fn update_fields(
        &mut self,
        _e_field: &mut [Vec3],
        _b_field: &mut [Vec3],
        _dt: f32,
        _dx: f32,
    ) -> Result<()> {
        if !self.enabled {
            anyhow::bail!("GPU compute pipeline not enabled");
        }

        // TODO: Dispatch compute shader
        log::warn!("GPU compute not implemented, use CPU theory update instead");

        Ok(())
    }

    /// Check if GPU compute is available
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// GPU buffer representation for Vec3 (must be 16-byte aligned for WGSL)
#[repr(C, align(16))]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuVec3 {
    x: f32,
    y: f32,
    z: f32,
    _padding: f32,
}

impl From<Vec3> for GpuVec3 {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            _padding: 0.0,
        }
    }
}

impl From<GpuVec3> for Vec3 {
    fn from(v: GpuVec3) -> Self {
        Vec3::new(v.x, v.y, v.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_pipeline_disabled() {
        let pipeline = ComputePipeline::disabled();
        assert!(!pipeline.is_enabled());
    }

    #[test]
    fn test_gpu_vec3_conversion() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let gpu_v: GpuVec3 = v.into();
        let v2: Vec3 = gpu_v.into();

        assert_eq!(v, v2);
    }

    #[test]
    fn test_gpu_vec3_alignment() {
        // WGSL requires 16-byte alignment for vec3
        assert_eq!(std::mem::size_of::<GpuVec3>(), 16);
        assert_eq!(std::mem::align_of::<GpuVec3>(), 16);
    }
}
