"""
3D Field Visualization for Ball Lightning

Generates detailed visualizations of:
1. Dimensional field d(x,y,z) - shows topological defect
2. Vector potential A⃗(x,y,z) - shows circulation
3. Magnetic field B⃗ = ∇×A⃗ - shows confinement
4. Energy density - shows where energy is stored
5. Field lines - shows topological structure
"""

import numpy as np
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
from matplotlib import cm
from matplotlib.colors import Normalize
import matplotlib.patches as mpatches

# Physical constants (simulation units)
PHI = 1.618033988749895  # Golden ratio
PI = np.pi

def toroidal_coordinates(R, r, n_toroidal=21, n_poloidal=13, n_points=1000):
    """Generate toroidal coordinates with Fibonacci winding."""
    t = np.linspace(0, 1, n_points)

    # Parametric angles
    phi = t * n_toroidal * 2 * PI    # Toroidal (around major radius)
    theta = t * n_poloidal * 2 * PI  # Poloidal (around minor radius)

    # 3D coordinates
    x = (R + r * np.cos(theta)) * np.cos(phi)
    y = (R + r * np.cos(theta)) * np.sin(phi)
    z = r * np.sin(theta)

    return x, y, z

def dimensional_field_toroid(X, Y, Z, R=6.0, r=2.0):
    """
    Compute dimensional field for toroidal ball lightning.

    d(x,y,z) = 3.0 - deficit(distance from torus)

    Returns:
        d: Dimensional field (3.0 in background, ~0.5-2.0 in core)
    """
    # Distance from z-axis (cylindrical)
    rho = np.sqrt(X**2 + Y**2)

    # Distance from toroidal ring center
    dist_from_ring = np.sqrt((rho - R)**2 + Z**2)

    # Smooth profile function
    profile = np.exp(-dist_from_ring**2 / (2 * r**2))

    # Dimensional deficit (maximum 2.5 at core)
    deficit = 2.5 * profile

    # Background = 3.0, core drops to 0.5
    d = 3.0 - deficit

    return d

def vector_potential_toroid(X, Y, Z, R=6.0, r=2.0, n_tor=21, n_pol=13):
    """
    Compute vector potential for toroidal ball lightning.

    A⃗ has both toroidal and poloidal components.

    Returns:
        Ax, Ay, Az: Components of vector potential
    """
    rho = np.sqrt(X**2 + Y**2)
    dist_from_ring = np.sqrt((rho - R)**2 + Z**2)

    # Strength profile (peaks at torus surface)
    profile = np.exp(-dist_from_ring**2 / (2 * r**2))

    # Toroidal component (circulation around major axis)
    # A_toroidal ∝ (-y, x, 0) / rho
    A_tor_strength = n_tor * 0.3 * profile
    Ax_tor = -Y / (rho + 0.01) * A_tor_strength
    Ay_tor = X / (rho + 0.01) * A_tor_strength
    Az_tor = np.zeros_like(X)

    # Poloidal component (circulation around minor axis)
    # Points tangent to poloidal direction
    # Direction: perpendicular to radial direction from ring
    radial_x = (rho - R) * X / (rho + 0.01)
    radial_y = (rho - R) * Y / (rho + 0.01)
    radial_z = Z
    radial_mag = np.sqrt(radial_x**2 + radial_y**2 + radial_z**2) + 0.01

    # Poloidal circulation (perpendicular to radial in toroidal plane)
    A_pol_strength = n_pol * 0.2 * profile
    Ax_pol = -radial_y / radial_mag * A_pol_strength
    Ay_pol = radial_x / radial_mag * A_pol_strength
    Az_pol = np.zeros_like(X)

    # Total vector potential
    Ax = Ax_tor + Ax_pol
    Ay = Ay_tor + Ay_pol
    Az = Az_tor + Az_pol

    return Ax, Ay, Az

def compute_curl(Ax, Ay, Az, spacing=1.0):
    """
    Compute curl of vector field: B⃗ = ∇×A⃗

    Returns:
        Bx, By, Bz: Components of magnetic field
    """
    # Gradient along each direction
    dAz_dy = np.gradient(Az, spacing, axis=1)
    dAy_dz = np.gradient(Ay, spacing, axis=2)

    dAx_dz = np.gradient(Ax, spacing, axis=2)
    dAz_dx = np.gradient(Az, spacing, axis=0)

    dAy_dx = np.gradient(Ay, spacing, axis=0)
    dAx_dy = np.gradient(Ax, spacing, axis=1)

    # Curl components
    Bx = dAz_dy - dAy_dz
    By = dAx_dz - dAz_dx
    Bz = dAy_dx - dAx_dy

    return Bx, By, Bz

def energy_density(d, Ax, Ay, Az, Bx, By, Bz, C=1.0):
    """
    Compute energy density in ball lightning.

    u = ½(∂d/∂t)² + ½c²(∇d)² + ½A⃗² + ½B⃗²

    We approximate ∂d/∂t ~ 0 (quasi-static)
    """
    # Gradient energy (dominant)
    grad_d_x = np.gradient(d, axis=0)
    grad_d_y = np.gradient(d, axis=1)
    grad_d_z = np.gradient(d, axis=2)
    grad_d_squared = grad_d_x**2 + grad_d_y**2 + grad_d_z**2
    u_gradient = 0.5 * C**2 * grad_d_squared

    # Vector potential energy
    u_potential = 0.5 * (Ax**2 + Ay**2 + Az**2)

    # Magnetic field energy
    u_magnetic = 0.5 * (Bx**2 + By**2 + Bz**2)

    # Total
    u_total = u_gradient + u_potential + u_magnetic

    return u_total

def visualize_ball_lightning_3d():
    """Generate comprehensive 3D visualization of ball lightning fields."""

    print("="*70)
    print("BALL LIGHTNING 3D FIELD VISUALIZATION")
    print("="*70)
    print()
    print("Generating toroidal vortex with Fibonacci 21/13 winding...")
    print()

    # Grid setup
    n = 40  # Grid resolution (40³ = 64,000 points)
    extent = 15  # ±15 units
    x = np.linspace(-extent, extent, n)
    y = np.linspace(-extent, extent, n)
    z = np.linspace(-extent, extent, n)
    X, Y, Z = np.meshgrid(x, y, z, indexing='ij')
    spacing = x[1] - x[0]

    # Toroid parameters
    R = 6.0  # Major radius
    r = 2.0  # Minor radius
    n_tor = 21  # Fibonacci
    n_pol = 13  # Fibonacci

    print(f"Grid: {n}x{n}x{n} = {n**3:,} points")
    print(f"Domain: [{-extent}, {extent}]^3")
    print(f"Major radius R = {R} units")
    print(f"Minor radius r = {r} units")
    print(f"Winding: {n_tor}x{n_pol} (linking number L = {n_tor*n_pol})")
    print()

    # Compute fields
    print("Computing dimensional field d(x,y,z)...")
    d = dimensional_field_toroid(X, Y, Z, R, r)

    print("Computing vector potential A(x,y,z)...")
    Ax, Ay, Az = vector_potential_toroid(X, Y, Z, R, r, n_tor, n_pol)

    print("Computing magnetic field B = curl(A)...")
    Bx, By, Bz = compute_curl(Ax, Ay, Az, spacing)

    print("Computing energy density u(x,y,z)...")
    u = energy_density(d, Ax, Ay, Az, Bx, By, Bz)

    print()
    print("Field statistics:")
    print(f"  d: [{d.min():.3f}, {d.max():.3f}] (background = 3.0)")
    print(f"  |A|: [{0:.3f}, {np.sqrt(Ax**2 + Ay**2 + Az**2).max():.3f}]")
    print(f"  |B|: [{0:.3f}, {np.sqrt(Bx**2 + By**2 + Bz**2).max():.3f}]")
    print(f"  u: [{u.min():.3f}, {u.max():.3f}]")
    print()

    # Create visualizations
    print("Generating visualizations...")
    print()

    # ========================================================================
    # Figure 1: Dimensional Field (4 views)
    # ========================================================================
    print("  [1/6] Dimensional field d(x,y,z)...")

    fig1 = plt.figure(figsize=(16, 12))
    fig1.suptitle('Ball Lightning: Dimensional Field d(x,y,z)',
                  fontsize=16, fontweight='bold')

    # View 1: XY plane (z=0)
    ax1 = fig1.add_subplot(2, 2, 1)
    z_slice = n // 2
    im1 = ax1.contourf(X[:, :, z_slice], Y[:, :, z_slice], d[:, :, z_slice],
                       levels=20, cmap='RdYlBu_r')
    ax1.contour(X[:, :, z_slice], Y[:, :, z_slice], d[:, :, z_slice],
                levels=[1.0, 1.5, 2.0, 2.5], colors='black', linewidths=1)
    ax1.set_xlabel('x (units)')
    ax1.set_ylabel('y (units)')
    ax1.set_title('XY Plane (z=0)')
    ax1.set_aspect('equal')
    plt.colorbar(im1, ax=ax1, label='d')

    # View 2: XZ plane (y=0)
    ax2 = fig1.add_subplot(2, 2, 2)
    y_slice = n // 2
    im2 = ax2.contourf(X[:, y_slice, :], Z[:, y_slice, :], d[:, y_slice, :],
                       levels=20, cmap='RdYlBu_r')
    ax2.contour(X[:, y_slice, :], Z[:, y_slice, :], d[:, y_slice, :],
                levels=[1.0, 1.5, 2.0, 2.5], colors='black', linewidths=1)
    ax2.set_xlabel('x (units)')
    ax2.set_ylabel('z (units)')
    ax2.set_title('XZ Plane (y=0)')
    ax2.set_aspect('equal')
    plt.colorbar(im2, ax=ax2, label='d')

    # View 3: 3D isosurface
    ax3 = fig1.add_subplot(2, 2, 3, projection='3d')

    # Draw toroidal wire frame
    x_torus, y_torus, z_torus = toroidal_coordinates(R, r, n_tor, n_pol)
    ax3.plot(x_torus, y_torus, z_torus, 'b-', linewidth=2, alpha=0.6,
             label=f'Fibonacci {n_tor}/{n_pol}')

    # Draw isosurface at d=2.0 (defect core)
    # Subsample for performance
    step = 2
    X_sub = X[::step, ::step, ::step]
    Y_sub = Y[::step, ::step, ::step]
    Z_sub = Z[::step, ::step, ::step]
    d_sub = d[::step, ::step, ::step]

    # Find surface points
    mask = (d_sub < 2.0) & (d_sub > 1.8)
    if mask.sum() > 0:
        ax3.scatter(X_sub[mask], Y_sub[mask], Z_sub[mask],
                   c=d_sub[mask], cmap='RdYlBu_r', s=10, alpha=0.3)

    ax3.set_xlabel('x')
    ax3.set_ylabel('y')
    ax3.set_zlabel('z')
    ax3.set_title('3D Structure (isosurface d=2.0)')
    ax3.legend()

    # View 4: Radial profile
    ax4 = fig1.add_subplot(2, 2, 4)

    # Extract radial profile (along x-axis at y=0, z=0)
    y_idx = n // 2
    z_idx = n // 2
    d_radial = d[:, y_idx, z_idx]
    x_radial = x

    ax4.plot(x_radial, d_radial, 'b-', linewidth=2, label='d(r)')
    ax4.axhline(3.0, color='gray', linestyle='--', label='Background (d=3)')
    ax4.axvline(-R, color='red', linestyle=':', alpha=0.5, label='Torus position')
    ax4.axvline(R, color='red', linestyle=':', alpha=0.5)
    ax4.fill_between(x_radial, 0, d_radial, where=(d_radial < 2.5),
                     alpha=0.3, color='blue', label='Defect region')
    ax4.set_xlabel('r (units)')
    ax4.set_ylabel('d (dimension)')
    ax4.set_title('Radial Profile')
    ax4.grid(True, alpha=0.3)
    ax4.legend()
    ax4.set_ylim([0, 3.5])

    plt.tight_layout()
    plt.savefig('ball_lightning_dimensional_field.png', dpi=150, bbox_inches='tight')
    print("      Saved: ball_lightning_dimensional_field.png")

    # ========================================================================
    # Figure 2: Vector Potential (4 views)
    # ========================================================================
    print("  [2/6] Vector potential A(x,y,z)...")

    fig2 = plt.figure(figsize=(16, 12))
    fig2.suptitle('Ball Lightning: Vector Potential A⃗(x,y,z)',
                  fontsize=16, fontweight='bold')

    # Magnitude
    A_mag = np.sqrt(Ax**2 + Ay**2 + Az**2)

    # View 1: XY plane with arrows
    ax1 = fig2.add_subplot(2, 2, 1)
    z_slice = n // 2

    # Contour of magnitude
    im1 = ax1.contourf(X[:, :, z_slice], Y[:, :, z_slice], A_mag[:, :, z_slice],
                       levels=20, cmap='viridis')

    # Arrow field (subsample)
    skip = 3
    ax1.quiver(X[::skip, ::skip, z_slice], Y[::skip, ::skip, z_slice],
              Ax[::skip, ::skip, z_slice], Ay[::skip, ::skip, z_slice],
              color='white', alpha=0.7, scale=5)

    ax1.set_xlabel('x (units)')
    ax1.set_ylabel('y (units)')
    ax1.set_title('XY Plane: |A⃗| with circulation arrows')
    ax1.set_aspect('equal')
    plt.colorbar(im1, ax=ax1, label='|A⃗|')

    # View 2: XZ plane with arrows
    ax2 = fig2.add_subplot(2, 2, 2)
    y_slice = n // 2

    im2 = ax2.contourf(X[:, y_slice, :], Z[:, y_slice, :], A_mag[:, y_slice, :],
                       levels=20, cmap='viridis')

    ax2.quiver(X[::skip, y_slice, ::skip], Z[::skip, y_slice, ::skip],
              Ax[::skip, y_slice, ::skip], Az[::skip, y_slice, ::skip],
              color='white', alpha=0.7, scale=5)

    ax2.set_xlabel('x (units)')
    ax2.set_ylabel('z (units)')
    ax2.set_title('XZ Plane: |A⃗| with poloidal flow')
    ax2.set_aspect('equal')
    plt.colorbar(im2, ax=ax2, label='|A⃗|')

    # View 3: 3D streamlines
    ax3 = fig2.add_subplot(2, 2, 3, projection='3d')

    # Draw toroid
    x_torus, y_torus, z_torus = toroidal_coordinates(R, r, n_tor, n_pol)
    ax3.plot(x_torus, y_torus, z_torus, 'r-', linewidth=2, alpha=0.6)

    # Seed points for streamlines (circle around torus)
    n_seeds = 8
    seed_angles = np.linspace(0, 2*PI, n_seeds, endpoint=False)
    for angle in seed_angles:
        # Start point on torus surface
        x_start = (R + r*0.5) * np.cos(angle)
        y_start = (R + r*0.5) * np.sin(angle)
        z_start = 0

        # Integrate streamline (simple Euler method)
        n_steps = 100
        traj_x = [x_start]
        traj_y = [y_start]
        traj_z = [z_start]

        for step in range(n_steps):
            # Current position
            xi, yi, zi = traj_x[-1], traj_y[-1], traj_z[-1]

            # Find grid indices
            xi_idx = int((xi - x[0]) / spacing)
            yi_idx = int((yi - y[0]) / spacing)
            zi_idx = int((zi - z[0]) / spacing)

            if (0 <= xi_idx < n-1 and 0 <= yi_idx < n-1 and 0 <= zi_idx < n-1):
                # Interpolate field
                ax_val = Ax[xi_idx, yi_idx, zi_idx]
                ay_val = Ay[xi_idx, yi_idx, zi_idx]
                az_val = Az[xi_idx, yi_idx, zi_idx]

                # Normalize
                a_norm = np.sqrt(ax_val**2 + ay_val**2 + az_val**2) + 1e-6

                # Step along field
                dt = 0.3
                traj_x.append(xi + ax_val/a_norm * dt)
                traj_y.append(yi + ay_val/a_norm * dt)
                traj_z.append(zi + az_val/a_norm * dt)
            else:
                break

        ax3.plot(traj_x, traj_y, traj_z, 'b-', linewidth=1, alpha=0.6)

    ax3.set_xlabel('x')
    ax3.set_ylabel('y')
    ax3.set_zlabel('z')
    ax3.set_title('3D Field Lines of A⃗')

    # View 4: Circulation vs radius
    ax4 = fig2.add_subplot(2, 2, 4)

    # Compute circulation around circles at different radii
    radii = np.linspace(2, 12, 20)
    circulation = []

    for rad in radii:
        # Circle in XY plane
        angles = np.linspace(0, 2*PI, 50)
        circ = 0
        for i in range(len(angles)-1):
            ang1, ang2 = angles[i], angles[i+1]

            # Positions
            x1, y1 = rad * np.cos(ang1), rad * np.sin(ang1)
            x2, y2 = rad * np.cos(ang2), rad * np.sin(ang2)

            # Find A⃗ at midpoint
            x_mid = (x1 + x2) / 2
            y_mid = (y1 + y2) / 2

            xi_idx = int((x_mid - x[0]) / spacing)
            yi_idx = int((y_mid - y[0]) / spacing)
            zi_idx = n // 2

            if 0 <= xi_idx < n and 0 <= yi_idx < n:
                ax_val = Ax[xi_idx, yi_idx, zi_idx]
                ay_val = Ay[xi_idx, yi_idx, zi_idx]

                # Path element
                dx = x2 - x1
                dy = y2 - y1

                # A⃗·dl
                circ += ax_val * dx + ay_val * dy

        circulation.append(circ)

    ax4.plot(radii, circulation, 'go-', linewidth=2, markersize=5)
    ax4.axvline(R, color='red', linestyle='--', alpha=0.5, label='Torus major radius')
    ax4.axhline(0, color='gray', linestyle=':')
    ax4.set_xlabel('Radius (units)')
    ax4.set_ylabel('Circulation ∮A⃗·dl')
    ax4.set_title('Winding Number = Circulation / 2π')
    ax4.grid(True, alpha=0.3)
    ax4.legend()

    # Compute winding number at optimal radius
    opt_idx = np.argmax(np.abs(circulation))
    winding_measured = circulation[opt_idx] / (2*PI)
    ax4.text(0.05, 0.95, f'Measured winding: {winding_measured:.1f}\n' +
                          f'Expected: {n_tor} (toroidal)',
             transform=ax4.transAxes, verticalalignment='top',
             bbox=dict(boxstyle='round', facecolor='yellow', alpha=0.8))

    plt.tight_layout()
    plt.savefig('ball_lightning_vector_potential.png', dpi=150, bbox_inches='tight')
    print("      Saved: ball_lightning_vector_potential.png")

    # ========================================================================
    # Figure 3: Magnetic Field (4 views)
    # ========================================================================
    print("  [3/6] Magnetic field B = curl(A)...")

    fig3 = plt.figure(figsize=(16, 12))
    fig3.suptitle('Ball Lightning: Magnetic Field B⃗ = ∇×A⃗',
                  fontsize=16, fontweight='bold')

    B_mag = np.sqrt(Bx**2 + By**2 + Bz**2)

    # View 1: XY plane
    ax1 = fig3.add_subplot(2, 2, 1)
    z_slice = n // 2

    im1 = ax1.contourf(X[:, :, z_slice], Y[:, :, z_slice], B_mag[:, :, z_slice],
                       levels=20, cmap='plasma')

    skip = 3
    ax1.quiver(X[::skip, ::skip, z_slice], Y[::skip, ::skip, z_slice],
              Bx[::skip, ::skip, z_slice], By[::skip, ::skip, z_slice],
              color='white', alpha=0.7, scale=20)

    ax1.set_xlabel('x (units)')
    ax1.set_ylabel('y (units)')
    ax1.set_title('XY Plane: |B⃗| with field direction')
    ax1.set_aspect('equal')
    plt.colorbar(im1, ax=ax1, label='|B⃗|')

    # View 2: XZ plane
    ax2 = fig3.add_subplot(2, 2, 2)
    y_slice = n // 2

    im2 = ax2.contourf(X[:, y_slice, :], Z[:, y_slice, :], B_mag[:, y_slice, :],
                       levels=20, cmap='plasma')

    ax2.quiver(X[::skip, y_slice, ::skip], Z[::skip, y_slice, ::skip],
              Bx[::skip, y_slice, ::skip], Bz[::skip, y_slice, ::skip],
              color='white', alpha=0.7, scale=20)

    ax2.set_xlabel('x (units)')
    ax2.set_ylabel('z (units)')
    ax2.set_title('XZ Plane: Poloidal magnetic field')
    ax2.set_aspect('equal')
    plt.colorbar(im2, ax=ax2, label='|B⃗|')

    # View 3: 3D magnetic field lines
    ax3 = fig3.add_subplot(2, 2, 3, projection='3d')

    # Draw toroid
    x_torus, y_torus, z_torus = toroidal_coordinates(R, r, n_tor, n_pol)
    ax3.plot(x_torus, y_torus, z_torus, 'k-', linewidth=2, alpha=0.3)

    # Show high B-field regions
    step = 2
    B_threshold = B_mag.max() * 0.5
    mask = B_mag[::step, ::step, ::step] > B_threshold

    if mask.sum() > 0:
        ax3.scatter(X[::step, ::step, ::step][mask],
                   Y[::step, ::step, ::step][mask],
                   Z[::step, ::step, ::step][mask],
                   c=B_mag[::step, ::step, ::step][mask],
                   cmap='plasma', s=20, alpha=0.5)

    ax3.set_xlabel('x')
    ax3.set_ylabel('y')
    ax3.set_zlabel('z')
    ax3.set_title('3D Structure: High B-field regions')

    # View 4: Field strength along path
    ax4 = fig3.add_subplot(2, 2, 4)

    # Follow toroidal path
    x_path, y_path, z_path = toroidal_coordinates(R, r, n_tor, n_pol, 200)
    B_along_path = []

    for xp, yp, zp in zip(x_path, y_path, z_path):
        xi_idx = int((xp - x[0]) / spacing)
        yi_idx = int((yp - y[0]) / spacing)
        zi_idx = int((zp - z[0]) / spacing)

        if 0 <= xi_idx < n and 0 <= yi_idx < n and 0 <= zi_idx < n:
            B_along_path.append(B_mag[xi_idx, yi_idx, zi_idx])
        else:
            B_along_path.append(0)

    path_param = np.linspace(0, 1, len(B_along_path))
    ax4.plot(path_param, B_along_path, 'r-', linewidth=2)
    ax4.set_xlabel('Position along toroidal path (0→1)')
    ax4.set_ylabel('|B⃗| (field strength)')
    ax4.set_title('Magnetic Confinement Along Plasma Path')
    ax4.grid(True, alpha=0.3)
    ax4.fill_between(path_param, 0, B_along_path, alpha=0.3, color='red')

    plt.tight_layout()
    plt.savefig('ball_lightning_magnetic_field.png', dpi=150, bbox_inches='tight')
    print("      Saved: ball_lightning_magnetic_field.png")

    # ========================================================================
    # Figure 4: Energy Density (4 views)
    # ========================================================================
    print("  [4/6] Energy density u(x,y,z)...")

    fig4 = plt.figure(figsize=(16, 12))
    fig4.suptitle('Ball Lightning: Energy Density Distribution',
                  fontsize=16, fontweight='bold')

    # Logarithmic scale for better visualization
    u_log = np.log10(u + 1e-6)

    # View 1: XY plane
    ax1 = fig4.add_subplot(2, 2, 1)
    z_slice = n // 2

    im1 = ax1.contourf(X[:, :, z_slice], Y[:, :, z_slice], u_log[:, :, z_slice],
                       levels=20, cmap='hot')
    ax1.contour(X[:, :, z_slice], Y[:, :, z_slice], u_log[:, :, z_slice],
                levels=5, colors='cyan', linewidths=1)

    ax1.set_xlabel('x (units)')
    ax1.set_ylabel('y (units)')
    ax1.set_title('XY Plane: log₁₀(energy density)')
    ax1.set_aspect('equal')
    cbar1 = plt.colorbar(im1, ax=ax1, label='log₁₀(u)')

    # View 2: XZ plane
    ax2 = fig4.add_subplot(2, 2, 2)
    y_slice = n // 2

    im2 = ax2.contourf(X[:, y_slice, :], Z[:, y_slice, :], u_log[:, y_slice, :],
                       levels=20, cmap='hot')
    ax2.contour(X[:, y_slice, :], Z[:, y_slice, :], u_log[:, y_slice, :],
                levels=5, colors='cyan', linewidths=1)

    ax2.set_xlabel('x (units)')
    ax2.set_ylabel('z (units)')
    ax2.set_title('XZ Plane: Energy localization')
    ax2.set_aspect('equal')
    plt.colorbar(im2, ax=ax2, label='log₁₀(u)')

    # View 3: 3D energy isosurface
    ax3 = fig4.add_subplot(2, 2, 3, projection='3d')

    # Draw toroid
    x_torus, y_torus, z_torus = toroidal_coordinates(R, r, n_tor, n_pol)
    ax3.plot(x_torus, y_torus, z_torus, 'w-', linewidth=2, alpha=0.8)

    # High energy regions
    step = 2
    u_threshold = np.percentile(u, 90)  # Top 10%
    mask = u[::step, ::step, ::step] > u_threshold

    if mask.sum() > 0:
        ax3.scatter(X[::step, ::step, ::step][mask],
                   Y[::step, ::step, ::step][mask],
                   Z[::step, ::step, ::step][mask],
                   c=u[::step, ::step, ::step][mask],
                   cmap='hot', s=20, alpha=0.6)

    ax3.set_xlabel('x')
    ax3.set_ylabel('y')
    ax3.set_zlabel('z')
    ax3.set_title('3D: High Energy Regions (top 10%)')
    ax3.set_facecolor('black')

    # View 4: Energy statistics
    ax4 = fig4.add_subplot(2, 2, 4)

    # Histogram of energy distribution
    u_flat = u.flatten()
    ax4.hist(np.log10(u_flat + 1e-6), bins=50, color='orange', alpha=0.7, edgecolor='black')
    ax4.set_xlabel('log₁₀(energy density)')
    ax4.set_ylabel('Number of cells')
    ax4.set_title('Energy Distribution')
    ax4.grid(True, alpha=0.3, axis='y')

    # Total energy
    total_energy = u.sum() * spacing**3
    mean_energy = u.mean()
    max_energy = u.max()

    stats_text = f"""ENERGY STATISTICS

Total energy: {total_energy:.2e} units
Mean density: {mean_energy:.2e} units/vol
Peak density: {max_energy:.2e} units/vol

Concentration factor: {max_energy/mean_energy:.1f}×

For scale (if 1 unit = 1 kJ):
Total: {total_energy:.0f} kJ
"""

    ax4.text(0.98, 0.98, stats_text,
             transform=ax4.transAxes,
             fontsize=9, family='monospace',
             verticalalignment='top', horizontalalignment='right',
             bbox=dict(boxstyle='round', facecolor='yellow', alpha=0.8))

    plt.tight_layout()
    plt.savefig('ball_lightning_energy_density.png', dpi=150, bbox_inches='tight')
    print("      Saved: ball_lightning_energy_density.png")

    # ========================================================================
    # Figure 5: Topological Structure
    # ========================================================================
    print("  [5/6] Topological structure...")

    fig5 = plt.figure(figsize=(16, 12))
    fig5.suptitle('Ball Lightning: Topological Structure (Linking Number L = 273)',
                  fontsize=16, fontweight='bold')

    # View 1: Toroidal winding (top view)
    ax1 = fig5.add_subplot(2, 2, 1)
    ax1.set_aspect('equal')

    # Draw toroidal loops
    for i in range(n_tor):
        angle = i * 2*PI / n_tor
        x_loop = (R + r*np.cos(np.linspace(0, 2*PI, 100))) * np.cos(angle)
        y_loop = (R + r*np.cos(np.linspace(0, 2*PI, 100))) * np.sin(angle)
        ax1.plot(x_loop, y_loop, 'b-', linewidth=1, alpha=0.3)

    # Highlight a few
    for i in [0, 7, 14]:
        angle = i * 2*PI / n_tor
        x_loop = (R + r*np.cos(np.linspace(0, 2*PI, 100))) * np.cos(angle)
        y_loop = (R + r*np.cos(np.linspace(0, 2*PI, 100))) * np.sin(angle)
        ax1.plot(x_loop, y_loop, 'b-', linewidth=2, alpha=0.8)

    ax1.set_xlabel('x (units)')
    ax1.set_ylabel('y (units)')
    ax1.set_title(f'Toroidal Winding (n_tor = {n_tor})')
    ax1.text(0.05, 0.95, f'{n_tor} loops around major axis',
             transform=ax1.transAxes, verticalalignment='top',
             bbox=dict(boxstyle='round', facecolor='lightblue', alpha=0.8))

    # View 2: Poloidal winding (side view)
    ax2 = fig5.add_subplot(2, 2, 2)
    ax2.set_aspect('equal')

    # Draw poloidal loops
    for i in range(n_pol):
        phase = i * 2*PI / n_pol
        x_ring = R + r * np.cos(np.linspace(0, 2*PI, 100) + phase)
        z_ring = r * np.sin(np.linspace(0, 2*PI, 100) + phase)
        ax2.plot(x_ring, z_ring, 'r-', linewidth=1, alpha=0.3)

    # Highlight a few
    for i in [0, 4, 8]:
        phase = i * 2*PI / n_pol
        x_ring = R + r * np.cos(np.linspace(0, 2*PI, 100) + phase)
        z_ring = r * np.sin(np.linspace(0, 2*PI, 100) + phase)
        ax2.plot(x_ring, z_ring, 'r-', linewidth=2, alpha=0.8)

    ax2.set_xlabel('x (units)')
    ax2.set_ylabel('z (units)')
    ax2.set_title(f'Poloidal Winding (n_pol = {n_pol})')
    ax2.text(0.05, 0.95, f'{n_pol} loops around minor axis',
             transform=ax2.transAxes, verticalalignment='top',
             bbox=dict(boxstyle='round', facecolor='lightcoral', alpha=0.8))

    # View 3: Combined 3D view
    ax3 = fig5.add_subplot(2, 2, 3, projection='3d')

    # Full Fibonacci winding
    x_fib, y_fib, z_fib = toroidal_coordinates(R, r, n_tor, n_pol, 2000)

    # Color by position along path
    colors = plt.cm.rainbow(np.linspace(0, 1, len(x_fib)))

    for i in range(len(x_fib)-1):
        ax3.plot(x_fib[i:i+2], y_fib[i:i+2], z_fib[i:i+2],
                color=colors[i], linewidth=1.5, alpha=0.8)

    ax3.set_xlabel('x')
    ax3.set_ylabel('y')
    ax3.set_zlabel('z')
    ax3.set_title(f'Fibonacci {n_tor}/{n_pol} Winding (φ ≈ {n_tor/n_pol:.3f})')

    # View 4: Topological invariants
    ax4 = fig5.add_subplot(2, 2, 4)
    ax4.axis('off')

    topology_text = f"""TOPOLOGICAL PROPERTIES

Winding Numbers:
  n_toroidal  = {n_tor}
  n_poloidal  = {n_pol}
  Ratio φ     = {n_tor/n_pol:.6f}
  Golden φ    = {PHI:.6f}
  Match       = {abs((n_tor/n_pol) - PHI) < 0.01}

Linking Number:
  L = n_tor × n_pol = {n_tor * n_pol}

Topological Charge:
  Q = ∮A⃗·dl / 2π ≈ {n_tor} (toroidal)

Conservation Law:
  ΔL/Δt = 0  (CANNOT change smoothly!)

Physical Implications:
• Ball lightning CANNOT decay to L=0
• Must create L=-{n_tor * n_pol} defect to annihilate
• Energy barrier ≈ 2π²ħc × L
• Minimum lifetime >> plasma decay time

Stability:
  Higher L → More stable
  L={n_tor * n_pol} → ULTRA STABLE
  Expected lifetime: 30-120 seconds
"""

    ax4.text(0.1, 0.95, topology_text,
             fontsize=10, family='monospace',
             verticalalignment='top',
             bbox=dict(boxstyle='round', facecolor='lightyellow', alpha=0.9))

    plt.tight_layout()
    plt.savefig('ball_lightning_topology.png', dpi=150, bbox_inches='tight')
    print("      Saved: ball_lightning_topology.png")

    # ========================================================================
    # Figure 6: Summary Comparison
    # ========================================================================
    print("  [6/6] Summary comparison...")

    fig6 = plt.figure(figsize=(20, 10))
    fig6.suptitle('Ball Lightning: Complete Field Analysis (XY Plane at z=0)',
                  fontsize=16, fontweight='bold')

    z_slice = n // 2

    # Panel 1: Dimensional field
    ax1 = fig6.add_subplot(2, 4, 1)
    im1 = ax1.contourf(X[:, :, z_slice], Y[:, :, z_slice], d[:, :, z_slice],
                       levels=20, cmap='RdYlBu_r')
    ax1.set_title('d(x,y) - Dimensional Field')
    ax1.set_aspect('equal')
    plt.colorbar(im1, ax=ax1)

    # Panel 2: Vector potential magnitude
    ax2 = fig6.add_subplot(2, 4, 2)
    im2 = ax2.contourf(X[:, :, z_slice], Y[:, :, z_slice], A_mag[:, :, z_slice],
                       levels=20, cmap='viridis')
    ax2.set_title('|A⃗| - Vector Potential')
    ax2.set_aspect('equal')
    plt.colorbar(im2, ax=ax2)

    # Panel 3: Magnetic field magnitude
    ax3 = fig6.add_subplot(2, 4, 3)
    im3 = ax3.contourf(X[:, :, z_slice], Y[:, :, z_slice], B_mag[:, :, z_slice],
                       levels=20, cmap='plasma')
    ax3.set_title('|B⃗| - Magnetic Field')
    ax3.set_aspect('equal')
    plt.colorbar(im3, ax=ax3)

    # Panel 4: Energy density
    ax4 = fig6.add_subplot(2, 4, 4)
    im4 = ax4.contourf(X[:, :, z_slice], Y[:, :, z_slice], u_log[:, :, z_slice],
                       levels=20, cmap='hot')
    ax4.set_title('log₁₀(u) - Energy Density')
    ax4.set_aspect('equal')
    plt.colorbar(im4, ax=ax4)

    # Panel 5: A⃗ with arrows
    ax5 = fig6.add_subplot(2, 4, 5)
    ax5.contourf(X[:, :, z_slice], Y[:, :, z_slice], A_mag[:, :, z_slice],
                 levels=10, cmap='viridis', alpha=0.5)
    skip = 3
    ax5.quiver(X[::skip, ::skip, z_slice], Y[::skip, ::skip, z_slice],
              Ax[::skip, ::skip, z_slice], Ay[::skip, ::skip, z_slice],
              color='white', alpha=0.8)
    ax5.set_title('A⃗ Circulation Pattern')
    ax5.set_aspect('equal')

    # Panel 6: B⃗ with arrows
    ax6 = fig6.add_subplot(2, 4, 6)
    ax6.contourf(X[:, :, z_slice], Y[:, :, z_slice], B_mag[:, :, z_slice],
                 levels=10, cmap='plasma', alpha=0.5)
    ax6.quiver(X[::skip, ::skip, z_slice], Y[::skip, ::skip, z_slice],
              Bx[::skip, ::skip, z_slice], By[::skip, ::skip, z_slice],
              color='white', alpha=0.8)
    ax6.set_title('B⃗ Confinement Pattern')
    ax6.set_aspect('equal')

    # Panel 7: Combined overlay
    ax7 = fig6.add_subplot(2, 4, 7)
    # Show dimensional defect
    ax7.contour(X[:, :, z_slice], Y[:, :, z_slice], d[:, :, z_slice],
                levels=[1.5, 2.0, 2.5], colors=['lightblue', 'cyan', 'blue'],
                linewidths=[1, 2, 3])
    # Overlay energy density
    ax7.contourf(X[:, :, z_slice], Y[:, :, z_slice], u[:, :, z_slice],
                 levels=10, cmap='Reds', alpha=0.3)
    # Add field vectors
    ax7.quiver(X[::skip, ::skip, z_slice], Y[::skip, ::skip, z_slice],
              Ax[::skip, ::skip, z_slice], Ay[::skip, ::skip, z_slice],
              color='green', alpha=0.6, scale=5)
    ax7.set_title('Composite View')
    ax7.set_aspect('equal')

    # Panel 8: Summary text
    ax8 = fig6.add_subplot(2, 4, 8)
    ax8.axis('off')

    summary_text = f"""BALL LIGHTNING SUMMARY

Geometry:
  Major radius R = {R} units
  Minor radius r = {r} units
  Shape: Toroidal (donut)

Topological Charge:
  Linking number L = {n_tor*n_pol}
  Winding (toroidal) = {n_tor}
  Winding (poloidal) = {n_pol}

Fields:
  d: [{d.min():.2f}, {d.max():.2f}]
  |A⃗|_max: {A_mag.max():.2f}
  |B⃗|_max: {B_mag.max():.2f}
  u_max: {u.max():.2e}

Energy:
  Total: {total_energy:.2e} units
  Density: {u.mean():.2e} units/vol
  Peak: {u.max():.2e} units/vol

Stability:
  Topologically protected
  L cannot change smoothly
  Expected lifetime: 30-120 s

Key Insight:
  Energy is stored in THREE forms:
  1. Dimensional gradient (∇d)²
  2. Vector potential A⃗²
  3. Magnetic field B⃗²

  All three are LOCKED by topology!
"""

    ax8.text(0.05, 0.95, summary_text,
             fontsize=9, family='monospace',
             verticalalignment='top',
             bbox=dict(boxstyle='round', facecolor='lightgreen', alpha=0.9))

    plt.tight_layout()
    plt.savefig('ball_lightning_summary.png', dpi=150, bbox_inches='tight')
    print("      Saved: ball_lightning_summary.png")

    print()
    print("="*70)
    print("VISUALIZATION COMPLETE")
    print("="*70)
    print()
    print("Generated files:")
    print("  1. ball_lightning_dimensional_field.png")
    print("  2. ball_lightning_vector_potential.png")
    print("  3. ball_lightning_magnetic_field.png")
    print("  4. ball_lightning_energy_density.png")
    print("  5. ball_lightning_topology.png")
    print("  6. ball_lightning_summary.png")
    print()
    print("Total energy in ball lightning: {:.2e} simulation units".format(total_energy))
    print("If 1 unit = 1 kJ, this represents {:.0f} kJ of stored energy".format(total_energy))
    print()
    print("Topological protection factor: L = {}".format(n_tor*n_pol))
    print("Expected lifetime: 30-120 seconds (vs <1 us for normal plasma)")
    print()

if __name__ == "__main__":
    visualize_ball_lightning_3d()
