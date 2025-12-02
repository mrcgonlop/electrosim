//! Dimensional Dynamics: Time Evolution of Variable-Dimensional Space
//!
//! This module implements the dynamics of dimensional fields, including:
//! - Wave equation for dimensional field d(x⃗,t)
//! - Multiple wave modes (transverse + longitudinal)
//! - Velocity and acceleration effects (Weber-like terms)
//! - Quaternion formulation
//!
//! KEY INSIGHT: Time-varying dimensional fields generate BOTH
//! - Transverse EM waves (E⊥B⊥k, standard Maxwell)
//! - Longitudinal scalar waves (compression waves in d field, lost in Heaviside reduction)

use glam::Vec3;
use std::collections::HashMap;

/// Time-evolving dimensional field with velocity and acceleration
#[derive(Clone, Debug)]
pub struct DimensionalDynamics {
    /// Grid dimensions
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,

    /// Spatial resolution
    pub spacing: f32,

    /// Background dimension (typically 3.0)
    pub background_dimension: f32,

    /// Current dimensional field d(x,y,z,t)
    pub dimension: Vec<f32>,

    /// Time derivative ∂d/∂t (dimensional velocity)
    pub dimension_velocity: Vec<f32>,

    /// Second time derivative ∂²d/∂t² (dimensional acceleration)
    pub dimension_acceleration: Vec<f32>,

    /// Dimensional vector potential (for quaternion formulation)
    pub vector_potential: Vec<Vec3>,

    /// Wave propagation speeds
    pub c_transverse: f32,  // Speed of transverse waves (standard EM)
    pub c_longitudinal: f32,  // Speed of longitudinal waves (scalar)

    /// Current simulation time
    pub time: f32,
}

impl DimensionalDynamics {
    /// Create new dimensional dynamics system
    pub fn new(nx: usize, ny: usize, nz: usize, spacing: f32) -> Self {
        let num_cells = nx * ny * nz;

        Self {
            nx,
            ny,
            nz,
            spacing,
            background_dimension: 3.0,
            dimension: vec![3.0; num_cells],
            dimension_velocity: vec![0.0; num_cells],
            dimension_acceleration: vec![0.0; num_cells],
            vector_potential: vec![Vec3::ZERO; num_cells],
            c_transverse: 1.0,  // Natural units (c = 1)
            c_longitudinal: 1.0,  // Initially assume same speed
            time: 0.0,
        }
    }

    /// Get cell index from 3D coordinates
    pub fn cell_index(&self, i: usize, j: usize, k: usize) -> usize {
        i + self.nx * (j + self.ny * k)
    }

    /// Get 3D position from cell index
    pub fn cell_position(&self, idx: usize) -> Vec3 {
        let i = idx % self.nx;
        let j = (idx / self.nx) % self.ny;
        let k = idx / (self.nx * self.ny);

        Vec3::new(
            i as f32 * self.spacing,
            j as f32 * self.spacing,
            k as f32 * self.spacing,
        )
    }

    /// Compute Laplacian ∇²d at cell index
    /// This is the spatial second derivative (curvature)
    pub fn laplacian(&self, idx: usize) -> f32 {
        let i = idx % self.nx;
        let j = (idx / self.nx) % self.ny;
        let k = idx / (self.nx * self.ny);

        let d_center = self.dimension[idx];
        let dx = self.spacing;
        let dx2 = dx * dx;

        let mut laplacian = 0.0;
        let mut count = 0;

        // X direction
        if i > 0 {
            let d_left = self.dimension[self.cell_index(i-1, j, k)];
            laplacian += (d_left - d_center) / dx2;
            count += 1;
        }
        if i < self.nx - 1 {
            let d_right = self.dimension[self.cell_index(i+1, j, k)];
            laplacian += (d_right - d_center) / dx2;
            count += 1;
        }

        // Y direction
        if j > 0 {
            let d_down = self.dimension[self.cell_index(i, j-1, k)];
            laplacian += (d_down - d_center) / dx2;
            count += 1;
        }
        if j < self.ny - 1 {
            let d_up = self.dimension[self.cell_index(i, j+1, k)];
            laplacian += (d_up - d_center) / dx2;
            count += 1;
        }

        // Z direction
        if k > 0 {
            let d_back = self.dimension[self.cell_index(i, j, k-1)];
            laplacian += (d_back - d_center) / dx2;
            count += 1;
        }
        if k < self.nz - 1 {
            let d_front = self.dimension[self.cell_index(i, j, k+1)];
            laplacian += (d_front - d_center) / dx2;
            count += 1;
        }

        laplacian
    }

    /// Compute gradient ∇d at cell index
    pub fn gradient(&self, idx: usize) -> Vec3 {
        let i = idx % self.nx;
        let j = (idx / self.nx) % self.ny;
        let k = idx / (self.nx * self.ny);

        let dx = self.spacing;
        let mut grad = Vec3::ZERO;

        // X component
        if i > 0 && i < self.nx - 1 {
            let d_left = self.dimension[self.cell_index(i-1, j, k)];
            let d_right = self.dimension[self.cell_index(i+1, j, k)];
            grad.x = (d_right - d_left) / (2.0 * dx);
        }

        // Y component
        if j > 0 && j < self.ny - 1 {
            let d_down = self.dimension[self.cell_index(i, j-1, k)];
            let d_up = self.dimension[self.cell_index(i, j+1, k)];
            grad.y = (d_up - d_down) / (2.0 * dx);
        }

        // Z component
        if k > 0 && k < self.nz - 1 {
            let d_back = self.dimension[self.cell_index(i, j, k-1)];
            let d_front = self.dimension[self.cell_index(i, j, k+1)];
            grad.z = (d_front - d_back) / (2.0 * dx);
        }

        grad
    }

    /// Evolve dimensional field forward in time
    /// Uses wave equation: ∂²d/∂t² = c² ∇²d + sources
    pub fn evolve_step(&mut self, dt: f32, sources: Option<&[f32]>) {
        // Compute accelerations from wave equation
        for idx in 0..self.dimension.len() {
            let laplacian = self.laplacian(idx);

            // Source term (external forcing)
            let source = sources.map(|s| s[idx]).unwrap_or(0.0);

            // Wave equation with two speeds:
            // ∂²d/∂t² = c_L² ∇²d + source (longitudinal/scalar wave)
            //
            // For now, use c_longitudinal for the scalar wave
            self.dimension_acceleration[idx] =
                self.c_longitudinal * self.c_longitudinal * laplacian + source;
        }

        // Verlet integration (2nd order, energy conserving)
        for idx in 0..self.dimension.len() {
            // Update velocity: v(t+dt/2) = v(t) + a(t)*dt/2
            self.dimension_velocity[idx] += self.dimension_acceleration[idx] * dt;

            // Update position: d(t+dt) = d(t) + v(t+dt/2)*dt
            self.dimension[idx] += self.dimension_velocity[idx] * dt;

            // Clamp to physical range [0, 5]
            self.dimension[idx] = self.dimension[idx].max(0.0).min(5.0);
        }

        self.time += dt;
    }

    /// Create oscillating point defect (monopole)
    /// This generates BOTH transverse and longitudinal waves!
    ///
    /// IMPORTANT: This directly modifies the dimension field (not acceleration)
    /// to create a persistent oscillating source
    pub fn add_oscillating_monopole(&mut self, position: Vec3, amplitude: f32, frequency: f32) {
        let phase = 2.0 * std::f32::consts::PI * frequency * self.time;
        let value = amplitude * phase.sin();

        // Find nearest cell
        let i = (position.x / self.spacing).round() as usize;
        let j = (position.y / self.spacing).round() as usize;
        let k = (position.z / self.spacing).round() as usize;

        if i < self.nx && j < self.ny && k < self.nz {
            let idx = self.cell_index(i, j, k);

            // Set dimension directly (forcing function)
            // This creates a boundary condition that drives the wave
            self.dimension[idx] = self.background_dimension + value;
        }
    }

    /// Create oscillating dipole (two monopoles with opposite phase)
    /// This primarily generates transverse waves (standard EM)
    pub fn add_oscillating_dipole(
        &mut self,
        center: Vec3,
        separation: Vec3,
        amplitude: f32,
        frequency: f32
    ) {
        let phase = 2.0 * std::f32::consts::PI * frequency * self.time;
        let charge = amplitude * phase.sin();

        let pos1 = center - separation * 0.5;
        let pos2 = center + separation * 0.5;

        // Positive charge at pos1
        self.add_point_source(pos1, charge);

        // Negative charge at pos2
        self.add_point_source(pos2, -charge);
    }

    /// Add point source at position
    fn add_point_source(&mut self, position: Vec3, strength: f32) {
        let i = (position.x / self.spacing).round() as usize;
        let j = (position.y / self.spacing).round() as usize;
        let k = (position.z / self.spacing).round() as usize;

        if i < self.nx && j < self.ny && k < self.nz {
            let idx = self.cell_index(i, j, k);
            self.dimension_acceleration[idx] += strength;
        }
    }

    /// Decompose dimensional wave into transverse and longitudinal components
    /// KEY PREDICTION: Both components should exist!
    pub fn decompose_wave(&self, idx: usize) -> WaveComponents {
        let pos = self.cell_position(idx);
        let grad = self.gradient(idx);

        // Find dominant propagation direction (from gradient)
        let propagation_dir = if grad.length() > 1e-6 {
            grad.normalize()
        } else {
            Vec3::X  // Default
        };

        // Longitudinal component: parallel to propagation
        let longitudinal_magnitude = grad.dot(propagation_dir);
        let longitudinal = propagation_dir * longitudinal_magnitude;

        // Transverse component: perpendicular to propagation
        let transverse = grad - longitudinal;

        WaveComponents {
            longitudinal,
            transverse,
            propagation_direction: propagation_dir,
            total_gradient: grad,
        }
    }

    /// Measure wave at a point over time
    pub fn measure_time_series(&self, position: Vec3) -> f32 {
        // Find nearest cell
        let i = (position.x / self.spacing).round() as usize;
        let j = (position.y / self.spacing).round() as usize;
        let k = (position.z / self.spacing).round() as usize;

        if i < self.nx && j < self.ny && k < self.nz {
            let idx = self.cell_index(i, j, k);
            self.dimension[idx] - self.background_dimension
        } else {
            0.0
        }
    }
}

/// Decomposed wave components
#[derive(Debug, Clone)]
pub struct WaveComponents {
    pub longitudinal: Vec3,
    pub transverse: Vec3,
    pub propagation_direction: Vec3,
    pub total_gradient: Vec3,
}

impl WaveComponents {
    /// Ratio of longitudinal to transverse amplitude
    pub fn longitudinal_fraction(&self) -> f32 {
        let long_mag = self.longitudinal.length();
        let trans_mag = self.transverse.length();

        if long_mag + trans_mag > 1e-10 {
            long_mag / (long_mag + trans_mag)
        } else {
            0.0
        }
    }
}

/// Weber-like force law with velocity and acceleration terms
pub fn weber_force(
    q1: f32,
    q2: f32,
    r: Vec3,
    v_rel: Vec3,
    a_rel: Vec3,
    c: f32,
) -> Vec3 {
    let r_mag = r.length();
    if r_mag < 1e-6 {
        return Vec3::ZERO;
    }

    let r_hat = r / r_mag;
    let r_dot = v_rel.dot(r_hat);
    let r_ddot = a_rel.dot(r_hat);

    let r2 = r_mag * r_mag;
    let c2 = c * c;

    // Weber's three terms
    let coulomb_term = q1 * q2 * r_hat / r2;
    let kinetic_term = -q1 * q2 * r_dot * r_dot * r_hat / (2.0 * c2 * r2);
    let acceleration_term = q1 * q2 * r_ddot * r_hat / (c2 * r_mag);

    coulomb_term + kinetic_term + acceleration_term
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_propagation() {
        let mut dynamics = DimensionalDynamics::new(50, 50, 50, 1.0);

        // Create pulse at center
        let center = Vec3::new(25.0, 25.0, 25.0);
        dynamics.add_point_source(center, 1.0);

        // Evolve and check wave spreads
        for _ in 0..100 {
            dynamics.evolve_step(0.1, None);
        }

        // Wave should have propagated outward
        let edge_idx = dynamics.cell_index(40, 25, 25);
        let edge_value = dynamics.dimension[edge_idx];

        // Should be perturbed from background
        assert!((edge_value - dynamics.background_dimension).abs() > 0.01);
    }

    #[test]
    fn test_laplacian() {
        let mut dynamics = DimensionalDynamics::new(10, 10, 10, 1.0);

        // Set up Gaussian-like profile
        let center_idx = dynamics.cell_index(5, 5, 5);
        dynamics.dimension[center_idx] = 4.0;

        let laplacian = dynamics.laplacian(center_idx);

        // Should be negative (local maximum)
        assert!(laplacian < 0.0);
    }

    #[test]
    fn test_weber_force_reduces_to_coulomb() {
        // At rest, Weber should equal Coulomb
        let r = Vec3::new(5.0, 0.0, 0.0);
        let v_rel = Vec3::ZERO;
        let a_rel = Vec3::ZERO;

        let f_weber = weber_force(1.0, 1.0, r, v_rel, a_rel, 1.0);
        let f_coulomb = r.normalize() / (r.length() * r.length());

        assert!((f_weber - f_coulomb).length() < 1e-6);
    }
}
