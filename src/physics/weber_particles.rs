//! Weber Electrodynamics - Particle-based simulation
//!
//! Implements Weber's force law for direct particle-particle interactions:
//! F = (q₁q₂/r²)[1 - (ṙ)²/(2c²) + r·r̈/c²]
//!
//! Unlike Maxwell's field theory, this is action-at-a-distance with no
//! intermediate field. Forces depend on positions, velocities, and accelerations.

use glam::Vec2;

/// Single charged particle in 2D
#[derive(Clone, Debug)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub charge: f32,
    pub mass: f32,
}

impl Particle {
    /// Create a new particle
    pub fn new(position: Vec2, velocity: Vec2, charge: f32, mass: f32) -> Self {
        Self {
            position,
            velocity,
            acceleration: Vec2::ZERO,
            charge,
            mass,
        }
    }
}

/// Weber electrodynamics simulator for multiple particles
pub struct WeberParticles {
    pub particles: Vec<Particle>,
    pub c: f32, // Speed of light (m/s)
    pub k: f32, // Coulomb constant (N⋅m²/C²)
}

impl WeberParticles {
    /// Create a new Weber particle system
    pub fn new(c: f32) -> Self {
        Self {
            particles: Vec::new(),
            c,
            k: 8.988e9, // Coulomb constant
        }
    }

    /// Add a particle to the system
    pub fn add_particle(&mut self, particle: Particle) {
        self.particles.push(particle);
    }

    /// Compute Weber force between two particles
    ///
    /// F = k·q₁q₂/r² · [1 - (dr/dt)²/(2c²) + r·(d²r/dt²)/c²]
    ///
    /// where:
    /// - r = distance between particles
    /// - dr/dt = relative radial velocity
    /// - d²r/dt² = relative radial acceleration
    fn weber_force(&self, i: usize, j: usize) -> Vec2 {
        let pi = &self.particles[i];
        let pj = &self.particles[j];

        // Separation vector: r_ij = r_j - r_i
        let r_vec = pj.position - pi.position;
        let r = r_vec.length();

        if r < 1e-10 {
            return Vec2::ZERO; // Avoid singularity
        }

        let r_hat = r_vec / r; // Unit vector

        // Relative velocity: v_ij = v_j - v_i
        let v_rel = pj.velocity - pi.velocity;

        // Relative acceleration: a_ij = a_j - a_i
        let a_rel = pj.acceleration - pi.acceleration;

        // Radial velocity: ṙ = dr/dt = r_hat · v_rel
        let r_dot = r_hat.dot(v_rel);

        // Radial acceleration: r̈ = d²r/dt² = r_hat · a_rel
        let r_ddot = r_hat.dot(a_rel);

        // Weber force law
        // F = k·q₁q₂/r² · [1 - (ṙ)²/(2c²) + r·r̈/c²]
        let coulomb_term = self.k * pi.charge * pj.charge / (r * r);
        let velocity_correction = 1.0 - (r_dot * r_dot) / (2.0 * self.c * self.c);
        let acceleration_correction = (r * r_ddot) / (self.c * self.c);

        let weber_factor = velocity_correction + acceleration_correction;

        // Force magnitude (can be attractive or repulsive)
        let f_magnitude = coulomb_term * weber_factor;

        // Force direction: along r_hat
        f_magnitude * r_hat
    }

    /// Compute total force on particle i from all other particles
    fn compute_force(&self, i: usize) -> Vec2 {
        let mut total_force = Vec2::ZERO;

        for j in 0..self.particles.len() {
            if i != j {
                total_force += self.weber_force(i, j);
            }
        }

        total_force
    }

    /// Update particle positions using Velocity Verlet integration
    ///
    /// This is a symplectic integrator that conserves energy well:
    /// 1. x(t+dt) = x(t) + v(t)·dt + 0.5·a(t)·dt²
    /// 2. Compute new accelerations a(t+dt)
    /// 3. v(t+dt) = v(t) + 0.5·[a(t) + a(t+dt)]·dt
    pub fn step(&mut self, dt: f32) {
        let n = self.particles.len();

        // Store old state
        let old_accelerations: Vec<Vec2> = self.particles.iter()
            .map(|p| p.acceleration)
            .collect();
        let old_velocities: Vec<Vec2> = self.particles.iter()
            .map(|p| p.velocity)
            .collect();

        // Step 1: Update positions using current velocities and accelerations
        for i in 0..n {
            let vel = old_velocities[i];
            let acc = old_accelerations[i];
            self.particles[i].position += vel * dt + 0.5 * acc * dt * dt;
        }

        // Step 2: Compute new accelerations at new positions
        for i in 0..n {
            let force = self.compute_force(i);
            self.particles[i].acceleration = force / self.particles[i].mass;
        }

        // Step 3: Update velocities using average of old and new accelerations
        for i in 0..n {
            let new_acc = self.particles[i].acceleration;
            self.particles[i].velocity = old_velocities[i] + 0.5 * (old_accelerations[i] + new_acc) * dt;
        }
    }

    /// Get total kinetic energy
    pub fn kinetic_energy(&self) -> f32 {
        self.particles.iter()
            .map(|p| 0.5 * p.mass * p.velocity.length_squared())
            .sum()
    }

    /// Get total potential energy (Coulomb only, not including Weber corrections)
    pub fn potential_energy(&self) -> f32 {
        let mut u = 0.0;

        for i in 0..self.particles.len() {
            for j in (i + 1)..self.particles.len() {
                let r = (self.particles[j].position - self.particles[i].position).length();
                if r > 1e-10 {
                    u += self.k * self.particles[i].charge * self.particles[j].charge / r;
                }
            }
        }

        u
    }

    /// Get total energy
    pub fn total_energy(&self) -> f32 {
        self.kinetic_energy() + self.potential_energy()
    }

    /// Get center of mass
    pub fn center_of_mass(&self) -> Vec2 {
        let total_mass: f32 = self.particles.iter().map(|p| p.mass).sum();
        let com: Vec2 = self.particles.iter()
            .map(|p| p.position * p.mass)
            .sum();

        if total_mass > 0.0 {
            com / total_mass
        } else {
            Vec2::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_creation() {
        let p = Particle::new(Vec2::ZERO, Vec2::ZERO, 1e-6, 1e-9);
        assert_eq!(p.charge, 1e-6);
        assert_eq!(p.mass, 1e-9);
    }

    #[test]
    fn test_weber_system() {
        let mut system = WeberParticles::new(3e8);

        // Two opposite charges
        system.add_particle(Particle::new(
            Vec2::new(-0.1, 0.0),
            Vec2::ZERO,
            1e-6,
            1e-9,
        ));
        system.add_particle(Particle::new(
            Vec2::new(0.1, 0.0),
            Vec2::ZERO,
            -1e-6,
            1e-9,
        ));

        let e0 = system.total_energy();

        // Step the simulation
        for _ in 0..10 {
            system.step(1e-6);
        }

        let e1 = system.total_energy();

        // Energy should be approximately conserved
        let relative_error = (e1 - e0).abs() / e0.abs();
        assert!(relative_error < 0.1, "Energy not conserved: {}", relative_error);
    }
}
