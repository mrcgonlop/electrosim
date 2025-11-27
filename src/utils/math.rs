//! Mathematical utility functions

use glam::Vec3;

/// Calculate the divergence of a vector field at a point using finite differences.
///
/// ∇·F = ∂Fx/∂x + ∂Fy/∂y + ∂Fz/∂z
///
/// # Arguments
///
/// * `field` - The vector field values
/// * `i`, `j`, `k` - Grid coordinates
/// * `nx`, `ny`, `nz` - Grid dimensions
/// * `dx` - Grid spacing
pub fn divergence(
    field: &[Vec3],
    i: usize,
    j: usize,
    k: usize,
    nx: usize,
    ny: usize,
    nz: usize,
    dx: f32,
) -> f32 {
    let idx = i + nx * (j + ny * k);

    if i == 0 || i >= nx - 1 || j == 0 || j >= ny - 1 || k == 0 || k >= nz - 1 {
        return 0.0; // Boundary
    }

    let f_center = field[idx];

    let f_xp = field[(i + 1) + nx * (j + ny * k)];
    let f_xm = field[(i - 1) + nx * (j + ny * k)];

    let f_yp = field[i + nx * ((j + 1) + ny * k)];
    let f_ym = field[i + nx * ((j - 1) + ny * k)];

    let f_zp = field[i + nx * (j + ny * (k + 1))];
    let f_zm = field[i + nx * (j + ny * (k - 1))];

    let dfx_dx = (f_xp.x - f_xm.x) / (2.0 * dx);
    let dfy_dy = (f_yp.y - f_ym.y) / (2.0 * dx);
    let dfz_dz = (f_zp.z - f_zm.z) / (2.0 * dx);

    dfx_dx + dfy_dy + dfz_dz
}

/// Calculate the curl of a vector field at a point using finite differences.
///
/// ∇×F = (∂Fz/∂y - ∂Fy/∂z, ∂Fx/∂z - ∂Fz/∂x, ∂Fy/∂x - ∂Fx/∂y)
pub fn curl(
    field: &[Vec3],
    i: usize,
    j: usize,
    k: usize,
    nx: usize,
    ny: usize,
    nz: usize,
    dx: f32,
) -> Vec3 {
    let idx = i + nx * (j + ny * k);

    if i == 0 || i >= nx - 1 || j == 0 || j >= ny - 1 || k == 0 || k >= nz - 1 {
        return Vec3::ZERO; // Boundary
    }

    let f_xp = field[(i + 1) + nx * (j + ny * k)];
    let f_xm = field[(i - 1) + nx * (j + ny * k)];

    let f_yp = field[i + nx * ((j + 1) + ny * k)];
    let f_ym = field[i + nx * ((j - 1) + ny * k)];

    let f_zp = field[i + nx * (j + ny * (k + 1))];
    let f_zm = field[i + nx * (j + ny * (k - 1))];

    let curl_x = (f_yp.z - f_ym.z) / (2.0 * dx) - (f_zp.y - f_zm.y) / (2.0 * dx);
    let curl_y = (f_zp.x - f_zm.x) / (2.0 * dx) - (f_xp.z - f_xm.z) / (2.0 * dx);
    let curl_z = (f_xp.y - f_xm.y) / (2.0 * dx) - (f_yp.x - f_ym.x) / (2.0 * dx);

    Vec3::new(curl_x, curl_y, curl_z)
}

/// Linear interpolation
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Clamp a value between min and max
#[inline]
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

/// Calculate RMS (root mean square) of a field
pub fn rms(field: &[Vec3]) -> f32 {
    let sum_sq: f32 = field.iter().map(|v| v.length_squared()).sum();
    (sum_sq / field.len() as f32).sqrt()
}

/// Calculate maximum field magnitude
pub fn max_magnitude(field: &[Vec3]) -> f32 {
    field
        .iter()
        .map(|v| v.length())
        .fold(0.0f32, |a, b| a.max(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_lerp() {
        assert_relative_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_relative_eq!(lerp(0.0, 10.0, 1.0), 10.0);
        assert_relative_eq!(lerp(0.0, 10.0, 0.5), 5.0);
    }

    #[test]
    fn test_clamp() {
        assert_relative_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_relative_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
        assert_relative_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn test_rms() {
        let field = vec![Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)];
        let rms_val = rms(&field);
        assert_relative_eq!(rms_val, 1.0 / 2.0f32.sqrt(), epsilon = 1e-6);
    }

    #[test]
    fn test_max_magnitude() {
        let field = vec![
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(0.0, 0.0, 1.5),
        ];
        assert_relative_eq!(max_magnitude(&field), 2.0, epsilon = 1e-6);
    }

    #[test]
    fn test_divergence_zero_field() {
        let nx = 10;
        let ny = 10;
        let nz = 10;
        let field = vec![Vec3::ZERO; nx * ny * nz];

        let div = divergence(&field, 5, 5, 5, nx, ny, nz, 0.1);
        assert_relative_eq!(div, 0.0, epsilon = 1e-6);
    }

    #[test]
    fn test_curl_zero_field() {
        let nx = 10;
        let ny = 10;
        let nz = 10;
        let field = vec![Vec3::ZERO; nx * ny * nz];

        let curl_val = curl(&field, 5, 5, 5, nx, ny, nz, 0.1);
        assert_eq!(curl_val, Vec3::ZERO);
    }
}
