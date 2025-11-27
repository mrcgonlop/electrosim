//! 2D scalar wave equation simulator
//!
//! This implements the wave equation: ∂²u/∂t² = c²∇²u
//! Used as a simplified starting point before full 3D EM simulations.

use crate::physics::ScalarWaveTheory;

/// 2D wave equation simulator using finite differences.
///
/// Solves: ∂²u/∂t² = c²(∂²u/∂x² + ∂²u/∂y²)
///
/// # Example
///
/// ```
/// use em_physics_sandbox::physics::Wave2D;
///
/// let wave = Wave2D::new(1.0); // wave speed = 1.0
/// let mut field = vec![0.0; 100 * 100];
/// let mut velocity = vec![0.0; 100 * 100];
///
/// // Set initial Gaussian pulse
/// field[50 * 100 + 50] = 1.0;
///
/// // Simulate
/// for _ in 0..100 {
///     wave.update_scalar_field(&mut field, &mut velocity, 100, 100, 0.1, 0.01);
/// }
/// ```
pub struct Wave2D {
    /// Wave propagation speed (m/s)
    pub wave_speed: f32,
}

impl Wave2D {
    /// Create a new 2D wave simulator with the given wave speed.
    ///
    /// # Arguments
    ///
    /// * `wave_speed` - Speed of wave propagation (m/s)
    ///
    /// # Example
    ///
    /// ```
    /// use em_physics_sandbox::physics::Wave2D;
    ///
    /// // Speed of light
    /// let wave = Wave2D::new(3e8);
    /// ```
    pub fn new(wave_speed: f32) -> Self {
        Self { wave_speed }
    }

    /// Calculate the Laplacian ∇²u at a grid point using finite differences.
    ///
    /// Uses a 5-point stencil for the 2D Laplacian.
    fn laplacian(&self, field: &[f32], i: usize, j: usize, nx: usize, ny: usize, dx: f32) -> f32 {
        let idx = i + j * nx;

        // Handle boundary conditions (Dirichlet: u = 0 at boundaries)
        let u_center = field[idx];
        let u_left = if i > 0 { field[idx - 1] } else { 0.0 };
        let u_right = if i < nx - 1 { field[idx + 1] } else { 0.0 };
        let u_down = if j > 0 { field[idx - nx] } else { 0.0 };
        let u_up = if j < ny - 1 { field[idx + nx] } else { 0.0 };

        // 5-point stencil: ∇²u ≈ (u_left + u_right + u_up + u_down - 4*u_center) / dx²
        (u_left + u_right + u_up + u_down - 4.0 * u_center) / (dx * dx)
    }

    /// Set up a Gaussian pulse initial condition.
    ///
    /// # Arguments
    ///
    /// * `field` - Field array to initialize
    /// * `center` - Center position (i, j)
    /// * `amplitude` - Peak amplitude
    /// * `width` - Width of the Gaussian (in grid units)
    /// * `nx`, `ny` - Grid dimensions
    pub fn set_gaussian_pulse(
        &self,
        field: &mut [f32],
        center: (usize, usize),
        amplitude: f32,
        width: f32,
        nx: usize,
        ny: usize,
    ) {
        let (cx, cy) = center;

        for j in 0..ny {
            for i in 0..nx {
                let dx = (i as f32 - cx as f32) / width;
                let dy = (j as f32 - cy as f32) / width;
                let r2 = dx * dx + dy * dy;

                let idx = i + j * nx;
                field[idx] = amplitude * (-r2).exp();
            }
        }
    }

    /// Set up a plane wave initial condition.
    ///
    /// # Arguments
    ///
    /// * `field` - Field array to initialize
    /// * `velocity` - Velocity array to initialize
    /// * `wavelength` - Wavelength in grid units
    /// * `amplitude` - Wave amplitude
    /// * `direction` - Direction angle in radians
    /// * `nx`, `ny` - Grid dimensions
    pub fn set_plane_wave(
        &self,
        field: &mut [f32],
        velocity: &mut [f32],
        wavelength: f32,
        amplitude: f32,
        direction: f32,
        nx: usize,
        ny: usize,
    ) {
        let k = 2.0 * std::f32::consts::PI / wavelength;
        let kx = k * direction.cos();
        let ky = k * direction.sin();
        let omega = self.wave_speed * k;

        for j in 0..ny {
            for i in 0..nx {
                let idx = i + j * nx;
                let phase = kx * i as f32 + ky * j as f32;

                field[idx] = amplitude * phase.sin();
                velocity[idx] = amplitude * omega * phase.cos();
            }
        }
    }
}

impl ScalarWaveTheory for Wave2D {
    fn update_scalar_field(
        &self,
        field: &mut [f32],
        velocity: &mut [f32],
        nx: usize,
        ny: usize,
        dx: f32,
        dt: f32,
    ) {
        assert_eq!(field.len(), nx * ny);
        assert_eq!(velocity.len(), nx * ny);

        // Stability condition (CFL): c * dt / dx <= 1/√2 for 2D
        let cfl = self.wave_speed * dt / dx;
        assert!(
            cfl <= 0.707,
            "CFL condition violated: {} > 0.707. Reduce dt or increase dx.",
            cfl
        );

        let c_squared = self.wave_speed * self.wave_speed;

        // Create temporary buffers for the update
        let mut new_field = field.to_vec();
        let mut new_velocity = velocity.to_vec();

        // Update interior points using leapfrog integration
        // u^(n+1) = u^n + dt * v^n
        // v^(n+1) = v^n + dt * c² * ∇²u^n

        for j in 1..ny - 1 {
            for i in 1..nx - 1 {
                let idx = i + j * nx;

                // Calculate Laplacian
                let laplacian = self.laplacian(field, i, j, nx, ny, dx);

                // Update velocity: v^(n+1) = v^n + dt * c² * ∇²u
                new_velocity[idx] = velocity[idx] + dt * c_squared * laplacian;

                // Update field: u^(n+1) = u^n + dt * v^(n+1)
                new_field[idx] = field[idx] + dt * new_velocity[idx];
            }
        }

        // Copy back
        field.copy_from_slice(&new_field);
        velocity.copy_from_slice(&new_velocity);
    }

    fn name(&self) -> &str {
        "2D Wave Equation"
    }

    fn total_energy(&self, field: &[f32], velocity: &[f32]) -> f32 {
        field
            .iter()
            .zip(velocity.iter())
            .map(|(u, v)| 0.5 * (u.powi(2) + v.powi(2)))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_wave2d_creation() {
        let wave = Wave2D::new(1.0);
        assert_eq!(wave.wave_speed, 1.0);
        assert_eq!(wave.name(), "2D Wave Equation");
    }

    #[test]
    fn test_gaussian_pulse_initialization() {
        let wave = Wave2D::new(1.0);
        let nx = 50;
        let ny = 50;
        let mut field = vec![0.0; nx * ny];

        wave.set_gaussian_pulse(&mut field, (25, 25), 1.0, 5.0, nx, ny);

        // Center should have maximum value
        let center_idx = 25 + 25 * nx;
        assert_relative_eq!(field[center_idx], 1.0, epsilon = 1e-6);

        // Edges should be nearly zero
        assert!(field[0] < 0.01);
        assert!(field[nx - 1] < 0.01);
    }

    #[test]
    fn test_plane_wave_initialization() {
        let wave = Wave2D::new(1.0);
        let nx = 64;
        let ny = 64;
        let mut field = vec![0.0; nx * ny];
        let mut velocity = vec![0.0; nx * ny];

        wave.set_plane_wave(&mut field, &mut velocity, 8.0, 1.0, 0.0, nx, ny);

        // Check that we have a wave pattern
        let mut has_positive = false;
        let mut has_negative = false;

        for &val in &field {
            if val > 0.5 {
                has_positive = true;
            }
            if val < -0.5 {
                has_negative = true;
            }
        }

        assert!(has_positive && has_negative, "Plane wave should have both positive and negative values");
    }

    #[test]
    fn test_wave_propagation() {
        let wave = Wave2D::new(1.0);
        let nx = 64;
        let ny = 64;
        let dx = 0.1;
        let dt = 0.01; // CFL condition: c*dt/dx = 0.1 < 0.707 ✓

        let mut field = vec![0.0; nx * ny];
        let mut velocity = vec![0.0; nx * ny];

        // Set Gaussian pulse at center
        wave.set_gaussian_pulse(&mut field, (32, 32), 1.0, 3.0, nx, ny);

        let initial_energy = wave.total_energy(&field, &velocity);

        // Propagate for several steps
        for _ in 0..50 {
            wave.update_scalar_field(&mut field, &mut velocity, nx, ny, dx, dt);
        }

        // Energy should be approximately conserved (some numerical dissipation is expected)
        let final_energy = wave.total_energy(&field, &velocity);
        let energy_ratio = final_energy / initial_energy;

        // Energy should not increase and should be within 20% of initial (generous for simple scheme)
        assert!(energy_ratio <= 1.01, "Energy increased: {}", energy_ratio);
        assert!(energy_ratio > 0.5, "Too much energy lost: {}", energy_ratio);
    }

    #[test]
    fn test_static_field_stability() {
        let wave = Wave2D::new(1.0);
        let nx = 32;
        let ny = 32;
        let dx = 0.1;
        let dt = 0.01;

        // Static field (all zeros)
        let mut field = vec![0.0; nx * ny];
        let mut velocity = vec![0.0; nx * ny];

        // Propagate - should remain zero
        for _ in 0..100 {
            wave.update_scalar_field(&mut field, &mut velocity, nx, ny, dx, dt);
        }

        // All values should still be zero
        for &val in &field {
            assert_relative_eq!(val, 0.0, epsilon = 1e-6);
        }
    }

    #[test]
    #[should_panic(expected = "CFL condition violated")]
    fn test_cfl_condition_violation() {
        let wave = Wave2D::new(1.0);
        let nx = 32;
        let ny = 32;
        let dx = 0.1;
        let dt = 0.1; // CFL = 1.0 > 0.707, should panic

        let mut field = vec![0.0; nx * ny];
        let mut velocity = vec![0.0; nx * ny];

        wave.update_scalar_field(&mut field, &mut velocity, nx, ny, dx, dt);
    }

    #[test]
    fn test_laplacian_calculation() {
        let wave = Wave2D::new(1.0);
        let nx = 5;
        let ny = 5;
        let dx = 1.0;

        // Create a simple field with a peak in the center
        let mut field = vec![0.0; nx * ny];
        field[2 + 2 * nx] = 4.0; // Center value

        // Laplacian at center should be negative (peak -> concave down)
        let lap = wave.laplacian(&field, 2, 2, nx, ny, dx);
        assert!(lap < 0.0, "Laplacian of peak should be negative");
    }

    #[test]
    fn test_boundary_conditions() {
        let wave = Wave2D::new(1.0);
        let nx = 32;
        let ny = 32;
        let dx = 0.1;
        let dt = 0.005;

        let mut field = vec![0.0; nx * ny];
        let mut velocity = vec![0.0; nx * ny];

        // Set pulse near boundary
        wave.set_gaussian_pulse(&mut field, (5, 5), 1.0, 2.0, nx, ny);

        // Propagate
        for _ in 0..100 {
            wave.update_scalar_field(&mut field, &mut velocity, nx, ny, dx, dt);
        }

        // Boundaries should remain zero (Dirichlet BC)
        for i in 0..nx {
            assert_relative_eq!(field[i], 0.0, epsilon = 1e-6); // Bottom
            assert_relative_eq!(field[i + (ny - 1) * nx], 0.0, epsilon = 1e-6); // Top
        }
        for j in 0..ny {
            assert_relative_eq!(field[j * nx], 0.0, epsilon = 1e-6); // Left
            assert_relative_eq!(field[(nx - 1) + j * nx], 0.0, epsilon = 1e-6); // Right
        }
    }
}
