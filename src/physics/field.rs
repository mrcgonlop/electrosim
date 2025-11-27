//! Field data structures for electromagnetic and scalar fields

use glam::Vec3;
use bytemuck::{Pod, Zeroable};

/// Represents an electromagnetic field at a point in space
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct EMField {
    /// Electric field vector (V/m)
    pub electric: Vec3,
    /// Magnetic field vector (T)
    pub magnetic: Vec3,
}

impl EMField {
    /// Create a new EM field with given electric and magnetic components
    ///
    /// # Example
    /// ```
    /// use em_physics_sandbox::physics::EMField;
    /// use glam::Vec3;
    ///
    /// let field = EMField::new(
    ///     Vec3::new(1.0, 0.0, 0.0),
    ///     Vec3::new(0.0, 1.0, 0.0)
    /// );
    /// ```
    pub fn new(electric: Vec3, magnetic: Vec3) -> Self {
        Self { electric, magnetic }
    }

    /// Create a zero field
    pub fn zero() -> Self {
        Self {
            electric: Vec3::ZERO,
            magnetic: Vec3::ZERO,
        }
    }

    /// Calculate the energy density of the field (J/m³)
    /// U = (ε₀/2)|E|² + (1/2μ₀)|B|²
    pub fn energy_density(&self) -> f32 {
        const EPSILON_0: f32 = 8.854e-12; // F/m
        const MU_0: f32 = 1.257e-6; // H/m

        let electric_energy = 0.5 * EPSILON_0 * self.electric.length_squared();
        let magnetic_energy = 0.5 / MU_0 * self.magnetic.length_squared();

        electric_energy + magnetic_energy
    }

    /// Calculate the Poynting vector S = (1/μ₀) E × B (W/m²)
    pub fn poynting_vector(&self) -> Vec3 {
        const MU_0: f32 = 1.257e-6;
        self.electric.cross(self.magnetic) / MU_0
    }
}

impl Default for EMField {
    fn default() -> Self {
        Self::zero()
    }
}

/// Scalar field for 2D wave equation simulations
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct ScalarField {
    /// Current field value
    pub value: f32,
    /// Time derivative of field value
    pub velocity: f32,
}

impl ScalarField {
    /// Create a new scalar field with given value and velocity
    pub fn new(value: f32, velocity: f32) -> Self {
        Self { value, velocity }
    }

    /// Create a zero scalar field
    pub fn zero() -> Self {
        Self {
            value: 0.0,
            velocity: 0.0,
        }
    }

    /// Calculate the energy of this field element
    /// E = (1/2)v² + (1/2)u² (kinetic + potential)
    pub fn energy(&self) -> f32 {
        0.5 * (self.velocity.powi(2) + self.value.powi(2))
    }
}

impl Default for ScalarField {
    fn default() -> Self {
        Self::zero()
    }
}

// Implement Pod and Zeroable for GPU usage
unsafe impl Pod for ScalarField {}
unsafe impl Zeroable for ScalarField {}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_em_field_creation() {
        let field = EMField::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0)
        );
        assert_eq!(field.electric, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(field.magnetic, Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn test_em_field_zero() {
        let field = EMField::zero();
        assert_eq!(field.electric, Vec3::ZERO);
        assert_eq!(field.magnetic, Vec3::ZERO);
    }

    #[test]
    fn test_energy_density() {
        let field = EMField::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0)
        );
        let energy = field.energy_density();
        assert!(energy > 0.0, "Energy density should be positive");
    }

    #[test]
    fn test_poynting_vector() {
        // E in x direction, B in y direction -> S in z direction
        let field = EMField::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0)
        );
        let poynting = field.poynting_vector();

        // Should point in z direction
        assert!(poynting.z > 0.0);
        assert_relative_eq!(poynting.x, 0.0, epsilon = 1e-6);
        assert_relative_eq!(poynting.y, 0.0, epsilon = 1e-6);
    }

    #[test]
    fn test_scalar_field() {
        let field = ScalarField::new(2.0, 3.0);
        assert_eq!(field.value, 2.0);
        assert_eq!(field.velocity, 3.0);

        let energy = field.energy();
        assert_relative_eq!(energy, 0.5 * (4.0 + 9.0), epsilon = 1e-6);
    }
}
