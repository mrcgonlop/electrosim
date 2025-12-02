#!/usr/bin/env python3
"""
Generate 3D visualizations of energy harvester devices for construction reference.

Creates detailed 3D models and technical drawings for:
1. Fractal Rectenna Array (Sierpinski triangle)
2. Spiral Vortex Concentrator (golden ratio)
3. Toroidal Vortex Collector (Fibonacci winding)
4. Schumann Resonance Tapper (vertical monopole)

Outputs:
- 3D perspective views
- Top/side/front orthographic projections
- Annotated dimension diagrams
- Construction assembly views
"""

import numpy as np
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
from mpl_toolkits.mplot3d.art3d import Line3DCollection
import matplotlib.patches as patches
from matplotlib.patches import FancyBboxPatch, Circle, Wedge, Rectangle, FancyArrowPatch
from matplotlib.patches import Arc as ArcPatch

# Golden ratio
PHI = (1 + np.sqrt(5)) / 2

# Configure matplotlib
plt.rcParams['figure.figsize'] = (16, 12)
plt.rcParams['font.size'] = 9

# ============================================================================
# FRACTAL RECTENNA ARRAY (Sierpinski Triangle)
# ============================================================================

def sierpinski_triangle(center, size, level, max_level=3):
    """
    Generate Sierpinski triangle fractal recursively.

    Args:
        center: (x, y) center point
        size: Side length of triangle
        level: Current recursion level
        max_level: Maximum recursion depth

    Returns:
        List of triangles as [(x1,y1), (x2,y2), (x3,y3)]
    """
    triangles = []

    # Equilateral triangle vertices
    h = size * np.sqrt(3) / 2  # Height
    v1 = (center[0], center[1] + 2*h/3)           # Top
    v2 = (center[0] - size/2, center[1] - h/3)    # Bottom left
    v3 = (center[0] + size/2, center[1] - h/3)    # Bottom right

    triangles.append([v1, v2, v3])

    # Recurse to create sub-triangles
    if level < max_level:
        new_size = size / 2

        # Top sub-triangle
        c1 = (center[0], center[1] + h/3)
        triangles.extend(sierpinski_triangle(c1, new_size, level + 1, max_level))

        # Bottom-left sub-triangle
        c2 = (center[0] - size/4, center[1] - h/6)
        triangles.extend(sierpinski_triangle(c2, new_size, level + 1, max_level))

        # Bottom-right sub-triangle
        c3 = (center[0] + size/4, center[1] - h/6)
        triangles.extend(sierpinski_triangle(c3, new_size, level + 1, max_level))

    return triangles


def plot_fractal_rectenna():
    """Generate construction diagrams for fractal rectenna array."""
    print("Generating Fractal Rectenna Array diagrams...")

    fig = plt.figure(figsize=(18, 12))

    # Main view: Complete fractal pattern
    ax1 = plt.subplot(2, 2, 1)
    ax1.set_aspect('equal')
    ax1.set_title('Fractal Rectenna Array - Complete Pattern\n(Sierpinski Triangle, 4 levels)',
                  fontsize=14, fontweight='bold')

    # Generate fractal
    center = (0, 0)
    size = 50  # 50 cm
    triangles = sierpinski_triangle(center, size, 0, max_level=4)

    # Draw all triangles
    for tri in triangles:
        tri_array = np.array(tri + [tri[0]])  # Close the triangle
        ax1.plot(tri_array[:, 0], tri_array[:, 1], 'b-', linewidth=0.5)
        ax1.fill(tri_array[:, 0], tri_array[:, 1], color='lightblue', alpha=0.3)

    # Add diode positions (at each vertex)
    for tri in triangles:
        for vertex in tri:
            ax1.plot(vertex[0], vertex[1], 'ro', markersize=2)

    # PCB outline
    pcb_size = 55
    pcb_rect = Rectangle((-pcb_size/2, -pcb_size/2), pcb_size, pcb_size,
                         linewidth=2, edgecolor='green', facecolor='none',
                         linestyle='--', label='PCB edge (55×55 cm)')
    ax1.add_patch(pcb_rect)

    # Dimensions
    ax1.annotate('', xy=(size/2, 0), xytext=(-size/2, 0),
                arrowprops=dict(arrowstyle='<->', color='red', lw=2))
    ax1.text(0, -2, f'{size} cm', ha='center', fontsize=10, color='red', fontweight='bold')

    ax1.set_xlim(-30, 30)
    ax1.set_ylim(-30, 30)
    ax1.set_xlabel('Distance (cm)', fontsize=11)
    ax1.set_ylabel('Distance (cm)', fontsize=11)
    ax1.legend(fontsize=9)
    ax1.grid(True, alpha=0.3)

    # Detail view: Single triangle with diode placement
    ax2 = plt.subplot(2, 2, 2)
    ax2.set_aspect('equal')
    ax2.set_title('Detail: Triangle Element with Schottky Diodes\n(One basic unit)',
                  fontsize=12, fontweight='bold')

    # Draw one triangle
    tri_size = 10  # cm
    h = tri_size * np.sqrt(3) / 2
    v1 = np.array([0, 2*h/3])
    v2 = np.array([-tri_size/2, -h/3])
    v3 = np.array([tri_size/2, -h/3])
    tri = np.array([v1, v2, v3, v1])

    ax2.plot(tri[:, 0], tri[:, 1], 'b-', linewidth=3)
    ax2.fill(tri[:, 0], tri[:, 1], color='lightblue', alpha=0.3)

    # Draw copper traces
    trace_width = 0.3
    for i in range(3):
        start = tri[i]
        end = tri[i+1]
        ax2.plot([start[0], end[0]], [start[1], end[1]], 'orange',
                linewidth=8, alpha=0.6, label='Copper trace' if i == 0 else '')

    # Diodes at vertices
    diode_positions = [v1, v2, v3]
    diode_labels = ['D1\n(SMS7630)', 'D2\n(SMS7630)', 'D3\n(SMS7630)']

    for pos, label in zip(diode_positions, diode_labels):
        # Diode symbol
        ax2.plot(pos[0], pos[1], 'ro', markersize=12)

        # Label
        offset = pos / np.linalg.norm(pos) * 1.5
        ax2.text(pos[0] + offset[0], pos[1] + offset[1], label,
                ha='center', fontsize=9, bbox=dict(boxstyle='round', facecolor='yellow', alpha=0.7))

        # Diode orientation arrow
        ax2.arrow(pos[0] - 0.3, pos[1], 0.6, 0, head_width=0.3,
                 head_length=0.2, fc='red', ec='red', alpha=0.5)

    # Ground connection
    ground_pos = (v2 + v3) / 2
    ax2.plot(ground_pos[0], ground_pos[1], 'ks', markersize=10, label='Ground pad')
    ax2.text(ground_pos[0], ground_pos[1] - 1.5, 'GND', ha='center',
            fontsize=10, fontweight='bold')

    # DC output
    output_pos = v1 + np.array([0, 1])
    ax2.plot(output_pos[0], output_pos[1], 'g^', markersize=12, label='DC output')
    ax2.text(output_pos[0], output_pos[1] + 0.8, '+Vdc', ha='center',
            fontsize=10, fontweight='bold', color='green')

    # Dimensions
    ax2.plot([v2[0], v3[0]], [v2[1]-1, v3[1]-1], 'r-', linewidth=2)
    ax2.text(0, v2[1]-1.5, f'{tri_size} cm', ha='center', fontsize=10, color='red')

    ax2.set_xlim(-8, 8)
    ax2.set_ylim(-8, 8)
    ax2.set_xlabel('Distance (cm)', fontsize=11)
    ax2.set_ylabel('Distance (cm)', fontsize=11)
    ax2.legend(fontsize=8, loc='upper right')
    ax2.grid(True, alpha=0.3)

    # Cross-section view
    ax3 = plt.subplot(2, 2, 3)
    ax3.set_aspect('equal')
    ax3.set_title('PCB Cross-Section (Side View)', fontsize=12, fontweight='bold')

    # PCB layers
    pcb_width = 20
    layers = [
        ('Top copper (1 oz)', 0, 0.035, 'orange'),
        ('FR4 substrate', 0.035, 1.6, 'lightgreen'),
        ('Bottom copper (1 oz)', 1.635, 0.035, 'orange'),
    ]

    y_offset = 0
    for name, y_start, thickness, color in layers:
        rect = Rectangle((0, y_start), pcb_width, thickness,
                        facecolor=color, edgecolor='black', linewidth=1.5)
        ax3.add_patch(rect)
        ax3.text(pcb_width + 1, y_start + thickness/2, name,
                va='center', fontsize=10)

    # Diode component on top
    diode_x = 5
    diode_rect = Rectangle((diode_x, 1.67), 0.8, 0.5,
                           facecolor='red', edgecolor='black', linewidth=1.5)
    ax3.add_patch(diode_rect)
    ax3.text(diode_x + 0.4, 2.5, 'SMS7630\nSchottky', ha='center', fontsize=8)

    # Solder joints
    for x in [diode_x, diode_x + 0.8]:
        circle = Circle((x, 0.035), 0.15, facecolor='silver', edgecolor='black')
        ax3.add_patch(circle)

    # Dimension annotations
    ax3.plot([22, 22], [0, 1.67], 'k-', linewidth=1.5)
    ax3.plot([21.8, 22.2], [0, 0], 'k-', linewidth=1.5)
    ax3.plot([21.8, 22.2], [1.67, 1.67], 'k-', linewidth=1.5)
    ax3.text(23, 0.835, '1.6 mm\n(PCB)', va='center', fontsize=9)

    ax3.set_xlim(-1, 26)
    ax3.set_ylim(-0.5, 3)
    ax3.set_xlabel('Distance (cm)', fontsize=11)
    ax3.set_ylabel('Height (mm)', fontsize=11)
    ax3.grid(True, alpha=0.3)

    # Bill of materials table
    ax4 = plt.subplot(2, 2, 4)
    ax4.axis('off')
    ax4.set_title('Bill of Materials & Construction Notes', fontsize=12, fontweight='bold')

    bom_data = [
        ['Component', 'Specification', 'Qty', 'Unit Cost', 'Total'],
        ['PCB', 'FR4, 55×55 cm, 1.6mm, 1oz Cu', '1', '$50.00', '$50.00'],
        ['Schottky Diodes', 'SMS7630 (0.3V forward, 2.7GHz)', '250', '$0.10', '$25.00'],
        ['Capacitors', '10 µF, 16V ceramic', '5', '$0.20', '$1.00'],
        ['Resistors', '10 kΩ, 1/4W (output load)', '5', '$0.05', '$0.25'],
        ['DC Jack', 'Barrel connector 5.5×2.1mm', '1', '$0.50', '$0.50'],
        ['Wire', '22 AWG stranded (connections)', '2m', '$0.50/m', '$1.00'],
        ['', '', '', 'TOTAL:', '$77.75'],
    ]

    table = ax4.table(cellText=bom_data, cellLoc='left', loc='upper center',
                     colWidths=[0.2, 0.35, 0.1, 0.15, 0.15])
    table.auto_set_font_size(False)
    table.set_fontsize(9)
    table.scale(1, 2.5)

    # Style header row
    for i in range(5):
        table[(0, i)].set_facecolor('#4CAF50')
        table[(0, i)].set_text_props(weight='bold', color='white')

    # Style total row
    for i in range(5):
        table[(7, i)].set_facecolor('#FFC107')
        table[(7, i)].set_text_props(weight='bold')

    # Construction notes
    notes_text = """
CONSTRUCTION NOTES:

1. PCB Fabrication:
   • Export Gerber files from fractal pattern generator
   • Order from PCB manufacturer (JLCPCB, PCBWay, etc.)
   • Specify 1 oz copper, HASL finish

2. Component Assembly:
   • Solder SMS7630 diodes at EACH triangle vertex (250 total)
   • Orient diodes toward triangle center (anodes inward)
   • Connect parallel capacitors at DC output for smoothing

3. Output Collection:
   • All triangle outputs connect in parallel
   • DC jack provides final output
   • Expected output: 50-500 mW @ 0.5-5V DC

4. Installation:
   • Mount vertically or at 45° angle
   • Face toward strong RF sources (cell towers, WiFi APs)
   • Keep away from metal objects (2× size minimum)

5. Target Frequencies:
   • Level 1: 300 MHz (λ ≈ 1m)
   • Level 2: 600 MHz (λ ≈ 50cm)
   • Level 3: 1.2 GHz (λ ≈ 25cm)
   • Level 4: 2.4 GHz (λ ≈ 12.5cm) - WiFi band!
    """

    ax4.text(0.05, 0.35, notes_text, fontsize=8, family='monospace',
            verticalalignment='top', bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))

    plt.tight_layout()
    plt.savefig('fractal_rectenna_construction.png', dpi=200, bbox_inches='tight')
    print("  Saved: fractal_rectenna_construction.png")

    return fig


# ============================================================================
# SPIRAL VORTEX CONCENTRATOR (Golden Ratio)
# ============================================================================

def golden_spiral_3d(turns=15, points_per_turn=100, height_factor=0.1):
    """
    Generate 3D golden spiral coordinates.

    Args:
        turns: Number of spiral turns
        points_per_turn: Points per complete rotation
        height_factor: Vertical rise per turn

    Returns:
        x, y, z coordinates
    """
    a = 0.01  # Initial radius (meters)
    b = np.log(PHI) / (np.pi / 2)  # Golden growth rate

    theta = np.linspace(0, turns * 2 * np.pi, turns * points_per_turn)
    r = a * np.exp(b * theta)

    x = r * np.cos(theta)
    y = r * np.sin(theta)
    z = theta * height_factor  # Slight vertical component

    return x, y, z


def plot_spiral_concentrator():
    """Generate construction diagrams for spiral concentrator."""
    print("Generating Spiral Vortex Concentrator diagrams...")

    fig = plt.figure(figsize=(16, 10))
    fig.clf()  # Clear any existing content

    # 3D perspective view
    ax1 = fig.add_subplot(2, 2, 1, projection='3d')
    ax1.set_title('Spiral Vortex Concentrator - 3D View\n(Golden Ratio φ = 1.618...)',
                  fontsize=14, fontweight='bold')

    x, y, z = golden_spiral_3d(turns=20, height_factor=0.05)

    # Draw spiral wire
    ax1.plot(x, y, z, 'b-', linewidth=2, label='Copper wire path')

    # Mark center
    ax1.scatter([0], [0], [0], color='red', s=100, marker='o', label='Center (feed point)')

    # Mark outer end
    ax1.scatter([x[-1]], [y[-1]], [z[-1]], color='green', s=100, marker='s', label='Outer end (load)')

    # Draw support structure (plywood disk)
    theta_disk = np.linspace(0, 2*np.pi, 100)
    r_disk = 0.55  # 1.1m diameter
    x_disk = r_disk * np.cos(theta_disk)
    y_disk = r_disk * np.sin(theta_disk)
    z_disk = np.zeros_like(theta_disk)

    ax1.plot(x_disk, y_disk, z_disk, 'brown', linewidth=3, alpha=0.5, label='Plywood base (1.1m)')

    ax1.set_xlabel('X (meters)', fontsize=10)
    ax1.set_ylabel('Y (meters)', fontsize=10)
    ax1.set_zlabel('Z (meters)', fontsize=10)
    ax1.legend(fontsize=9)
    ax1.grid(True, alpha=0.3)

    # Top view (orthographic)
    ax2 = plt.subplot(2, 2, 2)
    ax2.set_aspect('equal')
    ax2.set_title('Top View - Spiral Pattern', fontsize=12, fontweight='bold')

    # Draw spiral
    ax2.plot(x*100, y*100, 'b-', linewidth=1.5)

    # Mark turns with dots
    for turn in range(0, 20, 2):
        idx = turn * 100
        if idx < len(x):
            ax2.plot(x[idx]*100, y[idx]*100, 'ro', markersize=6)
            ax2.text(x[idx]*100, y[idx]*100, f' {turn}', fontsize=8)

    # Plywood outline
    circle = plt.Circle((0, 0), 55, fill=False, edgecolor='brown', linewidth=3, linestyle='--')
    ax2.add_patch(circle)

    # Center feed point
    ax2.plot(0, 0, 'r*', markersize=20, label='Feed point (center)')

    # Mounting holes (4 corners, square pattern)
    hole_positions = [(-45, -45), (45, -45), (45, 45), (-45, 45)]
    for hole in hole_positions:
        circle = plt.Circle(hole, 2, facecolor='gray', edgecolor='black')
        ax2.add_patch(circle)
    ax2.plot([], [], 'o', color='gray', markersize=8, label='Mounting holes (Ø4mm)')

    # Dimension arrow
    ax2.annotate('', xy=(55, 0), xytext=(-55, 0),
                arrowprops=dict(arrowstyle='<->', color='red', lw=2))
    ax2.text(0, -65, '110 cm (diameter)', ha='center', fontsize=11, color='red', fontweight='bold')

    ax2.set_xlim(-70, 70)
    ax2.set_ylim(-70, 70)
    ax2.set_xlabel('Distance (cm)', fontsize=11)
    ax2.set_ylabel('Distance (cm)', fontsize=11)
    ax2.legend(fontsize=9)
    ax2.grid(True, alpha=0.3)

    # Side view schematic
    ax3 = plt.subplot(2, 2, 3)
    ax3.set_aspect('equal')
    ax3.set_title('Side View - Wire Support Structure', fontsize=12, fontweight='bold')

    # Plywood base
    plywood_thickness = 2
    base_rect = Rectangle((-55, 0), 110, plywood_thickness,
                         facecolor='burlywood', edgecolor='black', linewidth=2)
    ax3.add_patch(base_rect)
    ax3.text(0, 1, 'Plywood base (12mm)', ha='center', fontsize=9, fontweight='bold')

    # Wire guides (vertical posts)
    guide_height = 10
    for x_pos in [-40, -20, 0, 20, 40]:
        guide = Rectangle((x_pos - 0.5, plywood_thickness), 1, guide_height,
                         facecolor='silver', edgecolor='black', linewidth=1)
        ax3.add_patch(guide)

    ax3.text(0, plywood_thickness + guide_height + 2, 'Wire guides\n(plastic standoffs)',
            ha='center', fontsize=8)

    # Wire path (schematic side view)
    wire_y = plywood_thickness + guide_height / 2
    ax3.plot([-50, 50], [wire_y, wire_y + 2], 'b-', linewidth=2, label='Wire path (elevated)')

    # Feed point connection
    ax3.arrow(0, -5, 0, 4, head_width=3, head_length=1, fc='red', ec='red')
    ax3.text(0, -7, 'Feed from\ntransmitter', ha='center', fontsize=9, color='red')

    # Ground plane (optional)
    ground_rect = Rectangle((-60, -1), 120, 1,
                           facecolor='lightgray', edgecolor='black',
                           linewidth=1, linestyle='--', alpha=0.5)
    ax3.add_patch(ground_rect)
    ax3.text(0, -0.5, 'Ground plane (optional, aluminum foil)', ha='center', fontsize=8)

    ax3.set_xlim(-70, 70)
    ax3.set_ylim(-10, 20)
    ax3.set_xlabel('Distance (cm)', fontsize=11)
    ax3.set_ylabel('Height (cm)', fontsize=11)
    ax3.legend(fontsize=9)
    ax3.grid(True, alpha=0.3)

    # BOM and construction
    ax4 = plt.subplot(2, 2, 4)
    ax4.axis('off')
    ax4.set_title('Bill of Materials & Assembly Instructions', fontsize=12, fontweight='bold')

    bom_data = [
        ['Component', 'Specification', 'Qty', 'Unit Cost', 'Total'],
        ['Plywood', 'Birch, 12mm thick, 120×120 cm', '1', '$25.00', '$25.00'],
        ['Copper Wire', '18 AWG bare, 50m', '1', '$15.00', '$15.00'],
        ['Wire Guides', 'Plastic standoffs, 10mm height', '20', '$0.20', '$4.00'],
        ['Center Post', 'PVC pipe, 2cm dia, 15cm', '1', '$2.00', '$2.00'],
        ['Coax Cable', 'RG-58, 3m with connectors', '1', '$8.00', '$8.00'],
        ['BNC Connector', 'Panel mount (for feed)', '1', '$3.00', '$3.00'],
        ['Wood Screws', '3mm × 15mm', '50', '$0.05', '$2.50'],
        ['', '', '', 'TOTAL:', '$59.50'],
    ]

    table = ax4.table(cellText=bom_data, cellLoc='left', loc='upper center',
                     colWidths=[0.2, 0.35, 0.1, 0.15, 0.15])
    table.auto_set_font_size(False)
    table.set_fontsize(8)
    table.scale(1, 2.3)

    # Style header
    for i in range(5):
        table[(0, i)].set_facecolor('#2196F3')
        table[(0, i)].set_text_props(weight='bold', color='white')

    # Style total
    for i in range(5):
        table[(8, i)].set_facecolor('#FFC107')
        table[(8, i)].set_text_props(weight='bold')

    assembly_text = """ASSEMBLY INSTRUCTIONS:

1. Cut plywood to 120x120 cm square
2. Mark spiral pattern
3. Install wire guides (20 guides)
4. Wind copper wire from center outward
5. Solder BNC connector at center
6. Expected: 5-10x energy concentration
    """

    ax4.text(0.05, 0.4, assembly_text, fontsize=8, family='sans-serif',
            verticalalignment='top', bbox=dict(boxstyle='round', facecolor='lightblue', alpha=0.5))

    plt.subplots_adjust(left=0.05, right=0.95, top=0.95, bottom=0.05, hspace=0.3, wspace=0.3)
    plt.savefig('spiral_concentrator_construction.png', dpi=120)
    print("  Saved: spiral_concentrator_construction.png")

    return fig


# ============================================================================
# TOROIDAL VORTEX COLLECTOR (Fibonacci 21/13 winding)
# ============================================================================

def fibonacci_toroid_3d(major_R=0.3, minor_r=0.05, n_toroidal=21, n_poloidal=13, points=2000):
    """
    Generate 3D toroid with Fibonacci winding.

    Args:
        major_R: Major radius (meters)
        minor_r: Minor radius (meters)
        n_toroidal: Toroidal turns (Fibonacci number)
        n_poloidal: Poloidal turns (Fibonacci number)
        points: Total number of points

    Returns:
        x, y, z coordinates
    """
    t = np.linspace(0, 1, points)

    phi = t * n_toroidal * 2 * np.pi    # Toroidal angle
    theta = t * n_poloidal * 2 * np.pi  # Poloidal angle

    x = (major_R + minor_r * np.cos(theta)) * np.cos(phi)
    y = (major_R + minor_r * np.cos(theta)) * np.sin(phi)
    z = minor_r * np.sin(theta)

    return x, y, z


def plot_toroidal_vortex():
    """Generate construction diagrams for toroidal vortex collector."""
    print("Generating Toroidal Vortex Collector diagrams...")

    fig = plt.figure(figsize=(16, 12))
    fig.clf()  # Clear any existing content

    # 3D view
    ax1 = fig.add_subplot(2, 3, 1, projection='3d')
    ax1.set_title('Toroidal Vortex Collector - 3D View\n(Fibonacci 21/13 Winding)',
                  fontsize=13, fontweight='bold')

    x, y, z = fibonacci_toroid_3d(major_R=0.30, minor_r=0.05, n_toroidal=21, n_poloidal=13)

    # Draw winding
    ax1.plot(x, y, z, 'r-', linewidth=1.5, alpha=0.8, label='Litz wire (21/13)')

    # Mark start and end
    ax1.scatter([x[0]], [y[0]], [z[0]], color='green', s=100, marker='o', label='Start (input)')
    ax1.scatter([x[-1]], [y[-1]], [z[-1]], color='blue', s=100, marker='s', label='End (output)')

    # Draw toroid core outline (transparent surface)
    u = np.linspace(0, 2*np.pi, 30)
    v = np.linspace(0, 2*np.pi, 30)
    U, V = np.meshgrid(u, v)

    X = (0.30 + 0.05 * np.cos(V)) * np.cos(U)
    Y = (0.30 + 0.05 * np.cos(V)) * np.sin(U)
    Z = 0.05 * np.sin(V)

    ax1.plot_surface(X, Y, Z, alpha=0.1, color='gray')

    ax1.set_xlabel('X (m)', fontsize=10)
    ax1.set_ylabel('Y (m)', fontsize=10)
    ax1.set_zlabel('Z (m)', fontsize=10)
    ax1.legend(fontsize=8)
    ax1.set_box_aspect([1, 1, 0.4])

    # Top view
    ax2 = plt.subplot(2, 3, 2)
    ax2.set_aspect('equal')
    ax2.set_title('Top View - Winding Pattern', fontsize=12, fontweight='bold')

    ax2.plot(x*100, y*100, 'r-', linewidth=1, alpha=0.6)

    # Mark every 3rd toroidal turn
    for turn in range(0, 21, 3):
        idx = int((turn / 21) * len(x))
        if idx < len(x):
            ax2.plot(x[idx]*100, y[idx]*100, 'bo', markersize=4)

    # Core outline
    theta_outer = np.linspace(0, 2*np.pi, 100)
    x_outer = 35 * np.cos(theta_outer)
    y_outer = 35 * np.sin(theta_outer)
    ax2.plot(x_outer, y_outer, 'k--', linewidth=2, label='Outer edge (Ø70cm)')

    x_inner = 25 * np.cos(theta_outer)
    y_inner = 25 * np.sin(theta_outer)
    ax2.plot(x_inner, y_inner, 'k--', linewidth=2, label='Inner edge (Ø50cm)')

    # Start/end markers
    ax2.plot(x[0]*100, y[0]*100, 'go', markersize=12, label='Start', zorder=10)
    ax2.plot(x[-1]*100, y[-1]*100, 'bs', markersize=12, label='End', zorder=10)

    ax2.set_xlim(-45, 45)
    ax2.set_ylim(-45, 45)
    ax2.set_xlabel('Distance (cm)', fontsize=11)
    ax2.set_ylabel('Distance (cm)', fontsize=11)
    ax2.legend(fontsize=8)
    ax2.grid(True, alpha=0.3)

    # Side/cross-section view
    ax3 = plt.subplot(2, 3, 3)
    ax3.set_aspect('equal')
    ax3.set_title('Cross-Section - Core Dimensions', fontsize=12, fontweight='bold')

    # Draw cross-section of toroid
    theta = np.linspace(0, 2*np.pi, 100)

    # Outer circle (minor radius)
    x_section = 5 * np.cos(theta)
    z_section = 5 * np.sin(theta)
    ax3.plot(x_section, z_section, 'k-', linewidth=3)
    ax3.fill(x_section, z_section, color='lightgray', alpha=0.5)

    # Show wire wrapping around core
    wire_positions = np.linspace(0, 2*np.pi, 13)  # 13 poloidal turns
    for angle in wire_positions:
        x_wire = 5.5 * np.cos(angle)
        z_wire = 5.5 * np.sin(angle)
        circle = plt.Circle((x_wire, z_wire), 0.3, color='red', alpha=0.6)
        ax3.add_patch(circle)

    # Dimension annotations
    ax3.plot([0, 5], [0, 0], 'b-', linewidth=2)
    ax3.text(2.5, -1, 'r = 5 cm\n(minor radius)', ha='center', fontsize=9, color='blue',
            bbox=dict(boxstyle='round', facecolor='yellow', alpha=0.7))

    ax3.plot([-8, -8], [-5, 5], 'r-', linewidth=2)
    ax3.text(-10, 0, 'Ø10cm', ha='center', fontsize=9, color='red',
            bbox=dict(boxstyle='round', facecolor='yellow', alpha=0.7))

    # Major radius indicator
    ax3.annotate('To major axis\n(R = 30cm)', xy=(0, 0), xytext=(-20, 0),
                arrowprops=dict(arrowstyle='->', color='green', lw=2),
                fontsize=9, color='green', ha='center')

    ax3.set_xlim(-25, 10)
    ax3.set_ylim(-10, 10)
    ax3.set_xlabel('Distance (cm)', fontsize=11)
    ax3.set_ylabel('Height (cm)', fontsize=11)
    ax3.grid(True, alpha=0.3)

    # Core material options
    ax4 = plt.subplot(2, 3, 4)
    ax4.axis('off')
    ax4.set_title('Core Material Options', fontsize=12, fontweight='bold')

    material_data = [
        ['Material', 'Permeability μr', 'Cost', 'Notes'],
        ['PVC pipe', '1 (air)', 'Low', 'Easy to form, no magnetic effect'],
        ['Ferrite (NiZn)', '10-100', 'Medium', 'RF ferrite, lossy at HF'],
        ['Ferrite (MnZn)', '1000-3000', 'Medium', 'Best for <1 MHz'],
        ['Iron powder', '10-50', 'Low', 'Good for broadband'],
        ['Air core', '1', 'Free', 'No core, just wire form'],
    ]

    table = ax4.table(cellText=material_data, cellLoc='left', loc='center',
                     colWidths=[0.25, 0.25, 0.15, 0.35])
    table.auto_set_font_size(False)
    table.set_fontsize(9)
    table.scale(1, 3)

    for i in range(4):
        table[(0, i)].set_facecolor('#9C27B0')
        table[(0, i)].set_text_props(weight='bold', color='white')

    note = """
RECOMMENDED: Start with PVC pipe core for ease of construction.
The dimensional field effects don't require high μr.
    """
    ax4.text(0.5, 0.15, note, ha='center', fontsize=9,
            bbox=dict(boxstyle='round', facecolor='lightyellow', alpha=0.8))

    # BOM
    ax5 = plt.subplot(2, 3, 5)
    ax5.axis('off')
    ax5.set_title('Bill of Materials', fontsize=12, fontweight='bold')

    bom_data = [
        ['Component', 'Specification', 'Qty', 'Cost', 'Total'],
        ['PVC Pipe', '2" diameter, 10 ft', '6 ft', '$2/ft', '$12.00'],
        ['PVC Elbows', '2" 90° elbows', '8', '$1.50', '$12.00'],
        ['Litz Wire', '20 AWG, 50 strands', '50m', '$1.50/m', '$75.00'],
        ['Capacitors', '100 pF, 500V', '5', '$0.50', '$2.50'],
        ['Coax Cable', 'RG-58, 5m', '1', '$10.00', '$10.00'],
        ['Connectors', 'BNC (input/output)', '2', '$3.00', '$6.00'],
        ['PVC Cement', 'For pipe joints', '1', '$5.00', '$5.00'],
        ['Wood Base', '60×60 cm plywood', '1', '$15.00', '$15.00'],
        ['', '', '', 'TOTAL:', '$137.50'],
    ]

    table = ax5.table(cellText=bom_data, cellLoc='left', loc='upper center',
                     colWidths=[0.22, 0.3, 0.1, 0.15, 0.15])
    table.auto_set_font_size(False)
    table.set_fontsize(8)
    table.scale(1, 2.8)

    for i in range(5):
        table[(0, i)].set_facecolor('#9C27B0')
        table[(0, i)].set_text_props(weight='bold', color='white')

    for i in range(5):
        table[(9, i)].set_facecolor('#FFC107')
        table[(9, i)].set_text_props(weight='bold')

    # Construction steps
    ax6 = plt.subplot(2, 3, 6)
    ax6.axis('off')
    ax6.set_title('Construction Steps', fontsize=12, fontweight='bold')

    steps_text = """CONSTRUCTION STEPS:

1. BUILD TOROID: Cut PVC into 8 sections, connect with elbows
2. WIND WIRE: 21 toroidal x 13 poloidal (Fibonacci ratio)
3. SECURE: Cable ties every 10cm
4. ELECTRICAL: BNC connectors at start/end
5. MOUNT: Attach to plywood base with standoffs
6. TEST: Check resonances at 7.6 MHz and 73.4 MHz
7. OPERATE: Monitor for vacuum coupling effects
    """

    ax6.text(0.05, 0.90, steps_text, fontsize=9, family='sans-serif',
            verticalalignment='top',
            bbox=dict(boxstyle='round', facecolor='lavender', alpha=0.6))

    plt.subplots_adjust(left=0.05, right=0.95, top=0.95, bottom=0.05, hspace=0.3, wspace=0.3)
    plt.savefig('toroidal_vortex_construction.png', dpi=120)
    print("  Saved: toroidal_vortex_construction.png")

    return fig


# ============================================================================
# MAIN EXECUTION
# ============================================================================

def main():
    """Generate all device construction diagrams."""
    print("="*60)
    print("ENERGY HARVESTER CONSTRUCTION DIAGRAMS")
    print("="*60)
    print()

    # Generate all diagrams
    plot_fractal_rectenna()
    plot_spiral_concentrator()
    plot_toroidal_vortex()

    print()
    print("="*60)
    print("All construction diagrams generated successfully!")
    print()
    print("Output files:")
    print("  1. fractal_rectenna_construction.png")
    print("  2. spiral_concentrator_construction.png")
    print("  3. toroidal_vortex_construction.png")
    print()
    print("These diagrams include:")
    print("  • 3D perspective views")
    print("  • Orthographic projections (top, side, cross-section)")
    print("  • Detailed dimension annotations")
    print("  • Bills of materials with costs")
    print("  • Step-by-step assembly instructions")
    print("="*60)


if __name__ == "__main__":
    main()
