//! Spacetime cell - the fundamental unit of the simulation
//!
//! Each cell represents a discrete element of spacetime with geometric,
//! field, fluid, and particle properties. This unified representation
//! allows fields, fluids, and particles to emerge from the same substrate.

use glam::{Vec3, Vec4};

/// Universal spacetime cell - the fundamental computational unit
///
/// This structure represents a discrete "atom" of spacetime that can:
/// - Store electromagnetic field values (Maxwell theory)
/// - Store fluid dynamical state (velocity, density, pressure)
/// - Represent localized particles via charge/mass density
/// - Have geometric properties (metric, volume)
///
/// # Example
///
/// ```
/// use em_physics_sandbox::simulation::SpacetimeCell;
/// use glam::Vec4;
///
/// let mut cell = SpacetimeCell::default();
/// cell.position = Vec4::new(1.0, 2.0, 3.0, 0.0);
/// cell.charge_density = 1e-6;
///
/// assert!(cell.is_particle());
/// ```
#[derive(Debug, Clone)]
pub struct SpacetimeCell {
    // === GEOMETRIC PROPERTIES ===
    /// Spacetime position (x, y, z, t) in meters and seconds
    pub position: Vec4,

    /// Proper volume of this cell (m³) - can vary with metric
    pub volume: f32,

    /// Local metric scale factor (for curved spacetime effects)
    /// In flat spacetime this is 1.0
    pub metric_scale: f32,

    // === ELECTROMAGNETIC FIELD STATE ===
    /// Electric field vector (V/m)
    pub e_field: Vec3,

    /// Magnetic field vector (Tesla)
    pub b_field: Vec3,

    /// Electromagnetic 4-potential (φ, A_x, A_y, A_z)
    /// where φ is scalar potential (V) and A is vector potential (T·m)
    pub potential: Vec4,

    // === FLUID STATE ===
    /// Fluid velocity (m/s) - represents vacuum/aether flow
    pub velocity: Vec3,

    /// Mass-energy density (kg/m³) - represents vacuum density
    pub density: f32,

    /// Pressure (Pa) - thermodynamic pressure
    pub pressure: f32,

    /// Vorticity ∇×v (1/s) - curl of velocity field
    /// May be related to magnetic field in fluid EM theories
    pub vorticity: Vec3,

    /// Temperature (K) - for thermodynamic models
    pub temperature: f32,

    // === PARTICLE/CHARGE STATE ===
    /// Electric charge density (C/m³)
    /// High values indicate localized "particles"
    pub charge_density: f32,

    /// Current density J (A/m²)
    /// Related to charge motion: J = ρ_q * v
    pub current_density: Vec3,

    /// Discrete particle count (for particle-in-cell methods)
    pub particle_count: u32,

    // === MATERIAL PROPERTIES ===
    /// Electric permittivity ε (F/m)
    /// Can vary with fluid density (vacuum polarization)
    pub epsilon: f32,

    /// Magnetic permeability μ (H/m)
    /// Can vary with fluid properties
    pub mu: f32,

    /// Electric conductivity σ (S/m)
    /// For lossy media and conductors
    pub sigma: f32,
}

impl Default for SpacetimeCell {
    fn default() -> Self {
        const EPSILON_0: f32 = 8.854187817e-12; // F/m
        const MU_0: f32 = 1.25663706212e-6;     // H/m

        Self {
            // Geometric
            position: Vec4::ZERO,
            volume: 1.0,
            metric_scale: 1.0,

            // EM fields
            e_field: Vec3::ZERO,
            b_field: Vec3::ZERO,
            potential: Vec4::ZERO,

            // Fluid state
            velocity: Vec3::ZERO,
            density: 1.0,      // Nominal vacuum density
            pressure: 0.0,
            vorticity: Vec3::ZERO,
            temperature: 300.0,

            // Charge/particle
            charge_density: 0.0,
            current_density: Vec3::ZERO,
            particle_count: 0,

            // Material
            epsilon: EPSILON_0,
            mu: MU_0,
            sigma: 0.0,
        }
    }
}

impl SpacetimeCell {
    /// Create a new cell at a specific spacetime position
    pub fn at_position(position: Vec4, volume: f32) -> Self {
        let mut cell = Self::default();
        cell.position = position;
        cell.volume = volume;
        cell
    }

    /// Check if this cell contains a localized "particle"
    ///
    /// A particle is identified by high charge density relative to background.
    /// This allows particle-like behavior to emerge from field properties.
    pub fn is_particle(&self) -> bool {
        const PARTICLE_THRESHOLD: f32 = 1e-6; // C/m³
        self.charge_density.abs() > PARTICLE_THRESHOLD
    }

    /// Total energy density at this cell (J/m³)
    ///
    /// Includes electromagnetic, kinetic (fluid), and pressure contributions.
    pub fn energy_density(&self) -> f32 {
        // EM energy density: (1/2)(εE² + B²/μ)
        let em_energy = 0.5 * (
            self.epsilon * self.e_field.length_squared() +
            self.b_field.length_squared() / self.mu
        );

        // Kinetic energy density: (1/2)ρv²
        let kinetic_energy = 0.5 * self.density * self.velocity.length_squared();

        // Pressure energy (from equation of state)
        let pressure_energy = self.pressure;

        em_energy + kinetic_energy + pressure_energy
    }

    /// Total energy in this cell (J)
    pub fn total_energy(&self) -> f32 {
        self.energy_density() * self.volume
    }

    /// Poynting vector S = E×B/μ (W/m²)
    ///
    /// Represents electromagnetic energy flux.
    pub fn poynting_vector(&self) -> Vec3 {
        self.e_field.cross(self.b_field) / self.mu
    }

    /// EM momentum density (kg·m/s/m³)
    pub fn em_momentum_density(&self) -> Vec3 {
        self.poynting_vector() / (299_792_458.0_f32.powi(2))
    }

    /// Total momentum in this cell (kg·m/s)
    ///
    /// Includes both fluid mechanical and electromagnetic momentum.
    pub fn total_momentum(&self) -> Vec3 {
        let fluid_momentum = self.density * self.velocity;
        let em_momentum = self.em_momentum_density();
        (fluid_momentum + em_momentum) * self.volume
    }

    /// Update current density from charge and velocity
    ///
    /// J = ρ_q * v (charge continuity)
    pub fn update_current_density(&mut self) {
        self.current_density = self.charge_density * self.velocity;
    }

    /// Check if this cell represents a conductor
    pub fn is_conductor(&self) -> bool {
        const CONDUCTOR_THRESHOLD: f32 = 1e6; // S/m
        self.sigma > CONDUCTOR_THRESHOLD
    }

    /// Apply perfect conductor boundary condition
    ///
    /// In a conductor: E_tangential = 0, B_normal = 0
    pub fn apply_conductor_bc(&mut self) {
        self.e_field = Vec3::ZERO;
        // Keep B field normal component, zero tangential (simplified)
    }

    /// Compute local speed of light based on material properties
    ///
    /// c_local = 1/√(εμ)
    pub fn local_light_speed(&self) -> f32 {
        1.0 / (self.epsilon * self.mu).sqrt()
    }

    /// Compute local wave impedance
    ///
    /// Z = √(μ/ε)
    pub fn wave_impedance(&self) -> f32 {
        (self.mu / self.epsilon).sqrt()
    }

    /// Set this cell as a point charge
    ///
    /// Initializes charge density for a localized particle
    pub fn set_point_charge(&mut self, charge: f32) {
        self.charge_density = charge / self.volume;
        self.particle_count = 1;
    }

    /// Add a Gaussian charge distribution centered at this cell
    ///
    /// For smooth particle distributions
    pub fn add_gaussian_charge(&mut self, charge: f32, sigma: f32, distance: f32) {
        let gaussian = (-distance * distance / (2.0 * sigma * sigma)).exp();
        let normalization = 1.0 / ((2.0 * std::f32::consts::PI).powf(1.5) * sigma.powi(3));
        self.charge_density += charge * gaussian * normalization;
    }

    /// Reset all fields to vacuum state
    pub fn reset(&mut self) {
        self.e_field = Vec3::ZERO;
        self.b_field = Vec3::ZERO;
        self.potential = Vec4::ZERO;
        self.velocity = Vec3::ZERO;
        self.density = 1.0;
        self.pressure = 0.0;
        self.vorticity = Vec3::ZERO;
        self.charge_density = 0.0;
        self.current_density = Vec3::ZERO;
        self.particle_count = 0;
    }
}

// Implement common mathematical operations
impl SpacetimeCell {
    /// Linear interpolation between two cells
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let mut result = self.clone();
        result.e_field = self.e_field.lerp(other.e_field, t);
        result.b_field = self.b_field.lerp(other.b_field, t);
        result.velocity = self.velocity.lerp(other.velocity, t);
        result.density = self.density + t * (other.density - self.density);
        result.pressure = self.pressure + t * (other.pressure - self.pressure);
        result.charge_density = self.charge_density + t * (other.charge_density - self.charge_density);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cell() {
        let cell = SpacetimeCell::default();
        assert_eq!(cell.position, Vec4::ZERO);
        assert_eq!(cell.e_field, Vec3::ZERO);
        assert_eq!(cell.b_field, Vec3::ZERO);
        assert_eq!(cell.velocity, Vec3::ZERO);
        assert_eq!(cell.density, 1.0);
    }

    #[test]
    fn test_at_position() {
        let pos = Vec4::new(1.0, 2.0, 3.0, 0.0);
        let cell = SpacetimeCell::at_position(pos, 0.001);
        assert_eq!(cell.position, pos);
        assert_eq!(cell.volume, 0.001);
    }

    #[test]
    fn test_is_particle() {
        let mut cell = SpacetimeCell::default();
        assert!(!cell.is_particle());

        cell.charge_density = 1e-5;
        assert!(cell.is_particle());
    }

    #[test]
    fn test_energy_density() {
        let mut cell = SpacetimeCell::default();
        cell.e_field = Vec3::new(100.0, 0.0, 0.0);
        cell.b_field = Vec3::new(0.0, 1e-6, 0.0);

        let energy = cell.energy_density();
        assert!(energy > 0.0);
    }

    #[test]
    fn test_poynting_vector() {
        let mut cell = SpacetimeCell::default();
        cell.e_field = Vec3::new(1.0, 0.0, 0.0);
        cell.b_field = Vec3::new(0.0, 1e-6, 0.0);

        let s = cell.poynting_vector();
        // E × B should point in +z direction
        assert!(s.z > 0.0);
    }

    #[test]
    fn test_local_light_speed() {
        let cell = SpacetimeCell::default();
        let c = cell.local_light_speed();
        // Should be close to speed of light (within 1%)
        const C_EXPECTED: f32 = 299_792_458.0;
        assert!((c - C_EXPECTED).abs() / C_EXPECTED < 0.01);
    }

    #[test]
    fn test_set_point_charge() {
        let mut cell = SpacetimeCell::default();
        cell.volume = 1e-9; // 1 nm³

        cell.set_point_charge(1.6e-15); // Test charge
        // Charge density = 1.6e-15 / 1e-9 = 1.6e-6 C/m³ > threshold
        assert!(cell.is_particle());
        assert_eq!(cell.particle_count, 1);
    }

    #[test]
    fn test_update_current_density() {
        let mut cell = SpacetimeCell::default();
        cell.charge_density = 1e-6;
        cell.velocity = Vec3::new(1000.0, 0.0, 0.0);

        cell.update_current_density();
        assert_eq!(cell.current_density.x, 1e-3);
    }

    #[test]
    fn test_conductor_bc() {
        let mut cell = SpacetimeCell::default();
        cell.e_field = Vec3::new(100.0, 100.0, 100.0);
        cell.b_field = Vec3::new(1.0, 1.0, 1.0);
        cell.sigma = 1e8; // Conductor

        assert!(cell.is_conductor());
        cell.apply_conductor_bc();
        assert_eq!(cell.e_field, Vec3::ZERO);
    }

    #[test]
    fn test_total_momentum() {
        let mut cell = SpacetimeCell::default();
        cell.volume = 0.001;
        cell.density = 1.0;
        cell.velocity = Vec3::new(100.0, 0.0, 0.0);

        let momentum = cell.total_momentum();
        assert!(momentum.x > 0.0);
    }

    #[test]
    fn test_lerp() {
        let cell1 = SpacetimeCell::default();
        let mut cell2 = SpacetimeCell::default();
        cell2.e_field = Vec3::new(100.0, 0.0, 0.0);
        cell2.density = 2.0;

        let mid = cell1.lerp(&cell2, 0.5);
        assert_eq!(mid.e_field.x, 50.0);
        assert_eq!(mid.density, 1.5);
    }
}
