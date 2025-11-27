//! 2D fluid dynamics for vortex simulations
//!
//! Implements simplified 2D incompressible Navier-Stokes equations
//! for simulating vortex interactions and fluid flow.

use glam::Vec2;

/// 2D fluid simulation state
pub struct Fluid2D {
    /// Grid dimensions
    pub nx: usize,
    pub ny: usize,

    /// Cell spacing (meters)
    pub spacing: f32,

    /// Velocity field (u, v)
    pub velocity: Vec<Vec2>,

    /// Vorticity field (ω = ∂v/∂x - ∂u/∂y)
    pub vorticity: Vec<f32>,

    /// Pressure field
    pub pressure: Vec<f32>,

    /// Charge density field (C/m²) - for EM analog
    pub charge_density: Vec<f32>,

    /// Electric field (V/m) - computed from pressure gradient
    pub e_field: Vec<Vec2>,

    /// Kinematic viscosity (m²/s)
    pub viscosity: f32,

    /// Time step (seconds)
    pub dt: f32,
}

impl Fluid2D {
    /// Create a new 2D fluid simulation
    pub fn new(nx: usize, ny: usize, spacing: f32, viscosity: f32, dt: f32) -> Self {
        let size = nx * ny;

        Self {
            nx,
            ny,
            spacing,
            velocity: vec![Vec2::ZERO; size],
            vorticity: vec![0.0; size],
            pressure: vec![0.0; size],
            charge_density: vec![0.0; size],
            e_field: vec![Vec2::ZERO; size],
            viscosity,
            dt,
        }
    }

    /// Get linear index from 2D coordinates
    #[inline]
    pub fn index(&self, i: usize, j: usize) -> usize {
        i + self.nx * j
    }

    /// Check if coordinates are valid
    #[inline]
    pub fn in_bounds(&self, i: i32, j: i32) -> bool {
        i >= 0 && i < self.nx as i32 && j >= 0 && j < self.ny as i32
    }

    /// Add a vortex at position (x, y) with given circulation
    pub fn add_vortex(&mut self, center: Vec2, circulation: f32, core_radius: f32) {
        for j in 0..self.ny {
            for i in 0..self.nx {
                let pos = Vec2::new(
                    i as f32 * self.spacing,
                    j as f32 * self.spacing,
                );

                let r_vec = pos - center;
                let r = r_vec.length();

                if r < 1e-6 {
                    continue;
                }

                // Lamb-Oseen vortex velocity profile
                // v_θ = (Γ / 2πr) * (1 - exp(-r²/σ²))
                let theta = r_vec.y.atan2(r_vec.x);
                let v_theta = (circulation / (2.0 * std::f32::consts::PI * r)) *
                             (1.0 - (-r * r / (core_radius * core_radius)).exp());

                // Convert to Cartesian
                let v_x = -v_theta * theta.sin();
                let v_y = v_theta * theta.cos();

                let idx = self.index(i, j);
                self.velocity[idx] += Vec2::new(v_x, v_y);
            }
        }

        // Update vorticity
        self.compute_vorticity();
    }

    /// Compute vorticity from velocity field
    pub fn compute_vorticity(&mut self) {
        let dx_inv = 1.0 / self.spacing;

        for j in 1..self.ny - 1 {
            for i in 1..self.nx - 1 {
                let idx = self.index(i, j);
                let idx_xp = self.index(i + 1, j);
                let idx_xm = self.index(i - 1, j);
                let idx_yp = self.index(i, j + 1);
                let idx_ym = self.index(i, j - 1);

                // ω = ∂v/∂x - ∂u/∂y
                let dvdx = (self.velocity[idx_xp].y - self.velocity[idx_xm].y) * 0.5 * dx_inv;
                let dudy = (self.velocity[idx_yp].x - self.velocity[idx_ym].x) * 0.5 * dx_inv;

                self.vorticity[idx] = dvdx - dudy;
            }
        }
    }

    /// Add random perturbations for turbulence onset
    pub fn add_perturbations(&mut self, amplitude: f32) {
        use std::f32::consts::PI;

        for j in 1..self.ny - 1 {
            for i in 1..self.nx - 1 {
                let idx = self.index(i, j);

                // Pseudo-random based on position
                let x = i as f32 / self.nx as f32;
                let y = j as f32 / self.ny as f32;

                // Multiple frequency perturbations
                let perturb_x = amplitude * (
                    (x * 20.0 * PI).sin() * (y * 15.0 * PI).cos() +
                    0.5 * (x * 35.0 * PI).cos() * (y * 25.0 * PI).sin()
                );
                let perturb_y = amplitude * (
                    (x * 18.0 * PI).cos() * (y * 22.0 * PI).sin() +
                    0.5 * (x * 30.0 * PI).sin() * (y * 28.0 * PI).cos()
                );

                self.velocity[idx] += Vec2::new(perturb_x, perturb_y);
            }
        }

        self.compute_vorticity();
    }

    /// Time step the fluid simulation
    pub fn step(&mut self) {
        // 1. Advection (semi-Lagrangian)
        self.advect();
        self.advect_charge();  // Also advect charge density

        // 2. Diffusion (viscosity)
        self.diffuse();

        // 3. Projection (enforce incompressibility)
        self.project();

        // 4. Update vorticity
        self.compute_vorticity();
    }

    /// Advection step using semi-Lagrangian method
    fn advect(&mut self) {
        let mut new_velocity = self.velocity.clone();

        for j in 1..self.ny - 1 {
            for i in 1..self.nx - 1 {
                let idx = self.index(i, j);
                let pos = Vec2::new(i as f32, j as f32);

                // Trace particle backward in time
                let vel = self.velocity[idx];
                let back_pos = pos - vel * self.dt / self.spacing;

                // Bilinear interpolation
                new_velocity[idx] = self.interpolate_velocity(back_pos);
            }
        }

        self.velocity = new_velocity;
    }

    /// Advect charge density with the fluid
    fn advect_charge(&mut self) {
        let mut new_charge = self.charge_density.clone();

        for j in 1..self.ny - 1 {
            for i in 1..self.nx - 1 {
                let idx = self.index(i, j);
                let pos = Vec2::new(i as f32, j as f32);

                // Trace particle backward in time
                let vel = self.velocity[idx];
                let back_pos = pos - vel * self.dt / self.spacing;

                // Bilinear interpolation of charge density
                new_charge[idx] = self.interpolate_scalar(back_pos, &self.charge_density);
            }
        }

        self.charge_density = new_charge;
    }

    /// Interpolate velocity at arbitrary position using bilinear interpolation
    fn interpolate_velocity(&self, pos: Vec2) -> Vec2 {
        let i = pos.x.floor().max(0.0).min(self.nx as f32 - 2.0) as usize;
        let j = pos.y.floor().max(0.0).min(self.ny as f32 - 2.0) as usize;

        let fx = pos.x - i as f32;
        let fy = pos.y - j as f32;

        let idx00 = self.index(i, j);
        let idx10 = self.index(i + 1, j);
        let idx01 = self.index(i, j + 1);
        let idx11 = self.index(i + 1, j + 1);

        // Bilinear interpolation
        let v00 = self.velocity[idx00];
        let v10 = self.velocity[idx10];
        let v01 = self.velocity[idx01];
        let v11 = self.velocity[idx11];

        let v0 = v00.lerp(v10, fx);
        let v1 = v01.lerp(v11, fx);
        v0.lerp(v1, fy)
    }

    /// Interpolate scalar field at arbitrary position using bilinear interpolation
    fn interpolate_scalar(&self, pos: Vec2, field: &[f32]) -> f32 {
        let i = pos.x.floor().max(0.0).min(self.nx as f32 - 2.0) as usize;
        let j = pos.y.floor().max(0.0).min(self.ny as f32 - 2.0) as usize;

        let fx = pos.x - i as f32;
        let fy = pos.y - j as f32;

        let idx00 = self.index(i, j);
        let idx10 = self.index(i + 1, j);
        let idx01 = self.index(i, j + 1);
        let idx11 = self.index(i + 1, j + 1);

        // Bilinear interpolation
        let s00 = field[idx00];
        let s10 = field[idx10];
        let s01 = field[idx01];
        let s11 = field[idx11];

        let s0 = s00 * (1.0 - fx) + s10 * fx;
        let s1 = s01 * (1.0 - fx) + s11 * fx;
        s0 * (1.0 - fy) + s1 * fy
    }

    /// Diffusion step (viscosity)
    fn diffuse(&mut self) {
        if self.viscosity < 1e-8 {
            return; // Skip if inviscid
        }

        let alpha = self.dt * self.viscosity / (self.spacing * self.spacing);
        let beta = 1.0 / (1.0 + 4.0 * alpha);

        // Gauss-Seidel relaxation (5 iterations)
        for _ in 0..5 {
            let mut new_velocity = self.velocity.clone();

            for j in 1..self.ny - 1 {
                for i in 1..self.nx - 1 {
                    let idx = self.index(i, j);
                    let idx_xp = self.index(i + 1, j);
                    let idx_xm = self.index(i - 1, j);
                    let idx_yp = self.index(i, j + 1);
                    let idx_ym = self.index(i, j - 1);

                    let laplacian = self.velocity[idx_xp] + self.velocity[idx_xm] +
                                   self.velocity[idx_yp] + self.velocity[idx_ym];

                    new_velocity[idx] = beta * (self.velocity[idx] + alpha * laplacian);
                }
            }

            self.velocity = new_velocity;
        }
    }

    /// Projection step (enforce incompressibility: ∇·v = 0)
    fn project(&mut self) {
        let dx_inv = 1.0 / self.spacing;
        let mut divergence = vec![0.0; self.nx * self.ny];

        // Compute divergence
        for j in 1..self.ny - 1 {
            for i in 1..self.nx - 1 {
                let idx = self.index(i, j);
                let idx_xp = self.index(i + 1, j);
                let idx_xm = self.index(i - 1, j);
                let idx_yp = self.index(i, j + 1);
                let idx_ym = self.index(i, j - 1);

                divergence[idx] = 0.5 * dx_inv * (
                    self.velocity[idx_xp].x - self.velocity[idx_xm].x +
                    self.velocity[idx_yp].y - self.velocity[idx_ym].y
                );
            }
        }

        // Solve for pressure using Jacobi iteration
        // ∇²p = ∇·v discretizes to:
        // (p[i+1] + p[i-1] + p[j+1] + p[j-1] - 4*p[i,j]) / dx² = div
        // Solving: p[i,j] = (p[i+1] + p[i-1] + p[j+1] + p[j-1] - div*dx²) / 4
        self.pressure.fill(0.0);
        let dx_squared = self.spacing * self.spacing;

        for _ in 0..20 {
            let mut new_pressure = self.pressure.clone();

            for j in 1..self.ny - 1 {
                for i in 1..self.nx - 1 {
                    let idx = self.index(i, j);
                    let idx_xp = self.index(i + 1, j);
                    let idx_xm = self.index(i - 1, j);
                    let idx_yp = self.index(i, j + 1);
                    let idx_ym = self.index(i, j - 1);

                    new_pressure[idx] = (
                        self.pressure[idx_xp] + self.pressure[idx_xm] +
                        self.pressure[idx_yp] + self.pressure[idx_ym] -
                        divergence[idx] * dx_squared
                    ) * 0.25;
                }
            }

            self.pressure = new_pressure;
        }

        // Subtract pressure gradient from velocity
        for j in 1..self.ny - 1 {
            for i in 1..self.nx - 1 {
                let idx = self.index(i, j);
                let idx_xp = self.index(i + 1, j);
                let idx_xm = self.index(i - 1, j);
                let idx_yp = self.index(i, j + 1);
                let idx_ym = self.index(i, j - 1);

                let grad_p = Vec2::new(
                    (self.pressure[idx_xp] - self.pressure[idx_xm]) * 0.5 * dx_inv,
                    (self.pressure[idx_yp] - self.pressure[idx_ym]) * 0.5 * dx_inv,
                );

                self.velocity[idx] -= grad_p;
            }
        }
    }

    /// Get maximum vorticity (for visualization scaling)
    pub fn max_vorticity(&self) -> f32 {
        self.vorticity.iter().map(|&w| w.abs()).fold(0.0, f32::max)
    }

    /// Get maximum velocity magnitude
    pub fn max_velocity(&self) -> f32 {
        self.velocity.iter().map(|v| v.length()).fold(0.0, f32::max)
    }

    /// Get kinetic energy
    pub fn kinetic_energy(&self) -> f32 {
        let cell_volume = self.spacing * self.spacing;
        self.velocity
            .iter()
            .map(|v| 0.5 * v.length_squared() * cell_volume)
            .sum()
    }

    /// Get enstrophy (integrated vorticity squared - measure of turbulence intensity)
    pub fn enstrophy(&self) -> f32 {
        let cell_volume = self.spacing * self.spacing;
        self.vorticity
            .iter()
            .map(|&w| 0.5 * w * w * cell_volume)
            .sum()
    }

    /// Add uniform charge density in a circular region (for Faraday disk)
    pub fn add_charge_in_disk(&mut self, center: Vec2, radius: f32, charge_density: f32) {
        for j in 0..self.ny {
            for i in 0..self.nx {
                let pos = Vec2::new(i as f32 * self.spacing, j as f32 * self.spacing);
                let r = (pos - center).length();

                if r < radius {
                    let idx = self.index(i, j);
                    self.charge_density[idx] = charge_density;
                }
            }
        }
    }

    /// Compute electric field from pressure gradient
    /// In Martins' theory: E ∝ -∇p (pressure gradient drives electric field)
    pub fn compute_e_field_from_pressure(&mut self, coupling_constant: f32) {
        let dx_inv = 1.0 / self.spacing;

        for j in 1..self.ny - 1 {
            for i in 1..self.nx - 1 {
                let idx = self.index(i, j);
                let idx_xp = self.index(i + 1, j);
                let idx_xm = self.index(i - 1, j);
                let idx_yp = self.index(i, j + 1);
                let idx_ym = self.index(i, j - 1);

                // E = -k * ∇p (negative pressure gradient)
                let grad_p = Vec2::new(
                    (self.pressure[idx_xp] - self.pressure[idx_xm]) * 0.5 * dx_inv,
                    (self.pressure[idx_yp] - self.pressure[idx_ym]) * 0.5 * dx_inv,
                );

                self.e_field[idx] = -coupling_constant * grad_p;
            }
        }
    }

    /// Get radial electric field at a given radius from center
    pub fn radial_e_field(&self, center: Vec2, radius: f32) -> f32 {
        let mut e_radial_sum = 0.0;
        let mut count = 0;

        for j in 0..self.ny {
            for i in 0..self.nx {
                let pos = Vec2::new(i as f32 * self.spacing, j as f32 * self.spacing);
                let r_vec = pos - center;
                let r = r_vec.length();

                // Sample points near the desired radius
                if (r - radius).abs() < self.spacing * 0.5 {
                    let idx = self.index(i, j);
                    let radial_dir = r_vec.normalize_or_zero();
                    let e_radial = self.e_field[idx].dot(radial_dir);

                    e_radial_sum += e_radial;
                    count += 1;
                }
            }
        }

        if count > 0 {
            e_radial_sum / count as f32
        } else {
            0.0
        }
    }

    /// Get maximum E-field magnitude
    pub fn max_e_field(&self) -> f32 {
        self.e_field.iter().map(|e| e.length()).fold(0.0, f32::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fluid_creation() {
        let fluid = Fluid2D::new(64, 64, 0.01, 1e-4, 0.001);
        assert_eq!(fluid.nx, 64);
        assert_eq!(fluid.velocity.len(), 64 * 64);
    }

    #[test]
    fn test_add_vortex() {
        let mut fluid = Fluid2D::new(64, 64, 0.01, 1e-4, 0.001);
        let center = Vec2::new(0.32, 0.32);

        fluid.add_vortex(center, 1.0, 0.05);

        // Should have non-zero velocity
        let max_vel = fluid.max_velocity();
        assert!(max_vel > 0.0);
    }

    #[test]
    fn test_vorticity_computation() {
        let mut fluid = Fluid2D::new(64, 64, 0.01, 1e-4, 0.001);
        fluid.add_vortex(Vec2::new(0.32, 0.32), 1.0, 0.05);

        let max_vort = fluid.max_vorticity();
        assert!(max_vort > 0.0);
    }

    #[test]
    fn test_step_detailed() {
        let mut fluid = Fluid2D::new(32, 32, 0.01, 1e-4, 0.001);
        fluid.add_vortex(Vec2::new(0.16, 0.16), 1.0, 0.05);

        println!("Initial: E={:.3e}, max_v={:.3e}", fluid.kinetic_energy(), fluid.max_velocity());

        // Test each step separately
        fluid.advect();
        println!("After advect: E={:.3e}, max_v={:.3e}", fluid.kinetic_energy(), fluid.max_velocity());

        fluid.diffuse();
        println!("After diffuse: E={:.3e}, max_v={:.3e}", fluid.kinetic_energy(), fluid.max_velocity());

        fluid.project();
        println!("After project: E={:.3e}, max_v={:.3e}", fluid.kinetic_energy(), fluid.max_velocity());

        fluid.compute_vorticity();
        println!("After vorticity: E={:.3e}, max_v={:.3e}", fluid.kinetic_energy(), fluid.max_velocity());
    }

    #[test]
    fn test_step() {
        let mut fluid = Fluid2D::new(32, 32, 0.01, 1e-4, 0.001);
        fluid.add_vortex(Vec2::new(0.16, 0.16), 1.0, 0.05);

        let energy_before = fluid.kinetic_energy();
        println!("Energy before: {}", energy_before);

        // Step the simulation
        for i in 0..10 {
            fluid.step();
            let e = fluid.kinetic_energy();
            let max_v = fluid.max_velocity();
            println!("Step {}: E={}, max_v={}", i+1, e, max_v);
        }

        let energy_after = fluid.kinetic_energy();
        println!("Energy after: {}", energy_after);

        // Energy should decay due to viscosity
        assert!(energy_after < energy_before,
                "Energy increased! Before: {}, After: {}", energy_before, energy_after);
    }
}
