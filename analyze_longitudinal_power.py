#!/usr/bin/env python3
"""
Analyze longitudinal wave wireless power transfer simulation results.

This script:
1. Loads CSV data from the simulation
2. Creates visualization plots
3. Computes efficiency metrics
4. Compares with classical predictions
"""

import numpy as np
import matplotlib.pyplot as plt
import pandas as pd
from pathlib import Path

# Configure matplotlib for nice plots
plt.style.use('seaborn-v0_8-darkgrid')
plt.rcParams['figure.figsize'] = (12, 8)
plt.rcParams['font.size'] = 10

def load_data():
    """Load simulation results from CSV files."""
    print("Loading simulation data...")

    # Load power transfer efficiency vs distance
    transfer_file = Path("longitudinal_power_transfer.csv")
    if not transfer_file.exists():
        print(f"ERROR: {transfer_file} not found. Did the simulation complete?")
        return None, None

    transfer_data = pd.read_csv(transfer_file)

    # Load power vs time data
    time_file = Path("power_vs_time.csv")
    if not time_file.exists():
        print(f"WARNING: {time_file} not found.")
        time_data = None
    else:
        time_data = pd.read_csv(time_file)

    print(f"  Loaded {len(transfer_data)} distance measurements")
    if time_data is not None:
        print(f"  Loaded {len(time_data)} timesteps")

    return transfer_data, time_data

def plot_efficiency_vs_distance(transfer_data):
    """Plot power transfer efficiency vs distance."""
    print("\nCreating efficiency vs distance plot...")

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 10))

    distances = transfer_data['distance'].values
    power_density = transfer_data['power_density'].values
    energy_received = transfer_data['energy_received'].values

    # Plot 1: Power received vs distance (linear scale)
    ax1.plot(distances, power_density, 'o-', linewidth=2, markersize=8,
             label='Power Density', color='blue')
    ax1.plot(distances, energy_received, 's--', linewidth=2, markersize=8,
             label='Total Energy Received', color='red')

    # Classical 1/r² falloff for comparison
    classical = power_density[0] * (distances[0] / distances)**2
    ax1.plot(distances, classical, ':', linewidth=2,
             label='Classical 1/r² (dipole)', color='gray', alpha=0.7)

    ax1.set_xlabel('Distance from Transmitter (units)', fontsize=12)
    ax1.set_ylabel('Power Density', fontsize=12)
    ax1.set_title('Longitudinal Wave Power Transfer', fontsize=14, fontweight='bold')
    ax1.legend(fontsize=10)
    ax1.grid(True, alpha=0.3)
    ax1.set_xlim(left=0)
    ax1.set_ylim(bottom=0)

    # Plot 2: Same data on log-log scale to see power law
    ax2.loglog(distances, power_density, 'o-', linewidth=2, markersize=8,
               label='Power Density', color='blue')
    ax2.loglog(distances, energy_received, 's--', linewidth=2, markersize=8,
               label='Total Energy', color='red')
    ax2.loglog(distances, classical, ':', linewidth=2,
               label='Classical 1/r² (dipole)', color='gray', alpha=0.7)

    ax2.set_xlabel('Distance from Transmitter (units)', fontsize=12)
    ax2.set_ylabel('Power Density (log scale)', fontsize=12)
    ax2.set_title('Log-Log Plot: Power Law Falloff', fontsize=14, fontweight='bold')
    ax2.legend(fontsize=10)
    ax2.grid(True, alpha=0.3, which='both')

    plt.tight_layout()
    plt.savefig('longitudinal_power_efficiency.png', dpi=150, bbox_inches='tight')
    print("  Saved: longitudinal_power_efficiency.png")

    # Compute power law exponent
    if len(distances) >= 3:
        # Filter out zero values
        nonzero = power_density > 0
        if nonzero.sum() >= 3:
            log_dist = np.log(distances[nonzero])
            log_power = np.log(power_density[nonzero])
            # Linear fit in log space: log(P) = log(A) - n*log(r)
            coeffs = np.polyfit(log_dist, log_power, 1)
            exponent = -coeffs[0]
            print(f"\n  Power falloff exponent: r^(-{exponent:.2f})")
            print(f"  (Classical dipole: r^(-2.0))")

            if exponent < 1.5:
                print(f"  *** Longitudinal waves show SLOWER falloff than dipole! ***")
            elif exponent > 2.5:
                print(f"  *** Longitudinal waves show FASTER falloff than dipole ***")
            else:
                print(f"  *** Falloff similar to classical dipole ***")

    return fig

def plot_power_vs_time(time_data, transfer_data):
    """Plot power density vs time at each receiver."""
    if time_data is None:
        print("\nSkipping power vs time plot (no data)")
        return None

    print("\nCreating power vs time plot...")

    fig, ax = plt.subplots(figsize=(14, 8))

    # Time data has columns: time, power_5m, power_10m, power_15m, ...
    time_vals = time_data['time'].values

    # Plot each power column
    power_columns = [col for col in time_data.columns if col.startswith('power_')]
    colors = plt.cm.viridis(np.linspace(0, 1, len(power_columns)))

    for i, col in enumerate(power_columns):
        # Extract distance from column name (e.g., "power_5m" -> "5m")
        dist_label = col.replace('power_', '')
        power_vals = time_data[col].values

        ax.plot(time_vals, power_vals, linewidth=1.5,
                label=f'd = {dist_label}', color=colors[i], alpha=0.8)

    ax.set_xlabel('Time (simulation units)', fontsize=12)
    ax.set_ylabel('Power Density at Receiver', fontsize=12)
    ax.set_title('Power Density vs Time at Different Distances',
                 fontsize=14, fontweight='bold')
    ax.legend(fontsize=10, loc='best', ncol=2)
    ax.grid(True, alpha=0.3)
    ax.set_xlim(left=0)

    plt.tight_layout()
    plt.savefig('power_vs_time.png', dpi=150, bbox_inches='tight')
    print("  Saved: power_vs_time.png")

    return fig

def compute_metrics(transfer_data):
    """Compute and display key performance metrics."""
    print("\n" + "="*60)
    print("WIRELESS POWER TRANSFER METRICS")
    print("="*60)

    distances = transfer_data['distance'].values
    power_density = transfer_data['power_density'].values
    efficiency = transfer_data['efficiency'].values
    energy_received = transfer_data['energy_received'].values

    # Reference power at closest distance
    P0 = power_density[0]
    d0 = distances[0]

    print(f"\nReference (d = {d0:.1f}):")
    print(f"  Power density: {P0:.6e}")
    print(f"  Energy received: {energy_received[0]:.6f}")

    print(f"\nPower Transfer Efficiency:")
    for i, d in enumerate(distances):
        eff = efficiency[i]
        classical_eff = (d0 / d)**2 * 100

        print(f"  d = {d:5.1f}:  {eff:6.2f}%  "
              f"(classical: {classical_eff:6.2f}%)  "
              f"ratio: {eff/classical_eff if classical_eff > 0 else 0:.2f}x")

    # Energy comparison
    print(f"\nTotal Energy Received (integrated over time):")
    for i, d in enumerate(distances):
        print(f"  d = {d:5.1f}:  {energy_received[i]:.4f}")

    # Practical engineering estimates
    print(f"\n" + "-"*60)
    print("PRACTICAL ENGINEERING ESTIMATES")
    print("-"*60)

    # Assume transmitter power = 1 kW, grid spacing = 1 meter
    tx_power_watts = 1000  # 1 kW transmitter
    grid_spacing_m = 1.0   # meters per grid unit

    print(f"\nAssuming:")
    print(f"  - Transmitter power: {tx_power_watts} W")
    print(f"  - Grid spacing: {grid_spacing_m} m/unit")

    # Power density to watts conversion (very rough!)
    # Assume receiver has 1 m² collection area
    receiver_area_m2 = 1.0

    print(f"\nReceived Power (at 1 m² receiver):")
    for i, d in enumerate(distances):
        d_meters = d * grid_spacing_m
        # Scale power density by transmitter power and receiver area
        if P0 > 0:
            received_watts = power_density[i] * tx_power_watts * receiver_area_m2 / P0
        else:
            received_watts = 0

        print(f"  d = {d_meters:5.1f} m:  {received_watts:8.2f} W  "
              f"({received_watts/tx_power_watts*100:5.2f}% efficiency)")

    print("\nNOTE: These are rough estimates for illustration purposes!")
    print("Real efficiency depends on antenna design, matching, losses, etc.")

def main():
    """Main analysis routine."""
    print("="*60)
    print("LONGITUDINAL WAVE WIRELESS POWER ANALYSIS")
    print("="*60)

    # Load data
    transfer_data, time_data = load_data()
    if transfer_data is None:
        print("\nERROR: Could not load simulation data. Exiting.")
        return

    # Create plots
    plot_efficiency_vs_distance(transfer_data)
    plot_power_vs_time(time_data, transfer_data)

    # Compute metrics
    compute_metrics(transfer_data)

    print("\n" + "="*60)
    print("Analysis complete! Check the generated PNG files.")
    print("="*60)

    # Don't show plots interactively (runs in CLI)
    # plt.show()

if __name__ == "__main__":
    main()
