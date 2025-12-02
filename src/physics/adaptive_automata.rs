//! Adaptive Automata: Variable-Dimensional Cellular Automata
//!
//! Projects abstract hypergraph onto 3D space, where local dimension can vary.
//! - Base: 3D spatial grid (our observable universe)
//! - Lower dimensions (d<3): Projected as dimensional defects/strings/sheets
//! - Higher dimensions (d>3): Projected as bulges/thickness in extra dimensions
//!
//! Key insight: We experience 3D, but underlying reality may have varying dimension.
//! This framework makes dimensional transitions visible!

use crate::physics::hypergraph::{Hypergraph, NodeID};
use glam::Vec3;
use std::collections::HashMap;

/// A cell in the adaptive automata
///
/// Unlike traditional automata, each cell has a local dimension that can vary!
#[derive(Clone, Debug)]
pub struct Cell {
    /// Position in 3D space (our observable dimensions)
    pub position: Vec3,

    /// Local effective dimension (can be fractional!)
    /// d=0: Point defect (particle)
    /// d=1: String/line defect
    /// d=2: Sheet/membrane
    /// d=3: Normal 3D space
    /// d>3: Higher-dimensional bulge
    pub dimension: f32,

    /// Extra-dimensional coordinates (for d>3)
    /// These are projected onto the 3D base
    pub extra_coords: Vec<f32>,

    /// Neighbors (count depends on dimension)
    pub neighbors: Vec<usize>,

    /// Field values (charge, mass, velocity, etc.)
    pub scalars: HashMap<String, f32>,
    pub vectors: HashMap<String, Vec3>,
}

impl Cell {
    pub fn new(position: Vec3, dimension: f32) -> Self {
        Self {
            position,
            dimension,
            extra_coords: vec![0.0; (dimension.ceil() as usize).saturating_sub(3)],
            neighbors: Vec::new(),
            scalars: HashMap::new(),
            vectors: HashMap::new(),
        }
    }

    /// Number of neighbors based on local dimension
    pub fn expected_neighbor_count(&self) -> usize {
        // d=1: 2 neighbors (line)
        // d=2: 4-8 neighbors (plane)
        // d=3: 6-26 neighbors (volume)
        // d=4: ~80 neighbors (hypercube)

        if self.dimension < 1.5 {
            2  // 1D
        } else if self.dimension < 2.5 {
            8  // 2D (8-connected for smooth interpolation)
        } else if self.dimension < 3.5 {
            26  // 3D (26-connected cube)
        } else {
            // Higher dimensions: exponential growth
            (3_usize.pow(self.dimension.ceil() as u32)).min(100)
        }
    }

    /// Interaction strength falloff with distance
    /// This is dimension-dependent!
    pub fn interaction_falloff(&self, distance: f32) -> f32 {
        if distance < 1e-6 {
            return 0.0;
        }

        // Force law depends on dimension:
        // d=1: F ~ constant
        // d=2: F ~ 1/r
        // d=3: F ~ 1/r²
        // d=4: F ~ 1/r³

        let exponent = (self.dimension - 1.0).max(0.0);
        1.0 / distance.powf(exponent)
    }

    /// Volume element (measure) in this dimension
    pub fn volume_element(&self, radius: f32) -> f32 {
        // V ~ r^d
        radius.powf(self.dimension)
    }
}

/// Adaptive automata with variable local dimension
pub struct AdaptiveAutomata {
    /// Cells arranged in 3D base space
    pub cells: Vec<Cell>,

    /// Grid dimensions (base 3D structure)
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,

    /// Spatial resolution
    pub spacing: f32,

    /// Background dimension (typically 3.0)
    pub background_dimension: f32,

    /// Dimensional field statistics
    pub avg_dimension: f32,
    pub min_dimension: f32,
    pub max_dimension: f32,
}

impl AdaptiveAutomata {
    /// Create from hypergraph by projecting onto 3D
    pub fn from_hypergraph(graph: &Hypergraph, nx: usize, ny: usize, nz: usize) -> Self {
        let spacing = 1.0;
        let mut cells = Vec::new();

        // Measure dimension everywhere in graph
        let dim_radius = 5;

        // For each cell in 3D grid, find corresponding graph region and measure dimension
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let position = Vec3::new(
                        i as f32 * spacing,
                        j as f32 * spacing,
                        k as f32 * spacing,
                    );

                    // Map to graph node (for lattice graphs)
                    // For general graphs, would use proximity search
                    let graph_idx = i + nx * (j + ny * k);

                    // Measure local dimension
                    let dimension = if let Some(&node) = graph.nodes.iter().nth(graph_idx % graph.nodes.len()) {
                        graph.measure_dimension(node, dim_radius)
                    } else {
                        3.0  // Default to 3D
                    };

                    cells.push(Cell::new(position, dimension));
                }
            }
        }

        // Connect neighbors based on dimension
        let mut automata = Self {
            cells,
            nx,
            ny,
            nz,
            spacing,
            background_dimension: 3.0,
            avg_dimension: 0.0,
            min_dimension: 0.0,
            max_dimension: 0.0,
        };

        automata.update_neighbors();
        automata.update_statistics();

        automata
    }

    /// Create uniform 3D automata
    pub fn new_uniform(nx: usize, ny: usize, nz: usize, dimension: f32) -> Self {
        let spacing = 1.0;
        let mut cells = Vec::new();

        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let position = Vec3::new(
                        i as f32 * spacing,
                        j as f32 * spacing,
                        k as f32 * spacing,
                    );
                    cells.push(Cell::new(position, dimension));
                }
            }
        }

        let mut automata = Self {
            cells,
            nx,
            ny,
            nz,
            spacing,
            background_dimension: dimension,
            avg_dimension: dimension,
            min_dimension: dimension,
            max_dimension: dimension,
        };

        automata.update_neighbors();
        automata
    }

    /// Get cell index from 3D coordinates
    pub fn cell_index(&self, i: usize, j: usize, k: usize) -> usize {
        i + self.nx * (j + self.ny * k)
    }

    /// Get cell at 3D position
    pub fn cell_at(&self, i: usize, j: usize, k: usize) -> Option<&Cell> {
        if i < self.nx && j < self.ny && k < self.nz {
            Some(&self.cells[self.cell_index(i, j, k)])
        } else {
            None
        }
    }

    /// Update neighbor connectivity based on dimension
    pub fn update_neighbors(&mut self) {
        for idx in 0..self.cells.len() {
            let i = idx % self.nx;
            let j = (idx / self.nx) % self.ny;
            let k = idx / (self.nx * self.ny);

            let mut neighbors = Vec::new();
            let dimension = self.cells[idx].dimension;

            // Connect based on dimension
            if dimension < 1.5 {
                // 1D: Connect along dominant direction
                if i > 0 { neighbors.push(self.cell_index(i - 1, j, k)); }
                if i < self.nx - 1 { neighbors.push(self.cell_index(i + 1, j, k)); }
            } else if dimension < 2.5 {
                // 2D: Connect in plane (8-connected)
                for di in -1..=1_i32 {
                    for dj in -1..=1_i32 {
                        if di == 0 && dj == 0 { continue; }
                        let ni = (i as i32 + di) as usize;
                        let nj = (j as i32 + dj) as usize;
                        if ni < self.nx && nj < self.ny {
                            neighbors.push(self.cell_index(ni, nj, k));
                        }
                    }
                }
            } else {
                // 3D: Full 26-connected neighborhood
                for di in -1..=1_i32 {
                    for dj in -1..=1_i32 {
                        for dk in -1..=1_i32 {
                            if di == 0 && dj == 0 && dk == 0 { continue; }
                            let ni = (i as i32 + di) as usize;
                            let nj = (j as i32 + dj) as usize;
                            let nk = (k as i32 + dk) as usize;
                            if ni < self.nx && nj < self.ny && nk < self.nz {
                                neighbors.push(self.cell_index(ni, nj, nk));
                            }
                        }
                    }
                }
            }

            self.cells[idx].neighbors = neighbors;
        }
    }

    /// Update dimensional statistics
    pub fn update_statistics(&mut self) {
        if self.cells.is_empty() {
            return;
        }

        self.avg_dimension = self.cells.iter().map(|c| c.dimension).sum::<f32>() / self.cells.len() as f32;
        self.min_dimension = self.cells.iter().map(|c| c.dimension).fold(f32::INFINITY, f32::min);
        self.max_dimension = self.cells.iter().map(|c| c.dimension).fold(f32::NEG_INFINITY, f32::max);
    }

    /// Set dimension in a region (for creating dimensional structures)
    pub fn set_dimension_sphere(&mut self, center: Vec3, radius: f32, dimension: f32) {
        for cell in &mut self.cells {
            let dist = (cell.position - center).length();
            if dist < radius {
                // Smooth transition at boundary
                let blend = ((radius - dist) / radius).clamp(0.0, 1.0);
                cell.dimension = cell.dimension * (1.0 - blend) + dimension * blend;
            }
        }

        self.update_neighbors();
        self.update_statistics();
    }

    /// Create a dimensional defect (particle as 0D point)
    pub fn create_particle_defect(&mut self, position: Vec3, radius: f32) {
        self.set_dimension_sphere(position, radius, 0.0);
    }

    /// Create a string defect (1D line)
    pub fn create_string_defect(&mut self, start: Vec3, end: Vec3, thickness: f32) {
        let direction = (end - start).normalize();
        let length = (end - start).length();

        for cell in &mut self.cells {
            // Distance to line
            let to_point = cell.position - start;
            let projection = to_point.dot(direction);

            if projection >= 0.0 && projection <= length {
                let point_on_line = start + direction * projection;
                let dist = (cell.position - point_on_line).length();

                if dist < thickness {
                    let blend = ((thickness - dist) / thickness).clamp(0.0, 1.0);
                    cell.dimension = cell.dimension * (1.0 - blend) + 1.0 * blend;
                }
            }
        }

        self.update_neighbors();
        self.update_statistics();
    }

    /// Create a membrane defect (2D sheet)
    pub fn create_membrane_defect(&mut self, center: Vec3, normal: Vec3, radius: f32, thickness: f32) {
        let normal = normal.normalize();

        for cell in &mut self.cells {
            // Distance to plane
            let to_point = cell.position - center;
            let dist_to_plane = to_point.dot(normal).abs();
            let dist_in_plane = (to_point - normal * to_point.dot(normal)).length();

            if dist_in_plane < radius && dist_to_plane < thickness {
                let blend = ((thickness - dist_to_plane) / thickness).clamp(0.0, 1.0);
                cell.dimension = cell.dimension * (1.0 - blend) + 2.0 * blend;
            }
        }

        self.update_neighbors();
        self.update_statistics();
    }

    /// Create a 4D bubble (higher-dimensional region)
    pub fn create_4d_bubble(&mut self, center: Vec3, radius: f32) {
        self.set_dimension_sphere(center, radius, 4.0);
    }

    /// Create particle defect with smooth exponential falloff
    /// This creates much stronger and more realistic dimensional gradients!
    ///
    /// charge: dimensional "charge" (positive = lower dimension, negative = higher)
    /// lambda: characteristic length scale (decay length)
    pub fn create_particle_defect_smooth(&mut self, position: Vec3, charge: f32, lambda: f32) {
        for cell in &mut self.cells {
            let r = (cell.position - position).length();
            // Exponential falloff: d(r) = d_background - charge * exp(-r/λ)
            let delta = charge * (-r / lambda).exp();
            cell.dimension = (self.background_dimension - delta).max(0.0).min(5.0);
        }

        self.update_neighbors();
        self.update_statistics();
    }

    /// Create particle defect with 1/r power law falloff
    /// This should naturally give F ∝ 1/r² for dimensional gradient force!
    ///
    /// charge: dimensional "charge" strength
    /// r0: core radius (prevents singularity at r=0)
    pub fn create_particle_defect_coulomb(&mut self, position: Vec3, charge: f32, r0: f32) {
        for cell in &mut self.cells {
            let r = (cell.position - position).length();
            // Power law falloff: d(r) = d_background - charge / (r + r0)
            // Gradient of this: ∇d ∝ 1/r² → Force ∝ 1/r²!
            let delta = charge / (r + r0);
            cell.dimension = (self.background_dimension - delta).max(0.0).min(5.0);
        }

        self.update_neighbors();
        self.update_statistics();
    }

    /// Create string defect with smooth exponential falloff
    pub fn create_string_defect_smooth(&mut self, start: Vec3, end: Vec3, charge: f32, lambda: f32) {
        let direction = (end - start).normalize();
        let length = (end - start).length();

        for cell in &mut self.cells {
            // Distance to line segment
            let to_point = cell.position - start;
            let projection = to_point.dot(direction).clamp(0.0, length);
            let point_on_line = start + direction * projection;
            let r = (cell.position - point_on_line).length();

            // Exponential falloff from line
            let delta = charge * (-r / lambda).exp();
            cell.dimension = (self.background_dimension - delta).max(0.0).min(5.0);
        }

        self.update_neighbors();
        self.update_statistics();
    }

    /// Create string defect with 1/r power law falloff from line (Coulomb-like)
    /// This should give B ∝ I/r for magnetic field from current!
    ///
    /// Represents a current-carrying wire:
    /// - String defect = current (1D dimensional structure)
    /// - Dimensional circulation around wire = magnetic field
    /// - Should satisfy Ampère's Law: ∮B·dl ∝ I
    ///
    /// KEY INSIGHT: We store a "magnetic_field" vector in each cell that represents
    /// the dimensional twist/circulation. This is B = (I/(2πr)) * φ_hat (azimuthal direction).
    ///
    /// charge: dimensional "current" strength
    /// r0: core radius (prevents singularity at r=0)
    pub fn create_string_defect_coulomb(&mut self, start: Vec3, end: Vec3, charge: f32, r0: f32) {
        let wire_direction = (end - start).normalize();
        let length = (end - start).length();

        for cell in &mut self.cells {
            // Distance to line segment
            let to_point = cell.position - start;
            let projection = to_point.dot(wire_direction).clamp(0.0, length);
            let point_on_line = start + wire_direction * projection;
            let radial_vec = cell.position - point_on_line;
            let r = radial_vec.length();

            // Lower dimension near the wire (1D string defect)
            let delta = charge / (r + r0);
            cell.dimension = (self.background_dimension - delta).max(0.0).min(5.0);

            // Magnetic field in azimuthal direction: B = (charge / (2π(r + r0))) * φ_hat
            // φ_hat = wire_direction × r_hat (right-hand rule)
            if r > 1e-6 {
                let r_hat = radial_vec.normalize();
                let phi_hat = wire_direction.cross(r_hat).normalize();
                let b_magnitude = charge / (2.0 * std::f32::consts::PI * (r + r0));
                let b_field = phi_hat * b_magnitude;

                // Store in cell's vector fields
                cell.vectors.insert("magnetic_field".to_string(), b_field);
            } else {
                // At the wire center, field is undefined (or zero)
                cell.vectors.insert("magnetic_field".to_string(), Vec3::ZERO);
            }
        }

        self.update_neighbors();
        self.update_statistics();
    }

    /// Measure dimensional circulation around a closed path
    /// This is the magnetic field analogue: ∮B·dl
    ///
    /// For a circular path around a wire, this should give:
    /// Circulation ∝ current (Ampère's Law)
    pub fn measure_circulation(&self, center: Vec3, radius: f32, normal: Vec3) -> f32 {
        let normal = normal.normalize();

        // Choose two perpendicular vectors in the plane
        let tangent1 = if normal.x.abs() < 0.9 {
            normal.cross(Vec3::X).normalize()
        } else {
            normal.cross(Vec3::Y).normalize()
        };
        let tangent2 = normal.cross(tangent1).normalize();

        let mut circulation = 0.0;
        let num_samples = 64;

        for i in 0..num_samples {
            let theta = (i as f32) * 2.0 * std::f32::consts::PI / (num_samples as f32);
            let next_theta = ((i + 1) as f32) * 2.0 * std::f32::consts::PI / (num_samples as f32);

            // Points on circle
            let p1 = center + tangent1 * (theta.cos() * radius) + tangent2 * (theta.sin() * radius);
            let p2 = center + tangent1 * (next_theta.cos() * radius) + tangent2 * (next_theta.sin() * radius);

            // Path segment
            let segment = p2 - p1;
            let segment_dir = segment.normalize();
            let segment_length = segment.length();

            // Magnetic field at midpoint (from stored vector field)
            let midpoint = (p1 + p2) * 0.5;
            let b_field = self.magnetic_field_at(midpoint);

            // Line integral: ∮B·dl
            circulation += b_field.dot(segment_dir) * segment_length;
        }

        circulation
    }

    /// Get magnetic field at arbitrary position (interpolated from grid)
    pub fn magnetic_field_at(&self, pos: Vec3) -> Vec3 {
        // Find nearest cell
        let i = ((pos.x / self.spacing).floor() as usize).min(self.nx - 1);
        let j = ((pos.y / self.spacing).floor() as usize).min(self.ny - 1);
        let k = ((pos.z / self.spacing).floor() as usize).min(self.nz - 1);

        let idx = self.cell_index(i, j, k);
        if idx < self.cells.len() {
            self.cells[idx].vectors.get("magnetic_field").copied().unwrap_or(Vec3::ZERO)
        } else {
            Vec3::ZERO
        }
    }

    /// Get electric field at arbitrary position (interpolated from grid)
    pub fn electric_field_at(&self, pos: Vec3) -> Vec3 {
        // Find nearest cell
        let i = ((pos.x / self.spacing).floor() as usize).min(self.nx - 1);
        let j = ((pos.y / self.spacing).floor() as usize).min(self.ny - 1);
        let k = ((pos.z / self.spacing).floor() as usize).min(self.nz - 1);

        let idx = self.cell_index(i, j, k);
        if idx < self.cells.len() {
            self.cells[idx].vectors.get("electric_field").copied().unwrap_or(Vec3::ZERO)
        } else {
            Vec3::ZERO
        }
    }

    /// Get dimensional gradient (for force calculations)
    pub fn dimensional_gradient(&self, idx: usize) -> Vec3 {
        let cell = &self.cells[idx];
        let mut grad = Vec3::ZERO;
        let mut count = 0;

        for &neighbor_idx in &cell.neighbors {
            let neighbor = &self.cells[neighbor_idx];
            let dd = neighbor.dimension - cell.dimension;
            let dir = (neighbor.position - cell.position).normalize();
            grad += dir * dd;
            count += 1;
        }

        if count > 0 {
            grad / count as f32
        } else {
            Vec3::ZERO
        }
    }

    /// Compute force on cell based on dimensional gradient
    /// Key insight: Particles are "pushed" by dimensional gradients!
    pub fn dimensional_force(&self, idx: usize) -> Vec3 {
        let grad = self.dimensional_gradient(idx);
        -grad  // Force opposes gradient (toward lower dimension)
    }

    /// Get dimension at arbitrary position (interpolated from grid)
    pub fn dimension_at(&self, pos: Vec3) -> f32 {
        // Find grid cell containing this position
        let i = ((pos.x / self.spacing).floor() as usize).min(self.nx - 1);
        let j = ((pos.y / self.spacing).floor() as usize).min(self.ny - 1);
        let k = ((pos.z / self.spacing).floor() as usize).min(self.nz - 1);

        let idx = self.cell_index(i, j, k);
        if idx < self.cells.len() {
            self.cells[idx].dimension
        } else {
            self.background_dimension
        }
    }

    /// Get dimensional gradient at arbitrary position
    pub fn dimension_gradient_at(&self, pos: Vec3) -> Vec3 {
        // Find nearest cell
        let i = ((pos.x / self.spacing).floor() as usize).min(self.nx - 1);
        let j = ((pos.y / self.spacing).floor() as usize).min(self.ny - 1);
        let k = ((pos.z / self.spacing).floor() as usize).min(self.nz - 1);

        let idx = self.cell_index(i, j, k);
        if idx < self.cells.len() {
            self.dimensional_gradient(idx)
        } else {
            Vec3::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_3d() {
        let automata = AdaptiveAutomata::new_uniform(10, 10, 10, 3.0);

        assert_eq!(automata.cells.len(), 1000);
        assert_eq!(automata.avg_dimension, 3.0);
        assert_eq!(automata.min_dimension, 3.0);
        assert_eq!(automata.max_dimension, 3.0);
    }

    #[test]
    fn test_particle_defect() {
        let mut automata = AdaptiveAutomata::new_uniform(20, 20, 20, 3.0);

        let center = Vec3::new(10.0, 10.0, 10.0);
        automata.create_particle_defect(center, 2.0);

        // Should have reduced dimension near center
        assert!(automata.min_dimension < 1.0, "Particle defect should create low-dimensional region");
    }

    #[test]
    fn test_4d_bubble() {
        let mut automata = AdaptiveAutomata::new_uniform(20, 20, 20, 3.0);

        let center = Vec3::new(10.0, 10.0, 10.0);
        automata.create_4d_bubble(center, 3.0);

        // Should have increased dimension near center
        assert!(automata.max_dimension > 3.5, "4D bubble should create high-dimensional region");
    }

    #[test]
    fn test_neighbor_count_scaling() {
        let cell_1d = Cell::new(Vec3::ZERO, 1.0);
        let cell_2d = Cell::new(Vec3::ZERO, 2.0);
        let cell_3d = Cell::new(Vec3::ZERO, 3.0);

        assert_eq!(cell_1d.expected_neighbor_count(), 2);
        assert_eq!(cell_2d.expected_neighbor_count(), 8);
        assert_eq!(cell_3d.expected_neighbor_count(), 26);
    }
}
