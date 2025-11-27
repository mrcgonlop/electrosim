//! Spacetime network - graph-based simulation structure
//!
//! Combines regular 3D grid topology with non-local connections for
//! supporting both local field propagation and action-at-a-distance.

use super::spacetime_cell::SpacetimeCell;
use glam::{Vec3, Vec4};
use std::collections::HashMap;

/// The complete spacetime network simulation
///
/// Combines:
/// - Regular 3D lattice (for local field/fluid dynamics)
/// - Sparse non-local graph (for Weber forces, entanglement, etc.)
///
/// # Example
///
/// ```
/// use em_physics_sandbox::simulation::SpacetimeNetwork;
///
/// let mut network = SpacetimeNetwork::new(32, 32, 32, 0.01);
/// assert_eq!(network.cells.len(), 32 * 32 * 32);
/// ```
pub struct SpacetimeNetwork {
    /// All spacetime cells (regular 3D grid)
    pub cells: Vec<SpacetimeCell>,

    /// Grid dimensions
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,

    /// Physical spacing between cells (meters)
    pub spacing: f32,

    /// Current simulation time (seconds)
    pub time: f32,

    /// Non-local connections (sparse graph)
    pub nonlocal_edges: Vec<NonLocalEdge>,

    /// Fast edge lookup: (source, target) -> edge_index
    pub edge_index: HashMap<(usize, usize), usize>,
}

/// Non-local edge connecting distant cells
///
/// Used for action-at-a-distance interactions like Weber forces,
/// quantum entanglement, or speculative wormhole connections.
#[derive(Debug, Clone)]
pub struct NonLocalEdge {
    /// Source cell index
    pub source: usize,

    /// Target cell index
    pub target: usize,

    /// Interaction strength/coupling constant
    pub coupling: f32,

    /// Type of interaction
    pub kind: InteractionKind,

    /// Time delay for retarded interactions (seconds)
    pub delay: f32,

    /// Spatial distance between cells (meters)
    pub distance: f32,
}

/// Types of non-local interactions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionKind {
    /// Instantaneous Weber electrodynamic force
    WeberInstantaneous,

    /// Retarded (light-speed limited) Weber force
    WeberRetarded,

    /// Quantum entanglement correlation
    Entanglement,

    /// Speculative wormhole/shortcut connection
    Wormhole,

    /// Custom user-defined interaction
    Custom,
}

impl SpacetimeNetwork {
    /// Create a new spacetime network with regular 3D grid
    ///
    /// # Arguments
    ///
    /// * `nx`, `ny`, `nz` - Grid dimensions
    /// * `spacing` - Physical distance between cells (meters)
    ///
    /// # Example
    ///
    /// ```
    /// use em_physics_sandbox::simulation::SpacetimeNetwork;
    ///
    /// // 64³ grid with 1cm spacing
    /// let network = SpacetimeNetwork::new(64, 64, 64, 0.01);
    /// ```
    pub fn new(nx: usize, ny: usize, nz: usize, spacing: f32) -> Self {
        let size = nx * ny * nz;
        let mut cells = Vec::with_capacity(size);

        // Initialize cells with spacetime positions
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let pos = Vec4::new(
                        i as f32 * spacing,
                        j as f32 * spacing,
                        k as f32 * spacing,
                        0.0, // Initial time
                    );

                    let volume = spacing.powi(3);
                    cells.push(SpacetimeCell::at_position(pos, volume));
                }
            }
        }

        Self {
            cells,
            nx,
            ny,
            nz,
            spacing,
            time: 0.0,
            nonlocal_edges: Vec::new(),
            edge_index: HashMap::new(),
        }
    }

    /// Get linear index from 3D coordinates
    #[inline]
    pub fn index(&self, i: usize, j: usize, k: usize) -> usize {
        i + self.nx * (j + self.ny * k)
    }

    /// Get 3D coordinates from linear index
    pub fn coords(&self, idx: usize) -> (usize, usize, usize) {
        let i = idx % self.nx;
        let j = (idx / self.nx) % self.ny;
        let k = idx / (self.nx * self.ny);
        (i, j, k)
    }

    /// Check if coordinates are valid
    pub fn in_bounds(&self, i: i32, j: i32, k: i32) -> bool {
        i >= 0 && i < self.nx as i32 &&
        j >= 0 && j < self.ny as i32 &&
        k >= 0 && k < self.nz as i32
    }

    /// Get linear index from 3D coordinates (checked)
    pub fn index_checked(&self, i: i32, j: i32, k: i32) -> Option<usize> {
        if self.in_bounds(i, j, k) {
            Some(self.index(i as usize, j as usize, k as usize))
        } else {
            None
        }
    }

    /// Get 6 nearest face neighbors (von Neumann neighborhood)
    pub fn face_neighbors(&self, idx: usize) -> Vec<usize> {
        let (i, j, k) = self.coords(idx);
        let mut neighbors = Vec::with_capacity(6);

        let offsets = [
            (1, 0, 0), (-1, 0, 0),
            (0, 1, 0), (0, -1, 0),
            (0, 0, 1), (0, 0, -1),
        ];

        for (di, dj, dk) in offsets {
            if let Some(nidx) = self.index_checked(
                i as i32 + di,
                j as i32 + dj,
                k as i32 + dk
            ) {
                neighbors.push(nidx);
            }
        }

        neighbors
    }

    /// Get 26 nearest neighbors (Moore neighborhood)
    pub fn neighbors(&self, idx: usize) -> Vec<usize> {
        let (i, j, k) = self.coords(idx);
        let mut neighbors = Vec::new();

        for dk in -1..=1 {
            for dj in -1..=1 {
                for di in -1..=1 {
                    if di == 0 && dj == 0 && dk == 0 {
                        continue; // Skip self
                    }

                    if let Some(nidx) = self.index_checked(
                        i as i32 + di,
                        j as i32 + dj,
                        k as i32 + dk
                    ) {
                        neighbors.push(nidx);
                    }
                }
            }
        }

        neighbors
    }

    /// Add a non-local edge between two cells
    pub fn add_nonlocal_edge(
        &mut self,
        source: usize,
        target: usize,
        coupling: f32,
        kind: InteractionKind,
    ) {
        // Compute distance
        let pos_s = self.cells[source].position.truncate();
        let pos_t = self.cells[target].position.truncate();
        let distance = (pos_t - pos_s).length();

        // Compute light travel time
        const C: f32 = 299_792_458.0; // m/s
        let delay = distance / C;

        let edge = NonLocalEdge {
            source,
            target,
            coupling,
            kind,
            delay,
            distance,
        };

        let edge_idx = self.nonlocal_edges.len();
        self.nonlocal_edges.push(edge);
        self.edge_index.insert((source, target), edge_idx);
    }

    /// Remove a non-local edge
    pub fn remove_nonlocal_edge(&mut self, source: usize, target: usize) {
        if let Some(&edge_idx) = self.edge_index.get(&(source, target)) {
            self.nonlocal_edges.remove(edge_idx);
            self.edge_index.remove(&(source, target));

            // Rebuild index (in production, use a better data structure)
            self.rebuild_edge_index();
        }
    }

    fn rebuild_edge_index(&mut self) {
        self.edge_index.clear();
        for (idx, edge) in self.nonlocal_edges.iter().enumerate() {
            self.edge_index.insert((edge.source, edge.target), idx);
        }
    }

    /// Automatically detect particles and create Weber graph
    ///
    /// Scans all cells for particles (high charge density) and creates
    /// pairwise non-local connections for Weber electrodynamics.
    pub fn update_weber_graph(&mut self) {
        // Clear old Weber edges
        self.nonlocal_edges.retain(|e|
            e.kind != InteractionKind::WeberInstantaneous &&
            e.kind != InteractionKind::WeberRetarded
        );

        // Find all particles
        let particle_indices: Vec<usize> = self.cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.is_particle())
            .map(|(idx, _)| idx)
            .collect();

        // Create pairwise Weber connections
        for i in 0..particle_indices.len() {
            for j in (i + 1)..particle_indices.len() {
                let idx_i = particle_indices[i];
                let idx_j = particle_indices[j];

                // Coupling strength: q_i * q_j
                let q_i = self.cells[idx_i].charge_density * self.cells[idx_i].volume;
                let q_j = self.cells[idx_j].charge_density * self.cells[idx_j].volume;
                let coupling = q_i * q_j;

                self.add_nonlocal_edge(
                    idx_i,
                    idx_j,
                    coupling,
                    InteractionKind::WeberInstantaneous,
                );
            }
        }
    }

    /// Get all edges connected to a cell
    pub fn edges_from(&self, idx: usize) -> Vec<&NonLocalEdge> {
        self.nonlocal_edges
            .iter()
            .filter(|e| e.source == idx || e.target == idx)
            .collect()
    }

    /// Advance simulation time
    pub fn advance_time(&mut self, dt: f32) {
        self.time += dt;

        // Update time coordinate in all cells
        for cell in &mut self.cells {
            cell.position.w = self.time;
        }
    }

    /// Get total energy in the network
    pub fn total_energy(&self) -> f32 {
        self.cells.iter().map(|c| c.total_energy()).sum()
    }

    /// Get total momentum in the network
    pub fn total_momentum(&self) -> Vec3 {
        self.cells.iter().map(|c| c.total_momentum()).sum()
    }

    /// Get total charge in the network (for conservation check)
    pub fn total_charge(&self) -> f32 {
        self.cells
            .iter()
            .map(|c| c.charge_density * c.volume)
            .sum()
    }

    /// Set up a Gaussian charge distribution
    pub fn add_gaussian_charge_distribution(
        &mut self,
        center: Vec3,
        total_charge: f32,
        sigma: f32,
    ) {
        for cell in &mut self.cells {
            let pos = cell.position.truncate();
            let distance = (pos - center).length();
            cell.add_gaussian_charge(total_charge, sigma, distance);
        }
    }

    /// Set up a plane wave in the EM fields
    pub fn initialize_plane_wave(
        &mut self,
        direction: Vec3,
        amplitude: f32,
        wavelength: f32,
    ) {
        let k = 2.0 * std::f32::consts::PI / wavelength;
        let dir_norm = direction.normalize();

        for cell in &mut self.cells {
            let pos = cell.position.truncate();
            let phase = k * pos.dot(dir_norm);

            // E field perpendicular to propagation
            let e_dir = Vec3::new(-dir_norm.y, dir_norm.x, 0.0).normalize();
            cell.e_field = amplitude * phase.cos() * e_dir;

            // B field: B = (k × E) / ω = (dir × E) / c
            cell.b_field = dir_norm.cross(cell.e_field) / 299_792_458.0;
        }
    }

    /// Set up a conductor region
    pub fn set_conductor_region<F>(&mut self, predicate: F)
    where
        F: Fn(Vec3) -> bool,
    {
        for cell in &mut self.cells {
            let pos = cell.position.truncate();
            if predicate(pos) {
                cell.sigma = 1e8; // High conductivity
                cell.apply_conductor_bc();
            }
        }
    }

    /// Get statistics for debugging
    pub fn stats(&self) -> NetworkStats {
        let particle_count = self.cells.iter().filter(|c| c.is_particle()).count();
        let nonlocal_edge_count = self.nonlocal_edges.len();
        let total_cells = self.cells.len();

        NetworkStats {
            total_cells,
            particle_count,
            nonlocal_edge_count,
            total_energy: self.total_energy(),
            total_charge: self.total_charge(),
            time: self.time,
        }
    }
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub total_cells: usize,
    pub particle_count: usize,
    pub nonlocal_edge_count: usize,
    pub total_energy: f32,
    pub total_charge: f32,
    pub time: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_creation() {
        let network = SpacetimeNetwork::new(10, 10, 10, 0.01);
        assert_eq!(network.cells.len(), 1000);
        assert_eq!(network.nx, 10);
        assert_eq!(network.spacing, 0.01);
    }

    #[test]
    fn test_indexing() {
        let network = SpacetimeNetwork::new(10, 10, 10, 0.01);
        let idx = network.index(5, 3, 2);
        let (i, j, k) = network.coords(idx);
        assert_eq!((i, j, k), (5, 3, 2));
    }

    #[test]
    fn test_face_neighbors() {
        let network = SpacetimeNetwork::new(10, 10, 10, 0.01);
        let center = network.index(5, 5, 5);
        let neighbors = network.face_neighbors(center);
        assert_eq!(neighbors.len(), 6);
    }

    #[test]
    fn test_neighbors() {
        let network = SpacetimeNetwork::new(10, 10, 10, 0.01);
        let center = network.index(5, 5, 5);
        let neighbors = network.neighbors(center);
        assert_eq!(neighbors.len(), 26);
    }

    #[test]
    fn test_corner_neighbors() {
        let network = SpacetimeNetwork::new(10, 10, 10, 0.01);
        let corner = network.index(0, 0, 0);
        let neighbors = network.neighbors(corner);
        // Corner has fewer neighbors
        assert_eq!(neighbors.len(), 7);
    }

    #[test]
    fn test_add_nonlocal_edge() {
        let mut network = SpacetimeNetwork::new(10, 10, 10, 0.01);
        let idx1 = network.index(2, 2, 2);
        let idx2 = network.index(7, 7, 7);

        network.add_nonlocal_edge(idx1, idx2, 1.0, InteractionKind::WeberInstantaneous);
        assert_eq!(network.nonlocal_edges.len(), 1);

        let edge = &network.nonlocal_edges[0];
        assert_eq!(edge.source, idx1);
        assert_eq!(edge.target, idx2);
        assert!(edge.distance > 0.0);
    }

    #[test]
    fn test_weber_graph_update() {
        let mut network = SpacetimeNetwork::new(10, 10, 10, 0.01);

        // Add two particles
        let idx1 = network.index(3, 3, 3);
        let idx2 = network.index(7, 7, 7);
        network.cells[idx1].charge_density = 1e-5;
        network.cells[idx2].charge_density = -1e-5;

        network.update_weber_graph();

        // Should have one edge connecting the two particles
        assert_eq!(network.nonlocal_edges.len(), 1);
        assert_eq!(network.nonlocal_edges[0].kind, InteractionKind::WeberInstantaneous);
    }

    #[test]
    fn test_advance_time() {
        let mut network = SpacetimeNetwork::new(5, 5, 5, 0.01);
        network.advance_time(1e-12);

        assert_eq!(network.time, 1e-12);
        assert_eq!(network.cells[0].position.w, 1e-12);
    }

    #[test]
    fn test_total_charge_conservation() {
        let mut network = SpacetimeNetwork::new(10, 10, 10, 0.01);

        // Add charge
        network.cells[50].set_point_charge(1e-19);
        network.cells[100].set_point_charge(-1e-19);

        let total_charge = network.total_charge();
        assert!(total_charge.abs() < 1e-25); // Should be ~zero
    }

    #[test]
    fn test_gaussian_charge_distribution() {
        let mut network = SpacetimeNetwork::new(20, 20, 20, 0.01);
        let center = Vec3::new(0.1, 0.1, 0.1);

        network.add_gaussian_charge_distribution(center, 1e-15, 0.02);

        let total_charge = network.total_charge();
        assert!((total_charge - 1e-15).abs() < 1e-17);
    }

    #[test]
    fn test_stats() {
        let mut network = SpacetimeNetwork::new(10, 10, 10, 0.01);
        network.cells[50].charge_density = 1e-5;

        let stats = network.stats();
        assert_eq!(stats.total_cells, 1000);
        assert_eq!(stats.particle_count, 1);
        assert_eq!(stats.time, 0.0);
    }
}
