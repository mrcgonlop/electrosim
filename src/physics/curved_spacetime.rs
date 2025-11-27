//! 2+1 Dimensional Curved Spacetime Automaton
//!
//! Implements emergent curved geometry from local update rules.
//! In 2+1D (2 space + 1 time), we can model:
//! - Point masses creating spatial curvature
//! - Geodesic deflection (gravitational lensing)
//! - Emergent Einstein equations from microscopic rules
//!
//! Key idea: Network edge lengths are dynamical - they respond to
//! local mass-energy density, creating effective curved geometry.

use glam::Vec2;
use std::f32::consts::PI;

/// Node in the spacetime network
#[derive(Clone, Debug)]
pub struct SpacetimeNode {
    /// Spatial position (2D)
    pub position: Vec2,

    /// Mass-energy density at this node (kg/m²)
    pub mass_density: f32,

    /// Metric scale factor g = 1 + φ (where φ is gravitational potential)
    /// This modifies distances: ds² = g² dx²
    pub metric_scale: f32,

    /// Curvature scalar (Ricci scalar in 2+1D)
    pub curvature: f32,
}

/// Edge connecting two nodes with dynamic length
#[derive(Clone, Debug)]
pub struct SpacetimeEdge {
    pub from: usize,
    pub to: usize,

    /// Rest length (Euclidean distance)
    pub rest_length: f32,

    /// Effective length (modified by metric)
    /// L_eff = L_rest * sqrt(g_from * g_to)
    pub effective_length: f32,
}

/// 2+1D Curved Spacetime Network
pub struct CurvedSpacetime2D {
    pub nodes: Vec<SpacetimeNode>,
    pub edges: Vec<SpacetimeEdge>,
    pub nx: usize,
    pub ny: usize,
    pub spacing: f32,

    /// Gravitational constant in 2+1D (different units!)
    /// In 2+1D: G has units [length/mass]
    pub g_2d: f32,
}

impl CurvedSpacetime2D {
    /// Create a new 2+1D curved spacetime lattice
    pub fn new(nx: usize, ny: usize, spacing: f32) -> Self {
        let mut nodes = Vec::with_capacity(nx * ny);

        // Initialize flat spacetime (all metric scales = 1)
        for j in 0..ny {
            for i in 0..nx {
                let position = Vec2::new(i as f32 * spacing, j as f32 * spacing);
                nodes.push(SpacetimeNode {
                    position,
                    mass_density: 0.0,
                    metric_scale: 1.0,
                    curvature: 0.0,
                });
            }
        }

        // Create edges (2D lattice connectivity)
        let mut edges = Vec::new();

        for j in 0..ny {
            for i in 0..nx {
                let idx = i + nx * j;

                // Right neighbor
                if i < nx - 1 {
                    let neighbor = idx + 1;
                    edges.push(SpacetimeEdge {
                        from: idx,
                        to: neighbor,
                        rest_length: spacing,
                        effective_length: spacing,
                    });
                }

                // Up neighbor
                if j < ny - 1 {
                    let neighbor = idx + nx;
                    edges.push(SpacetimeEdge {
                        from: idx,
                        to: neighbor,
                        rest_length: spacing,
                        effective_length: spacing,
                    });
                }

                // Diagonal neighbors (for better connectivity)
                if i < nx - 1 && j < ny - 1 {
                    let neighbor = idx + 1 + nx;
                    let diag_length = spacing * 2.0_f32.sqrt();
                    edges.push(SpacetimeEdge {
                        from: idx,
                        to: neighbor,
                        rest_length: diag_length,
                        effective_length: diag_length,
                    });
                }

                if i > 0 && j < ny - 1 {
                    let neighbor = idx - 1 + nx;
                    let diag_length = spacing * 2.0_f32.sqrt();
                    edges.push(SpacetimeEdge {
                        from: idx,
                        to: neighbor,
                        rest_length: diag_length,
                        effective_length: diag_length,
                    });
                }
            }
        }

        // G in 2+1D has units [length/mass]
        // Choosing a value that gives visible curvature
        let g_2d = 1e-13; // m/kg (weaker for stability)

        Self {
            nodes,
            edges,
            nx,
            ny,
            spacing,
            g_2d,
        }
    }

    /// Add a point mass at a location
    pub fn add_point_mass(&mut self, position: Vec2, mass: f32) {
        let mass_radius = self.spacing * 2.0; // Spread mass over ~2 cells

        for node in &mut self.nodes {
            let r = (node.position - position).length();

            if r < mass_radius {
                // Gaussian distribution of mass
                let sigma = mass_radius / 3.0;
                let density = mass * (-r * r / (2.0 * sigma * sigma)).exp()
                            / (2.0 * PI * sigma * sigma);
                node.mass_density += density;
            }
        }
    }

    /// Update metric from mass distribution
    ///
    /// In 2+1D, Einstein equations reduce to:
    /// R = -8πG T  (Ricci scalar = source)
    ///
    /// For weak field: g ≈ 1 - 2GM/r  (similar to Newtonian potential)
    pub fn update_metric(&mut self) {
        // Compute gravitational potential at each node from all mass
        for i in 0..self.nodes.len() {
            let pos_i = self.nodes[i].position;
            let mut phi = 0.0; // Gravitational potential

            // Sum contributions from all masses
            for j in 0..self.nodes.len() {
                if i == j {
                    continue;
                }

                let mass = self.nodes[j].mass_density * self.spacing * self.spacing;
                let r = (self.nodes[j].position - pos_i).length();

                if r > self.spacing * 0.1 {
                    // In 2+1D: φ ∝ G·m·ln(r) (different from 3+1D!)
                    // But for weak field, using 1/r approximation
                    phi -= self.g_2d * mass / r;
                }
            }

            // Metric scale factor: g = 1 + φ
            // Clamp to reasonable range
            self.nodes[i].metric_scale = (1.0 + phi).clamp(0.5, 2.0);
        }

        // Update edge effective lengths
        for edge in &mut self.edges {
            let g_from = self.nodes[edge.from].metric_scale;
            let g_to = self.nodes[edge.to].metric_scale;

            // Effective length is geometric mean of metric scales
            edge.effective_length = edge.rest_length * (g_from * g_to).sqrt();
        }
    }

    /// Compute curvature at each node
    ///
    /// Ricci scalar R ≈ ∇²(metric_scale) / metric_scale
    pub fn compute_curvature(&mut self) {
        let mut new_curvature = vec![0.0; self.nodes.len()];

        for j in 1..self.ny - 1 {
            for i in 1..self.nx - 1 {
                let idx = i + self.nx * j;

                // Laplacian of metric scale (discrete)
                let g_center = self.nodes[idx].metric_scale;
                let g_right = self.nodes[idx + 1].metric_scale;
                let g_left = self.nodes[idx - 1].metric_scale;
                let g_up = self.nodes[idx + self.nx].metric_scale;
                let g_down = self.nodes[idx - self.nx].metric_scale;

                let laplacian = (g_right + g_left + g_up + g_down - 4.0 * g_center)
                               / (self.spacing * self.spacing);

                // Ricci scalar (simplified)
                new_curvature[idx] = laplacian / g_center.max(0.1);
            }
        }

        // Update curvature
        for i in 0..self.nodes.len() {
            self.nodes[i].curvature = new_curvature[i];
        }
    }

    /// Trace a geodesic (shortest path in curved space)
    ///
    /// Returns positions along the geodesic from start to end
    ///
    /// Uses geodesic equation: d²x^μ/dλ² = -Γ^μ_νρ (dx^ν/dλ)(dx^ρ/dλ)
    /// In weak field: deflection ≈ -∇(ln g) for null geodesics
    pub fn trace_geodesic(&self, start: Vec2, end: Vec2, steps: usize) -> Vec<Vec2> {
        let mut path = Vec::with_capacity(steps);
        path.push(start);

        let mut current = start;
        let initial_dir = (end - start).normalize();
        let mut direction = initial_dir;
        let step_size = (end - start).length() / steps as f32;

        for _ in 0..steps {
            // Compute metric gradient for geodesic deflection
            let grad_g = self.metric_gradient(current);

            // Geodesic equation: direction changes due to Christoffel symbols
            // In weak field: Γ ≈ (1/2)∇g, so deflection ∝ ∇g
            let deflection = grad_g * (-0.5); // Deflection strength

            // Update direction (parallel transport with deflection)
            direction = (direction + deflection * step_size).normalize();

            // Step along geodesic
            current = current + direction * step_size;

            path.push(current);

            // Stop if we've reached the target area
            if (current - end).length() < step_size * 2.0 {
                break;
            }

            // Stop if we've left the domain
            if current.x < 0.0 || current.x >= (self.nx as f32 * self.spacing)
                || current.y < 0.0 || current.y >= (self.ny as f32 * self.spacing) {
                break;
            }
        }

        path.push(end);
        path
    }

    /// Compute gradient of metric scale at a position
    fn metric_gradient(&self, pos: Vec2) -> Vec2 {
        let h = self.spacing * 0.5; // Finite difference step

        let g_xp = self.interpolate_metric(pos + Vec2::new(h, 0.0));
        let g_xm = self.interpolate_metric(pos - Vec2::new(h, 0.0));
        let g_yp = self.interpolate_metric(pos + Vec2::new(0.0, h));
        let g_ym = self.interpolate_metric(pos - Vec2::new(0.0, h));

        Vec2::new(
            (g_xp - g_xm) / (2.0 * h),
            (g_yp - g_ym) / (2.0 * h),
        )
    }

    /// Interpolate metric scale at arbitrary position
    fn interpolate_metric(&self, pos: Vec2) -> f32 {
        let i = (pos.x / self.spacing).floor().max(0.0).min(self.nx as f32 - 2.0) as usize;
        let j = (pos.y / self.spacing).floor().max(0.0).min(self.ny as f32 - 2.0) as usize;

        let fx = (pos.x / self.spacing) - i as f32;
        let fy = (pos.y / self.spacing) - j as f32;

        let idx00 = i + self.nx * j;
        let idx10 = (i + 1) + self.nx * j;
        let idx01 = i + self.nx * (j + 1);
        let idx11 = (i + 1) + self.nx * (j + 1);

        // Bilinear interpolation
        let g00 = self.nodes[idx00].metric_scale;
        let g10 = self.nodes[idx10].metric_scale;
        let g01 = self.nodes[idx01].metric_scale;
        let g11 = self.nodes[idx11].metric_scale;

        let g0 = g00 * (1.0 - fx) + g10 * fx;
        let g1 = g01 * (1.0 - fx) + g11 * fx;
        g0 * (1.0 - fy) + g1 * fy
    }

    /// Get node index from grid coordinates
    pub fn index(&self, i: usize, j: usize) -> usize {
        i + self.nx * j
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spacetime_creation() {
        let st = CurvedSpacetime2D::new(10, 10, 0.1);
        assert_eq!(st.nodes.len(), 100);

        // Check flat space initially
        for node in &st.nodes {
            assert_eq!(node.metric_scale, 1.0);
            assert_eq!(node.curvature, 0.0);
        }
    }

    #[test]
    fn test_point_mass() {
        let mut st = CurvedSpacetime2D::new(20, 20, 0.1);
        let center = Vec2::new(1.0, 1.0);

        st.add_point_mass(center, 1e10); // Large mass
        st.update_metric();
        st.compute_curvature();

        // Metric should be modified near mass
        let center_idx = st.index(10, 10);
        assert_ne!(st.nodes[center_idx].metric_scale, 1.0);

        // Should have non-zero curvature
        assert_ne!(st.nodes[center_idx].curvature, 0.0);
    }
}
