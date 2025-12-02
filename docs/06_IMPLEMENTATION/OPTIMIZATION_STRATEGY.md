# Optimization Strategy: Scaling to Realistic EM Simulations

## Goal: Electron-Scale Simulations

**Target:** Simulate individual electrons, conductors, EM fields at physically realistic scales

**Challenge:**
- Electron needs ~10^6 - 10^9 graph nodes (estimate based on required dimensional structure)
- Conductor with multiple electrons: ~10^9+ nodes
- Real-time evolution required for experimental validation

**Current bottleneck:** Force-directed embedding is O(n² × iterations)

---

## Performance Analysis

### Critical Path (from profiling)

1. **Force-Directed Embedding: O(n² × iterations)** ⚠️ PRIMARY BOTTLENECK
   - For each iteration:
     - For each node pair: compute repulsive force  [O(n²)]
     - For each edge: compute attractive force       [O(|E|)]
     - Update positions                               [O(n)]
   - Dominates for n > 1000

2. **Graph Evolution: O(|E| × rules × steps)**
   - Pattern matching for each rule
   - Edge insertion/deletion
   - Scales reasonably well (linear in edges)

3. **Automata Projection: O(grid³ × nodes)**
   - For each cell, find nearest graph nodes
   - Compute local dimension
   - Manageable with spatial indexing

4. **Rendering: O(visible_cells × frames)**
   - Already mostly GPU-accelerated (rasterization)
   - Minor bottleneck

---

## Optimization Strategies

### Strategy 1: GPU-Accelerated Force-Directed Layout

**Why:** Force computation is embarrassingly parallel

**Implementation:**
```rust
// Use wgpu compute shaders
struct GPUEmbedding {
    device: wgpu::Device,
    queue: wgpu::Queue,
    force_pipeline: wgpu::ComputePipeline,

    // GPU buffers
    positions: wgpu::Buffer,      // Vec3 per node
    velocities: wgpu::Buffer,     // Vec3 per node
    edges: wgpu::Buffer,          // Edge list
    forces: wgpu::Buffer,         // Accumulated forces
}

impl GPUEmbedding {
    fn compute_forces_gpu(&mut self) {
        // Dispatch compute shader:
        // Each workgroup handles force computation for a node
        // All node pairs processed in parallel

        // Shader pseudocode:
        // @compute @workgroup_size(256)
        // fn compute_forces(@builtin(global_invocation_id) id: vec3<u32>) {
        //     let node_id = id.x;
        //     var force = vec3(0.0);
        //
        //     // Repulsive forces from all nodes
        //     for (var other = 0u; other < num_nodes; other++) {
        //         if (other != node_id) {
        //             force += repulsive_force(node_id, other);
        //         }
        //     }
        //
        //     // Attractive forces from connected nodes
        //     for (var e = 0u; e < num_edges; e++) {
        //         if (edges[e].contains(node_id)) {
        //             force += attractive_force(node_id, edges[e]);
        //         }
        //     }
        //
        //     forces[node_id] = force;
        // }
    }
}
```

**Expected speedup:** 100-1000x on GPU vs CPU for large graphs

**Dependencies:** `wgpu` crate (already used for rendering)

---

### Strategy 2: Hierarchical Approximation (Barnes-Hut)

**Why:** Don't need exact forces for distant nodes

**Algorithm:**
```rust
// Octree for spatial decomposition
struct OctreeNode {
    center: Vec3,
    size: f32,
    total_mass: f32,
    center_of_mass: Vec3,
    children: Option<Box<[OctreeNode; 8]>>,
}

// Approximate force from distant clusters
fn compute_force_barnes_hut(node_pos: Vec3, tree: &OctreeNode) -> Vec3 {
    let d = (tree.center_of_mass - node_pos).length();
    let theta = 0.5;  // Accuracy parameter

    if tree.size / d < theta || tree.children.is_none() {
        // Far enough: treat as single mass
        repulsive_force(node_pos, tree.center_of_mass, tree.total_mass)
    } else {
        // Too close: recurse
        tree.children.iter().map(|child| {
            compute_force_barnes_hut(node_pos, child)
        }).sum()
    }
}
```

**Complexity:** O(n log n) instead of O(n²)

**Tradeoff:** Slight accuracy loss, but massive speedup

---

### Strategy 3: Graph Compression (Remove Redundancy)

**Key insight:** Many graph regions have repetitive structure

**Compression strategies:**

#### 3a. Merge Equivalent Nodes
```rust
// If two nodes have identical connectivity, merge them
fn compress_equivalent_nodes(graph: &mut Hypergraph) {
    let mut equivalence_classes: HashMap<EdgeSet, Vec<NodeID>> = HashMap::new();

    for node in &graph.nodes {
        let edges = graph.edges_containing(node);
        equivalence_classes.entry(edges).or_default().push(node);
    }

    // Merge nodes in each equivalence class
    for (_, nodes) in equivalence_classes {
        if nodes.len() > 1 {
            merge_nodes(graph, &nodes);
        }
    }
}
```

#### 3b. Homogeneous Region Collapse
```rust
// If a region has uniform dimension, represent it compactly
struct CompressedRegion {
    bounding_box: AABB,
    dimension: f32,
    node_density: f32,  // Nodes per unit volume

    // Only store boundary nodes explicitly
    boundary_nodes: Vec<NodeID>,
}
```

#### 3c. Rule Memoization
```rust
// Don't apply rules that create redundant structure
struct RuleCache {
    applied_patterns: HashSet<GraphPattern>,
}

fn rule_with_memoization(graph: &mut Hypergraph, cache: &mut RuleCache) -> bool {
    let pattern = graph.canonical_pattern();

    if cache.applied_patterns.contains(&pattern) {
        return false;  // Already applied this rule to this pattern
    }

    if apply_rule(graph) {
        cache.applied_patterns.insert(pattern);
        true
    } else {
        false
    }
}
```

---

### Strategy 4: Sparse Automata Representation

**Problem:** 50³ grid = 125k cells, but most are uniform 3D space

**Solution:** Only store cells that deviate from background

```rust
struct SparseAutomata {
    background_dimension: f32,  // Typically 3.0

    // Only store non-background cells
    special_cells: HashMap<(usize, usize, usize), Cell>,

    nx: usize,
    ny: usize,
    nz: usize,
    spacing: f32,
}

impl SparseAutomata {
    fn get_cell(&self, i: usize, j: usize, k: usize) -> Cell {
        self.special_cells.get(&(i, j, k)).cloned().unwrap_or_else(|| {
            Cell {
                position: self.cell_position(i, j, k),
                dimension: self.background_dimension,
                // ... default values
            }
        })
    }
}
```

**Memory savings:** 1000x for typical sparse EM fields

---

### Strategy 5: Multi-Scale Simulation

**Insight:** Different regions need different resolution

```rust
struct MultiScaleGraph {
    // Fine scale (Planck-level): dimensional defects, particle cores
    fine_graphs: Vec<Hypergraph>,
    fine_positions: Vec<Vec3>,

    // Coarse scale: bulk spacetime, far-field EM
    coarse_graph: Hypergraph,

    // Coupling between scales
    boundary_conditions: Vec<ScaleCoupling>,
}

// Electron = fine-scale dimensional defect + coarse-scale field
```

**Example:**
- Electron core: 10^6 nodes at Planck scale
- EM field: 10^4 nodes at coarse scale
- Total: 10^6 nodes instead of 10^9

---

## New Pipeline Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ 1. GRAPH EVOLUTION (GPU-accelerated rule matching)         │
│    - Parallel pattern detection                             │
│    - Compressed representation                              │
│    - Rule memoization                                       │
└───────────────────┬─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. EMBEDDING (GPU Barnes-Hut)                               │
│    - Octree spatial indexing                                │
│    - Hierarchical force approximation                       │
│    - O(n log n) instead of O(n²)                            │
└───────────────────┬─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. SPARSE AUTOMATA (Only store non-background)              │
│    - Adaptive grid resolution                               │
│    - Fine scale near defects                                │
│    - Coarse scale in bulk                                   │
└───────────────────┬─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. EXPERIMENTAL TESTS (Null hypothesis via automata)        │
│    - Measure force laws in automata                         │
│    - Check dimensional predictions                          │
│    - Validate against historical experiments                │
└─────────────────────────────────────────────────────────────┘
```

---

## Mapping to Experimental Tests (Null Hypothesis)

### The Key Challenge

**Old approach:**
```rust
// Assumed Maxwell/Weber equations
// Applied to preset fields
// Compared results
```

**New approach:**
```rust
// 1. Evolve hypergraph (no EM assumed!)
// 2. Measure emergent forces in automata
// 3. Compare to experimental data
// 4. Null hypothesis: forces emerge from dimension gradients
```

### Example: Coulomb's Law Test

```rust
fn test_coulomb_emergence(graph: Hypergraph) -> TestResult {
    // Create two particle defects (electrons)
    let mut automata = graph_to_automata(graph);

    let particle1_pos = Vec3::new(10.0, 25.0, 25.0);
    let particle2_pos = Vec3::new(40.0, 25.0, 25.0);

    automata.create_particle_defect(particle1_pos, 2.0);
    automata.create_particle_defect(particle2_pos, 2.0);

    // Measure force via dimensional gradient
    let force = measure_dimensional_force(&automata, particle1_pos, particle2_pos);

    // Expected: F ∝ 1/r² in d=3 space
    let r = (particle2_pos - particle1_pos).length();
    let expected_force = 1.0 / (r * r);  // Coulomb form

    // Null hypothesis: force emerges without assuming EM!
    let ratio = force / expected_force;

    TestResult {
        passed: (ratio - 1.0).abs() < 0.1,  // 10% tolerance
        measured: force,
        expected: expected_force,
        ratio,
    }
}

fn measure_dimensional_force(
    automata: &AdaptiveAutomata,
    pos1: Vec3,
    pos2: Vec3,
) -> f32 {
    // Force = gradient of dimensional tension
    // F = -∇U where U ∝ ∫ (d - 3)² dV

    let mut force = Vec3::ZERO;
    let direction = (pos2 - pos1).normalize();

    // Measure dimensional gradient along line connecting particles
    let steps = 100;
    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let pos = pos1.lerp(pos2, t);

        let dim = automata.dimension_at(pos);
        let grad = automata.dimension_gradient_at(pos);

        // Dimensional tension creates force
        let local_force = -grad * (dim - 3.0).powi(2);
        force += local_force * direction;
    }

    force.length()
}
```

### Mapping All 9 Experiments

| Experiment | How to Test via Automata |
|------------|--------------------------|
| **Coulomb's Law** | Measure force between particle defects via dimension gradient |
| **Biot-Savart** | Measure field around string defect (current = moving dimension) |
| **Faraday Induction** | Change defect structure → measure induced dimensional waves |
| **Ampère's Law** | Circulation of dimensional gradient around string defect |
| **Lorentz Force** | Force on moving defect in dimensional field |
| **Gauss's Law** | Flux of dimensional gradient through surface |
| **Light Speed** | Measure wave speed in dimensional field perturbations |
| **EM Waves** | Oscillating defects create propagating dimensional waves |
| **Energy Conservation** | Total dimensional tension conserved during evolution |

---

## Code Cleanup Plan

### Files to REMOVE (obsolete):
```
src/physics/maxwell.rs          # Assumed Maxwell equations
src/physics/weber.rs            # Assumed Weber force
src/physics/weber_particles.rs  # Preset particle dynamics
src/physics/wave2d.rs           # 2D wave equation (ad-hoc)
src/physics/fluid2d.rs          # Fluid mechanics (not fundamental)
```

### Files to REFACTOR:
```
src/physics/experimental_tests.rs
  → Use automata.measure_force() instead of preset equations
  → Test emergence, not assumptions

src/physics/curved_spacetime.rs
  → Keep, but reinterpret: curvature = dimension gradients
```

### Files to KEEP (core framework):
```
src/physics/hypergraph.rs              # Foundation
src/physics/simple_rules.rs           # Fundamental evolution
src/physics/graph_embedding.rs         # Projection to observable space
src/physics/adaptive_automata.rs       # Emergent dimensional field
src/visualization/*                    # All visualization
```

---

## Implementation Priority

### Phase 1: Performance (IMMEDIATE)
1. ✓ Profile current bottlenecks
2. Implement Barnes-Hut O(n log n) embedding
3. GPU compute shader for forces
4. Sparse automata representation

### Phase 2: Scaling Test
1. Test with 10^4 nodes (CPU)
2. Test with 10^6 nodes (GPU)
3. Benchmark rule application at scale

### Phase 3: EM Emergence
1. Create electron defect (test particle)
2. Create conductor defect (extended structure)
3. Measure emergent Coulomb law
4. Measure emergent Biot-Savart

### Phase 4: Experimental Validation
1. Refactor all 9 tests to use automata
2. Run full test suite on emergent EM
3. Tune graph rules to match experiments
4. Publish results!

---

## Expected Performance After Optimization

| Operation | Current (CPU) | With GPU + Optimizations |
|-----------|---------------|--------------------------|
| Embed 1k nodes | ~5s | ~50ms |
| Embed 10k nodes | ~500s | ~500ms |
| Embed 100k nodes | Too slow | ~5s |
| Embed 1M nodes | N/A | ~30s |
| Rule application (1M edges) | ~1s | ~100ms (GPU patterns) |
| Automata projection (100³) | ~100ms | ~10ms (spatial index) |
| Full pipeline (electron) | N/A | **~1 min** ✓ |

**Conclusion:** Realistic EM simulations become feasible with these optimizations!
