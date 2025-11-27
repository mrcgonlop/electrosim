//! Performance benchmarks for the EM physics sandbox

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use em_physics_sandbox::{
    physics::{EMTheory, MaxwellTheory, ScalarWaveTheory, Wave2D},
    simulation::VoxelGrid,
};

/// Benchmark 2D wave equation updates
fn bench_2d_wave(c: &mut Criterion) {
    let mut group = c.benchmark_group("2d_wave");

    for size in [64, 128, 256] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let wave = Wave2D::new(1.0);
            let mut field = vec![0.0; size * size];
            let mut velocity = vec![0.0; size * size];

            wave.set_gaussian_pulse(&mut field, (size / 2, size / 2), 1.0, 8.0, size, size);

            b.iter(|| {
                wave.update_scalar_field(
                    black_box(&mut field),
                    black_box(&mut velocity),
                    size,
                    size,
                    0.1,
                    0.005,
                );
            });
        });
    }

    group.finish();
}

/// Benchmark 3D Maxwell FDTD updates
fn bench_maxwell_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("maxwell_fdtd");

    for size in [16, 32, 64] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let maxwell = MaxwellTheory::new();
            let mut grid = VoxelGrid::new(size, size, size, 0.01);

            maxwell.initialize(&mut grid);

            b.iter(|| {
                maxwell.update_fields(black_box(&mut grid), black_box(1e-12));
            });
        });
    }

    group.finish();
}

/// Benchmark voxel grid operations
fn bench_grid_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_ops");

    let grid = VoxelGrid::new(64, 64, 64, 0.01);

    group.bench_function("index_calculation", |b| {
        b.iter(|| {
            let mut sum = 0;
            for i in 0..64 {
                for j in 0..64 {
                    for k in 0..64 {
                        sum += black_box(grid.index(i, j, k));
                    }
                }
            }
            sum
        });
    });

    group.bench_function("field_interpolation", |b| {
        b.iter(|| {
            grid.interpolate_field(black_box(glam::Vec3::new(0.15, 0.15, 0.15)))
        });
    });

    group.finish();
}

/// Benchmark energy calculations
fn bench_energy_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("energy");

    for size in [16, 32, 64] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let maxwell = MaxwellTheory::new();
            let mut grid = VoxelGrid::new(size, size, size, 0.01);

            maxwell.initialize(&mut grid);

            b.iter(|| maxwell.total_energy(black_box(&grid)));
        });
    }

    group.finish();
}

/// Benchmark field initialization
fn bench_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("initialization");

    group.bench_function("gaussian_pulse", |b| {
        let maxwell = MaxwellTheory::new();
        let mut grid = VoxelGrid::new(32, 32, 32, 0.01);

        b.iter(|| {
            maxwell.initialize_gaussian_pulse(
                black_box(&mut grid),
                glam::Vec3::splat(0.16),
                0.05,
                100.0,
            );
        });
    });

    group.bench_function("plane_wave", |b| {
        let maxwell = MaxwellTheory::new();
        let mut grid = VoxelGrid::new(32, 32, 32, 0.01);

        b.iter(|| {
            maxwell.initialize_plane_wave(black_box(&mut grid), 1e9, glam::Vec3::X);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_2d_wave,
    bench_maxwell_update,
    bench_grid_operations,
    bench_energy_calculation,
    bench_initialization
);
criterion_main!(benches);
