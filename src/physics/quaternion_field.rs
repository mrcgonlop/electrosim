//! Quaternion Dimensional Field: Maxwell's Original Formulation
//!
//! HISTORICAL CONTEXT:
//! ===================
//! Maxwell (1865) originally formulated electromagnetism using QUATERNIONS!
//! - 20 equations in quaternion form
//! - Potentials (A⃗, φ) as primary fields
//! - Single unified equation
//!
//! Heaviside (1885) simplified to 4 vector equations:
//! - Easier to use for engineering
//! - But DISCARDED important physics:
//!   * Longitudinal/scalar waves
//!   * Quaternion structure
//!   * Potentials as fundamental
//!
//! QUATERNION FORMULATION:
//! =======================
//! Electromagnetic quaternion:
//!   Q = φ + iA_x + jA_y + kA_z
//!
//! where:
//!   φ = scalar potential (real part)
//!   A⃗ = (A_x, A_y, A_z) = vector potential (imaginary parts)
//!   i, j, k = quaternion units (like complex i, but 3 of them!)
//!
//! MAXWELL'S QUATERNION EQUATION:
//! ==============================
//! Single equation containing ALL of electromagnetism:
//!
//!   ∇_q Q = S
//!
//! where:
//!   ∇_q = quaternion gradient operator
//!   S = sources (charge + current density)
//!
//! Expanding this ONE equation gives:
//!   - All 4 Maxwell equations
//!   - Plus additional scalar/longitudinal terms!
//!
//! DIMENSIONAL INTERPRETATION:
//! ===========================
//! In our framework:
//!   Q = dimensional quaternion field
//!   φ ~ d (dimensional level)
//!   A⃗ ~ ∫∇d (dimensional circulation)
//!
//! Wave equation:
//!   □Q = sources  (d'Alembertian operator)
//!
//! This naturally produces BOTH:
//!   - Transverse waves (E⊥B⊥k, standard EM)
//!   - Longitudinal waves (scalar, lost in Heaviside!)

use glam::Vec3;

/// Quaternion representation: q = w + xi + yj + zk
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion {
    /// Real part (scalar component)
    pub w: f32,

    /// i component (x-direction)
    pub x: f32,

    /// j component (y-direction)
    pub y: f32,

    /// k component (z-direction)
    pub z: f32,
}

impl Quaternion {
    /// Create new quaternion
    pub fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self { w, x, y, z }
    }

    /// Create from scalar and vector parts
    pub fn from_scalar_vector(scalar: f32, vector: Vec3) -> Self {
        Self {
            w: scalar,
            x: vector.x,
            y: vector.y,
            z: vector.z,
        }
    }

    /// Zero quaternion
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Identity quaternion (1 + 0i + 0j + 0k)
    pub fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 0.0)
    }

    /// Scalar part (real component)
    pub fn scalar(&self) -> f32 {
        self.w
    }

    /// Vector part (imaginary components)
    pub fn vector(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    /// Magnitude (norm)
    pub fn magnitude(&self) -> f32 {
        (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Conjugate: q* = w - xi - yj - zk
    pub fn conjugate(&self) -> Self {
        Self::new(self.w, -self.x, -self.y, -self.z)
    }

    /// Normalize to unit quaternion
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag < 1e-10 {
            return Self::identity();
        }
        Self::new(
            self.w / mag,
            self.x / mag,
            self.y / mag,
            self.z / mag,
        )
    }

    /// Quaternion multiplication: q1 * q2
    /// NOT commutative! (q1*q2 ≠ q2*q1 in general)
    pub fn multiply(&self, other: &Quaternion) -> Self {
        // Hamilton's quaternion multiplication rules:
        // i² = j² = k² = ijk = -1
        // ij = k, jk = i, ki = j
        // ji = -k, kj = -i, ik = -j

        let w = self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z;
        let x = self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y;
        let y = self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x;
        let z = self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w;

        Self::new(w, x, y, z)
    }

    /// Quaternion addition
    pub fn add(&self, other: &Quaternion) -> Self {
        Self::new(
            self.w + other.w,
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
        )
    }

    /// Scalar multiplication
    pub fn scale(&self, scalar: f32) -> Self {
        Self::new(
            self.w * scalar,
            self.x * scalar,
            self.y * scalar,
            self.z * scalar,
        )
    }
}

/// Electromagnetic quaternion field: Q = φ + iA_x + jA_y + kA_z
#[derive(Clone, Copy, Debug)]
pub struct EMQuaternion {
    /// Electromagnetic potential as quaternion
    pub potential: Quaternion,
}

impl EMQuaternion {
    /// Create from potentials
    pub fn from_potentials(scalar_potential: f32, vector_potential: Vec3) -> Self {
        Self {
            potential: Quaternion::from_scalar_vector(scalar_potential, vector_potential),
        }
    }

    /// Scalar potential φ
    pub fn scalar_potential(&self) -> f32 {
        self.potential.scalar()
    }

    /// Vector potential A⃗
    pub fn vector_potential(&self) -> Vec3 {
        self.potential.vector()
    }

    /// Electric field: E⃗ = -∇φ - ∂A⃗/∂t
    pub fn electric_field(&self, grad_phi: Vec3, d_a_dt: Vec3) -> Vec3 {
        -grad_phi - d_a_dt
    }

    /// Magnetic field: B⃗ = ∇×A⃗
    pub fn magnetic_field(&self, curl_a: Vec3) -> Vec3 {
        curl_a
    }
}

/// Quaternion gradient operator: ∇_q = ∂/∂t + i∂/∂x + j∂/∂y + k∂/∂z
#[derive(Clone, Copy, Debug)]
pub struct QuaternionGradient {
    /// Time derivative
    pub dt: f32,

    /// Spatial derivatives
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

impl QuaternionGradient {
    /// Create quaternion gradient
    pub fn new(dt: f32, dx: f32, dy: f32, dz: f32) -> Self {
        Self { dt, dx, dy, dz }
    }

    /// Apply to quaternion field
    pub fn apply(&self, field: &Quaternion) -> Quaternion {
        // ∇_q Q = (∂/∂t + i∂/∂x + j∂/∂y + k∂/∂z) Q

        // Scalar part: ∂φ/∂t (from time derivative of scalar)
        let scalar = self.dt * field.w;

        // Vector parts from spatial derivatives
        let x = self.dx * field.w + self.dt * field.x;
        let y = self.dy * field.w + self.dt * field.y;
        let z = self.dz * field.w + self.dt * field.z;

        Quaternion::new(scalar, x, y, z)
    }
}

/// Quaternion dimensional field
/// Combines dimensional field d(x,t) with vector potential A⃗(x,t)
#[derive(Clone, Debug)]
pub struct QuaternionDimensionalField {
    /// Grid dimensions
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,

    /// Spatial resolution
    pub spacing: f32,

    /// Quaternion field at each grid point: Q = φ + iA_x + jA_y + kA_z
    pub field: Vec<Quaternion>,

    /// Time derivative of field
    pub field_velocity: Vec<Quaternion>,

    /// Background dimension (typically 3.0)
    pub background_dimension: f32,

    /// Current time
    pub time: f32,

    /// Wave speed
    pub wave_speed: f32,
}

impl QuaternionDimensionalField {
    /// Create new quaternion field
    pub fn new(nx: usize, ny: usize, nz: usize, spacing: f32) -> Self {
        let num_cells = nx * ny * nz;

        Self {
            nx,
            ny,
            nz,
            spacing,
            field: vec![Quaternion::from_scalar_vector(3.0, Vec3::ZERO); num_cells],
            field_velocity: vec![Quaternion::zero(); num_cells],
            background_dimension: 3.0,
            time: 0.0,
            wave_speed: 1.0,  // c = 1 in natural units
        }
    }

    /// Get cell index from 3D coordinates
    pub fn cell_index(&self, i: usize, j: usize, k: usize) -> usize {
        i + self.nx * (j + self.ny * k)
    }

    /// Get scalar potential at index
    pub fn scalar_potential(&self, idx: usize) -> f32 {
        self.field[idx].scalar()
    }

    /// Get vector potential at index
    pub fn vector_potential(&self, idx: usize) -> Vec3 {
        self.field[idx].vector()
    }

    /// Set electromagnetic quaternion at position
    pub fn set_em_quaternion(&mut self, idx: usize, phi: f32, a: Vec3) {
        self.field[idx] = Quaternion::from_scalar_vector(phi, a);
    }

    /// Quaternion Laplacian: ∇²Q
    pub fn laplacian(&self, idx: usize) -> Quaternion {
        let i = idx % self.nx;
        let j = (idx / self.nx) % self.ny;
        let k = idx / (self.nx * self.ny);

        let q_center = self.field[idx];
        let dx = self.spacing;
        let dx2 = dx * dx;

        let mut lap = Quaternion::zero();

        // X direction
        if i > 0 && i < self.nx - 1 {
            let q_left = self.field[self.cell_index(i - 1, j, k)];
            let q_right = self.field[self.cell_index(i + 1, j, k)];

            let lap_x = (q_left.add(&q_right).add(&q_center.scale(-2.0))).scale(1.0 / dx2);
            lap = lap.add(&lap_x);
        }

        // Y direction
        if j > 0 && j < self.ny - 1 {
            let q_down = self.field[self.cell_index(i, j - 1, k)];
            let q_up = self.field[self.cell_index(i, j + 1, k)];

            let lap_y = (q_down.add(&q_up).add(&q_center.scale(-2.0))).scale(1.0 / dx2);
            lap = lap.add(&lap_y);
        }

        // Z direction
        if k > 0 && k < self.nz - 1 {
            let q_back = self.field[self.cell_index(i, j, k - 1)];
            let q_front = self.field[self.cell_index(i, j, k + 1)];

            let lap_z = (q_back.add(&q_front).add(&q_center.scale(-2.0))).scale(1.0 / dx2);
            lap = lap.add(&lap_z);
        }

        lap
    }

    /// Evolve quaternion field: ∂²Q/∂t² = c²∇²Q + sources
    pub fn evolve_step(&mut self, dt: f32, sources: Option<&[Quaternion]>) {
        let c2 = self.wave_speed * self.wave_speed;

        // Compute accelerations
        let mut accelerations = Vec::with_capacity(self.field.len());

        for idx in 0..self.field.len() {
            let laplacian = self.laplacian(idx);

            // Source term
            let source = if let Some(srcs) = sources {
                srcs[idx]
            } else {
                Quaternion::zero()
            };

            // Wave equation: ∂²Q/∂t² = c²∇²Q + S
            let accel = laplacian.scale(c2).add(&source);
            accelerations.push(accel);
        }

        // Update velocities and positions (Verlet integration)
        for idx in 0..self.field.len() {
            // Update velocity: ∂Q/∂t += acceleration * dt
            self.field_velocity[idx] = self.field_velocity[idx].add(&accelerations[idx].scale(dt));

            // Update field: Q += velocity * dt
            self.field[idx] = self.field[idx].add(&self.field_velocity[idx].scale(dt));
        }

        self.time += dt;
    }

    /// Extract electric and magnetic fields from quaternion field
    pub fn extract_em_fields(&self, idx: usize) -> (Vec3, Vec3) {
        let i = idx % self.nx;
        let j = (idx / self.nx) % self.ny;
        let k = idx / (self.nx * self.ny);

        let dx = self.spacing;

        // Scalar potential
        let phi = self.field[idx].scalar();

        // Vector potential
        let a = self.field[idx].vector();

        // Time derivative of vector potential
        let da_dt = self.field_velocity[idx].vector();

        // Gradient of scalar potential: ∇φ
        let mut grad_phi = Vec3::ZERO;

        if i > 0 && i < self.nx - 1 {
            let phi_left = self.field[self.cell_index(i - 1, j, k)].scalar();
            let phi_right = self.field[self.cell_index(i + 1, j, k)].scalar();
            grad_phi.x = (phi_right - phi_left) / (2.0 * dx);
        }

        if j > 0 && j < self.ny - 1 {
            let phi_down = self.field[self.cell_index(i, j - 1, k)].scalar();
            let phi_up = self.field[self.cell_index(i, j + 1, k)].scalar();
            grad_phi.y = (phi_up - phi_down) / (2.0 * dx);
        }

        if k > 0 && k < self.nz - 1 {
            let phi_back = self.field[self.cell_index(i, j, k - 1)].scalar();
            let phi_front = self.field[self.cell_index(i, j, k + 1)].scalar();
            grad_phi.z = (phi_front - phi_back) / (2.0 * dx);
        }

        // Curl of vector potential: ∇×A⃗
        let mut curl_a = Vec3::ZERO;

        // ∂A_z/∂y - ∂A_y/∂z
        if j > 0 && j < self.ny - 1 && k > 0 && k < self.nz - 1 {
            let a_y_up = self.field[self.cell_index(i, j + 1, k)].vector().y;
            let a_y_down = self.field[self.cell_index(i, j - 1, k)].vector().y;
            let a_z_front = self.field[self.cell_index(i, j, k + 1)].vector().z;
            let a_z_back = self.field[self.cell_index(i, j, k - 1)].vector().z;

            curl_a.x = (a_z_front - a_z_back) / (2.0 * dx) - (a_y_up - a_y_down) / (2.0 * dx);
        }

        // Similar for y and z components
        if i > 0 && i < self.nx - 1 && k > 0 && k < self.nz - 1 {
            let a_z_right = self.field[self.cell_index(i + 1, j, k)].vector().z;
            let a_z_left = self.field[self.cell_index(i - 1, j, k)].vector().z;
            let a_x_front = self.field[self.cell_index(i, j, k + 1)].vector().x;
            let a_x_back = self.field[self.cell_index(i, j, k - 1)].vector().x;

            curl_a.y = (a_x_front - a_x_back) / (2.0 * dx) - (a_z_right - a_z_left) / (2.0 * dx);
        }

        if i > 0 && i < self.nx - 1 && j > 0 && j < self.ny - 1 {
            let a_y_right = self.field[self.cell_index(i + 1, j, k)].vector().y;
            let a_y_left = self.field[self.cell_index(i - 1, j, k)].vector().y;
            let a_x_up = self.field[self.cell_index(i, j + 1, k)].vector().x;
            let a_x_down = self.field[self.cell_index(i, j - 1, k)].vector().x;

            curl_a.z = (a_y_right - a_y_left) / (2.0 * dx) - (a_x_up - a_x_down) / (2.0 * dx);
        }

        // Electric field: E⃗ = -∇φ - ∂A⃗/∂t
        let e_field = -grad_phi - da_dt;

        // Magnetic field: B⃗ = ∇×A⃗
        let b_field = curl_a;

        (e_field, b_field)
    }

    /// Add oscillating source (both scalar and vector components)
    pub fn add_oscillating_source(
        &mut self,
        position: Vec3,
        scalar_amplitude: f32,
        vector_amplitude: Vec3,
        frequency: f32,
    ) {
        let phase = 2.0 * std::f32::consts::PI * frequency * self.time;
        let value = phase.sin();

        let i = (position.x / self.spacing).round() as usize;
        let j = (position.y / self.spacing).round() as usize;
        let k = (position.z / self.spacing).round() as usize;

        if i < self.nx && j < self.ny && k < self.nz {
            let idx = self.cell_index(i, j, k);

            // Oscillating quaternion source
            let phi = self.background_dimension + scalar_amplitude * value;
            let a = vector_amplitude * value;

            self.field[idx] = Quaternion::from_scalar_vector(phi, a);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quaternion_multiplication() {
        // i² = -1
        let i = Quaternion::new(0.0, 1.0, 0.0, 0.0);
        let i_squared = i.multiply(&i);
        assert!((i_squared.w + 1.0).abs() < 1e-6);  // Should be -1

        // ij = k
        let j = Quaternion::new(0.0, 0.0, 1.0, 0.0);
        let ij = i.multiply(&j);
        assert!((ij.z - 1.0).abs() < 1e-6);  // Should be k

        // ji = -k (not commutative!)
        let ji = j.multiply(&i);
        assert!((ji.z + 1.0).abs() < 1e-6);  // Should be -k
    }

    #[test]
    fn test_em_quaternion() {
        let phi = 1.0;
        let a = Vec3::new(0.1, 0.2, 0.3);

        let em_q = EMQuaternion::from_potentials(phi, a);

        assert!((em_q.scalar_potential() - phi).abs() < 1e-6);
        assert!((em_q.vector_potential() - a).length() < 1e-6);
    }

    #[test]
    fn test_quaternion_field_evolution() {
        let mut field = QuaternionDimensionalField::new(10, 10, 10, 1.0);

        // Add perturbation
        let center_idx = field.cell_index(5, 5, 5);
        field.set_em_quaternion(center_idx, 4.0, Vec3::new(0.1, 0.0, 0.0));

        // Evolve
        for _ in 0..10 {
            field.evolve_step(0.01, None);
        }

        // Field should have propagated
        let neighbor_idx = field.cell_index(6, 5, 5);
        let neighbor_phi = field.scalar_potential(neighbor_idx);

        // Should be perturbed from background
        assert!((neighbor_phi - field.background_dimension).abs() > 0.001);
    }
}
